use crate::zettel::{self, Id};
use crate::{zettel::Zettel, DateTime, ZettelMeta};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io::prelude::*,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    SerializationError(serde_yaml::Error),
    ZettelError(zettel::Error),
    Other(String),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => e.fmt(f),
            Self::ZettelError(e) => e.fmt(f),
            Self::SerializationError(e) => e.fmt(f),
            Self::Other(e) => e.fmt(f),
        }
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(e: serde_yaml::Error) -> Self {
        Self::SerializationError(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<std::io::ErrorKind> for Error {
    fn from(e: std::io::ErrorKind) -> Self {
        Self::IoError(e.into())
    }
}

impl From<zettel::Error> for Error {
    fn from(e: zettel::Error) -> Self {
        Self::ZettelError(e)
    }
}

impl From<String> for Error {
    fn from(e: String) -> Self {
        Self::Other(e)
    }
}

impl From<&str> for Error {
    fn from(e: &str) -> Self {
        Self::Other(e.to_owned())
    }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct ZkContents {
    pub meta: ZkMeta,
    pub default_frontmatter: HashMap<String, String>,
    // TODO: should be BTreeMap because ID is already totally ordered
    pub zettels: HashMap<zettel::Id, ZettelMeta>,
}

/// Store of zettels on the filesystem
#[derive(Debug, PartialEq)]
pub struct Zettelkasten {
    /// this is what gets serialized into the database
    contents: ZkContents,
    /// directory containing zettels and database file
    root_path: PathBuf,
    /// relative path to database file
    db_path: PathBuf,
}

impl AsRef<Self> for Zettelkasten {
    fn as_ref<'a>(&'a self) -> &'a Self {
        return &self;
    }
}

impl Zettelkasten {
    pub fn builder() -> ZettelkastenBuilder {
        Default::default()
    }

    /// Returns `None` if the path does not exist.
    pub fn open(path: impl AsRef<Path>) -> Result<Option<Self>> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(None);
        }
        let path = match resolve_db_path(path)? {
            Some(path) => path,
            None => return Ok(None),
        };
        let filename = path
            .file_name()
            .ok_or_else(|| format!("doesn't appear to be a file: {}", path.display()))?
            .to_str()
            .unwrap()
            .to_owned();
        let contents: ZkContents = {
            let file = File::open(&path)?;
            match filename.split(".").last() {
                Some("yaml") | Some("yml") => serde_yaml::from_reader(file)?,
                Some(suf) => return Err(format!("unrecognized db suffix {suf}").into()),
                _ => return Err(format!("no file suffix for {}", path.display()).into()),
            }
        };
        let root_path = path
            .parent()
            .ok_or_else(|| format!("no parent directory for {}", path.display()))?
            .into();
        Ok(Some(Self {
            contents,
            root_path,
            db_path: path.into(),
        }))
    }

    pub fn add(&mut self, zettel: impl AsRef<Zettel>) -> Result<()> {
        let zettel = zettel.as_ref();
        let path = self.abs_path(&zettel.meta.path);
        if path.exists() {
            return Err(format!("path already exists: {}", path.display()).into());
        }
        let mut file = File::create(&path)?;
        let zettel_str = zettel.as_string(&self.contents.default_frontmatter)?;
        file.write_all(zettel_str.as_bytes())?;
        self.contents
            .zettels
            .insert(zettel.meta.id.clone(), zettel.meta.clone());
        Ok(())
    }

    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    pub fn path_to(&self, id: &Id) -> Result<PathBuf> {
        let zettel_meta = self.get(id)?;
        Ok(self.abs_path(zettel_meta.path()))
    }

    pub fn get(&self, id: &Id) -> Result<&ZettelMeta> {
        self.contents
            .zettels
            .get(id)
            .ok_or_else(|| format!("no zettel with id {id}").into())
    }

    pub fn db_path(&self) -> PathBuf {
        let mut path = PathBuf::from(self.root_path());
        path.push(&self.db_path);
        path
    }

    /// compute the absolute path of the given relative path
    /// i.e., stick the root_path in front of if
    fn abs_path(&self, path: impl AsRef<Path>) -> PathBuf {
        let mut abs_path = PathBuf::from(self.root_path());
        abs_path.push(path);
        abs_path
    }

    pub fn sync(&mut self) -> Result<()> {
        self.sync_dir(self.root_path().to_owned())
    }

    fn sync_dir(&mut self, dir_path: PathBuf) -> Result<()> {
        let dir_entries = std::fs::read_dir(&dir_path)?;
        for entry in dir_entries {
            let entry: std::fs::DirEntry = entry.unwrap();
            let path = entry.path();
            if path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("_zettel")
            {
                continue;
            }
            if path.is_dir() {
                self.sync_dir(path)?;
            } else {
                self.sync_file(path);
            }
        }
        Ok(())
    }

    fn sync_file(&mut self, path: PathBuf) {
        let fm = match crate::frontmatter::parse_yaml_path(&path) {
            Ok(meta) => meta,
            Err(e) => {
                println!(
                    "skipping {} due to frontmatter error: {}",
                    path.to_str().unwrap(),
                    e
                );
                return;
            }
        };
        let id: zettel::Id = {
            let id = fm.get(&"id".into());
            if id.is_none() {
                println!(
                    "skipping {} due to missing key 'id' in frontmatter",
                    path.to_str().unwrap()
                );
                return;
            }
            let id = id.unwrap().as_str();
            if id.is_none() {
                println!(
                    "skipping {} due to 'id' in frontmatter not being a 'string'",
                    path.to_str().unwrap()
                );
                return;
            }
            id.unwrap().to_owned()
        };
        let root_path = self.root_path().to_owned();
        let current_meta = self.contents.zettels.get_mut(&id);
        if current_meta.is_none() {
            println!(
                "no metadata with id {} for zettel at {}; skipping",
                id,
                path.to_str().unwrap(),
            );
            return;
        }
        let current_meta = current_meta.unwrap();
        current_meta.path = path
            .strip_prefix(root_path)
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
        if let Some(title) = fm.get(&"title".into()).and_then(|t| t.as_str()) {
            current_meta.title = title.to_owned()
        }
    }

    /// Export state to database file
    pub fn commit(&self) -> Result<()> {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(self.db_path())?;
        serde_yaml::to_writer(file, &self.contents)?;
        Ok(())
    }
}

