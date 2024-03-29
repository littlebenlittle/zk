use crate::{
    zettel::{Zettel, ZettelMeta},
    zettelkasten::{Zettelkasten, ZkMeta},
    DateTime,
};
use std::{
    collections::HashMap,
    fs::File,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    SerializationError(serde_yaml::Error),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => e.fmt(f),
            Self::SerializationError(e) => e.fmt(f),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(e: serde_yaml::Error) -> Self {
        Self::SerializationError(e)
    }
}
type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Database {
    root_dir: PathBuf,
}

impl Database {
    pub fn new(root_dir: PathBuf) -> Result<Self> {
        Ok(Self {
            root_dir: std::fs::canonicalize(root_dir).unwrap(),
        })
    }

    pub fn root_dir(&self) -> &Path {
        self.root_dir.as_path()
    }

    pub fn get_zk(&self) -> Result<Option<Zettelkasten>> {
        let mut path = self.root_dir.clone();
        path.push("_zettel.yaml");
        if path.is_file() {
            let file = File::open(path)?;
            Ok(Some(serde_yaml::from_reader(file)?))
        } else {
            Ok(None)
        }
    }

    fn make_filename(&self, title: &str, date: DateTime) -> PathBuf {
        let mod_title = title.replace(" ", "-");
        let mut path = self.root_dir.clone();
        let date_str = date.format("%Y-%m-%d");
        let filename = format!("{date_str}-{mod_title}.md");
        path.push(filename);
        path
    }

    pub fn commit(&self, zk: impl AsRef<Zettelkasten>) -> Result<()> {
        let mut path = self.root_dir.clone();
        path.push("_zettel.yaml");
        serde_yaml::to_writer(File::create(&path)?, zk.as_ref())?;
        Ok(())
    }

    pub fn new_zettel(
        &self,
        title: impl AsRef<str>,
        id: impl AsRef<str>,
        date: DateTime,
    ) -> Result<Zettel> {
        let path = self.make_filename(title.as_ref(), date);
        let meta = ZettelMeta {
            created: date,
            modified: date,
            id: id.as_ref().to_owned(),
            title: title.as_ref().to_owned(),
            path: path.to_str().unwrap().to_owned(),
        };
        Ok(Zettel {
            meta,
            content: String::new(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::prelude::*;
    use tempdir::TempDir;

    #[test]
    fn init_db() -> Result<()> {
        let tmp_dir = TempDir::new("zk_yaml_test").expect("couldn't create temp dir");
        let mut db = Database::new(PathBuf::from(tmp_dir.path())).expect("could not create db");
        let zk = Zettelkasten::default();
        db.commit(&zk)?;
        let mut db_path = PathBuf::from(tmp_dir.path());
        db_path.push("_zettel.yaml");
        let file = File::open(db_path).expect("db file wasn't created");
        let read_db: Zettelkasten = serde_yaml::from_reader(file)?;
        assert_eq!(zk, read_db);
        Ok(())
    }

    #[test]
    fn new_zettel() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let tmp_dir = TempDir::new("zk_yaml_test").expect("couldn't create temp dir");
        let root_dir = PathBuf::from(tmp_dir.path());
        let mut db = Database::new(root_dir.clone())?;
        let mut zk = Zettelkasten::default();
        let id = "123456";
        let dt = chrono::Local.timestamp(1431648000, 0);
        let title = "a new blog post";
        let zettel = db.new_zettel(title, id, dt)?;
        zk.add(&zettel)?;
        let zettel_path = Path::new(&zettel.meta.path);
        assert!(zettel_path.exists(), "new zettel was not created on fs");
        let data = std::fs::read_to_string(zettel_path)?;
        let (fm, _) = {
            use extract_frontmatter::{config::Splitter, Extractor};
            let fm_extractor = Extractor::new(Splitter::EnclosingLines("---"));
            fm_extractor.extract(&data)
        };
        let meta: serde_yaml::Mapping = serde_yaml::from_str(&fm)?;
        assert_eq!(
            id,
            meta.get(&"id".into())
                .expect("frontmatter should containt field 'id'"),
            "id in frontmatter does not match"
        );
        assert_eq!(
            "2015-05-14",
            meta.get(&"date".into())
                .expect("frontmatter should containt field 'date'"),
            "date in frontmatter does not match"
        );
        assert_eq!(
            title,
            meta.get(&"title".into())
                .expect("frontmatter should containt field 'title'"),
            "title in frontmatter does not match"
        );
        db.commit(zk)?;
        let new_zk = db.get_zk()?.unwrap();
        assert!(
            new_zk.zettels.len() == 1,
            "new zettel should be reflected in db"
        );
        Ok(())
    }
}