/// Metadata about the database
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ZkMeta {
    /// database creation time
    pub created: DateTime,
    /// last modificiation time
    pub modified: DateTime,
}

enum DatabaseKind {
    Yaml,
}

pub struct ZettelkastenBuilder {
    root_path: Option<PathBuf>,
    created: Option<DateTime>,
    modified: Option<DateTime>,
    default_frontmatter: Option<HashMap<String, String>>,
    db_kind: DatabaseKind,
    subdirs: Vec<PathBuf>,
}

impl Default for ZettelkastenBuilder {
    fn default() -> Self {
        Self {
            root_path: None,
            created: None,
            modified: None,
            default_frontmatter: None,
            db_kind: DatabaseKind::Yaml,
            subdirs: Vec::new(),
        }
    }
}

impl ZettelkastenBuilder {
    pub fn root_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.root_path = Some(path.into());
        self
    }
    pub fn yaml(mut self) -> Self {
        self.db_kind = DatabaseKind::Yaml;
        self
    }
    pub fn add_subdir(mut self, path: impl Into<PathBuf>) -> Self {
        self.subdirs.push(path.into());
        self
    }
    pub fn build(self) -> Result<Zettelkasten> {
        let now = chrono::Local::now();
        let root_path = self.root_path.unwrap_or(std::env::current_dir()?);
        for rel_path in self.subdirs {
            let mut path = root_path.clone();
            path.push(rel_path);
            std::fs::create_dir(path)?;
        }
        let zk = Zettelkasten {
            contents: ZkContents {
                meta: ZkMeta {
                    created: now,
                    modified: now,
                },
                default_frontmatter: {
                    if let Some(fm) = self.default_frontmatter {
                        fm
                    } else {
                        let mut fm = HashMap::new();
                        fm.insert("title".to_owned(), "@title".to_owned());
                        fm.insert("id".to_owned(), "@id".to_owned());
                        fm.insert("date".to_owned(), "@created".to_owned());
                        fm
                    }
                },
                zettels: HashMap::new(),
            },
            root_path,
            db_path: PathBuf::from(format!(
                "_zettel.{}",
                match self.db_kind {
                    DatabaseKind::Yaml => "yaml",
                }
            )),
        };
        Ok(zk)
    }
}

fn resolve_db_path(path: &Path) -> Result<Option<PathBuf>> {
    if path.is_dir() {
        let mut yaml_path = PathBuf::from(&path);
        yaml_path.push("_zettel.yaml");
        if yaml_path.exists() {
            return Ok(Some(yaml_path));
        } else {
            return Ok(None);
        }
    } else {
        Ok(Some(path.to_owned()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::{Context, Result};
    use chrono::prelude::*;
    use zettel::Zettel;

    #[test]
    fn create_and_sync() -> Result<()> {
        env_logger::init();
        let tmp_dir = tempdir::TempDir::new("zk_command_test")?;
        let mut zk = Zettelkasten::builder()
            .root_path(tmp_dir.path())
            .yaml()
            .add_subdir("2022")
            .build()
            .context("building zk")?;
        let zettel = Zettel::builder()
            .title("my blog post")
            .created(chrono::Local.timestamp(1431648000, 0))
            .content("A post.")
            .build();
        zk.add(&zettel).context("adding zettel to zk")?;
        let zettel_meta = zk
            .get(zettel.uuid())
            .expect("zettul uuid should be in database before sync")
            .clone();
        let mut new_zettel_path = PathBuf::from(zk.root_path());
        new_zettel_path.push("2022");
        new_zettel_path.push("other.md");
        let old_zettel_path = zk
            .path_to(&zettel_meta.id)
            .context("retrieving zettel path")?;
        std::fs::copy(&old_zettel_path, &new_zettel_path).context("copying zettel")?;
        std::fs::remove_file(old_zettel_path).context("removing zettel")?;
        zk.sync()?;
        let new_zettel_meta = zk
            .get(&zettel_meta.id)
            .expect("zettel uuid should be in database after sync");
        assert_eq!(
            zk.path_to(&zettel_meta.id)?
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
            "2022"
        );
        assert_eq!(zettel_meta.created, new_zettel_meta.created);
        assert_eq!(zettel_meta.modified, new_zettel_meta.modified);
        assert_eq!(zettel_meta.title, new_zettel_meta.title);
        Ok(())
    }

    #[test]
    fn handle_subdirs() -> Result<()> {
        let tmp_dir = tempdir::TempDir::new("zk_command_test")?;
        let (db_path, zettel_uuid) = {
            let mut zk = Zettelkasten::builder()
                .root_path(tmp_dir.path())
                .add_subdir("2022")
                .build()
                .context("building zk")?;
            let zettel = Zettel::builder()
                .title("a zettel in a subdirectory")
                .created(chrono::Local.timestamp(1431648000, 0))
                .subdir("2022")
                .content("A post.")
                .build();
            zk.add(&zettel).context("adding zettel")?;
            zk.commit().context("committing zk")?;
            (zk.db_path(), zettel.uuid().clone())
        };
        let zk: Zettelkasten = Zettelkasten::open(db_path)
            .context("opening zk from database file")?
            .expect("database file to exist");
        let zettel = zk.get(&zettel_uuid).context("retrieving zettel from db")?;
        assert_eq!(zettel.path(), "2022/a-zettel-in-a-subdirectory.md");
        Ok(())
    }
}
