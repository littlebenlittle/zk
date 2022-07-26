use super::{Database as DbTrait, Error, Result};
use crate::zettel::{Metadata, Zettel};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::prelude::*, path::PathBuf, str::FromStr};
use uuid::Uuid;

#[derive(Debug)]
pub struct Database {
    root_dir: PathBuf,
    db_file: DbFile,
}

/// Data contained in the database.yaml file
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct DbFile {
    meta: DbMeta,
    zettels: HashMap<Uuid, Zettel>,
}

/// Metadata about the database
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct DbMeta {
    /// database creation time
    created: String,
    /// last modificiation time
    modified: String,
}

impl DbTrait for Database {
    type Config = Config;

    fn from_config(cfg: impl AsRef<Self::Config>) -> Result<Self> {
        let cfg = cfg.as_ref();
        let root_dir = cfg.root_dir.clone();
        let mut db_path = root_dir.clone();
        db_path.push("_zettel.yaml");
        let db_file = if db_path.is_file() {
            serde_yaml::from_reader(File::open(db_path)?)?
        } else {
            new_db_file()
        };
        Ok(Self { root_dir, db_file })
    }

    fn init(&mut self) -> Result<()> {
        // noop
        Ok(())
    }

    fn commit(&mut self) -> Result<()> {
        let mut path = self.root_dir.clone();
        path.push("_zettel.yaml");
        serde_yaml::to_writer(File::create(&path)?, &self.db_file)?;
        Ok(())
    }

    fn new_zettel(&mut self, title: &str) -> Result<Zettel> {
        let uuid = Uuid::new_v4();
        let meta = self.default_metadata(title, &uuid);
        let fm = format!("---\n{}---\n\n", serde_yaml::to_string(&meta)?);
        let now = Local::now().to_rfc3339();
        let zettel = Zettel {
            metadata: Metadata {
                created: now.clone(),
                modified: now.clone(),
            },
            local_path: self.make_filename(&title),
        };
        let mut file = File::create(&zettel.local_path)?;
        file.write_all(fm.as_bytes())?;
        self.db_file.zettels.insert(uuid, zettel.clone());
        Ok(zettel)
    }
}

impl Database {
    fn make_filename(&self, title: &str) -> PathBuf {
        let mod_title = title.replace(" ", "-");
        let mut path = self.root_dir.clone();
        let date_str = Local::now().format("%Y-%m-%d");
        let filename = format!("{date_str}-{mod_title}.md");
        path.push(filename);
        path
    }

    fn default_metadata(&self, title: &str, uuid: &Uuid) -> serde_yaml::Value {
        let mut meta = serde_yaml::Mapping::new();
        meta.insert("uuid".into(), (uuid.to_string()).into());
        meta.insert("title".into(), title.to_owned().into());
        meta.into()
    }
}

fn new_db_file() -> DbFile {
    let now = Local::now().to_rfc3339();
    DbFile {
        meta: DbMeta {
            created: now.clone(),
            modified: now,
        },
        zettels: HashMap::new(),
    }
}

#[derive(Debug, clap::Args, Clone)]
pub struct Config {
    root_dir: PathBuf,
}

impl FromStr for Config {
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let root_dir = PathBuf::from(s);
        // TODO validate dir
        Ok(Self { root_dir })
    }
}

impl AsRef<Self> for Config {
    fn as_ref<'a>(&'a self) -> &'a Self {
        return &self;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn init_db() {
        let tmp_dir = TempDir::new("zk_yaml_test").expect("couldn't create temp dir");
        let mut db = Database::from_config(Config {
            root_dir: PathBuf::from(tmp_dir.path()),
        })
        .expect("could not create db");
        unwrap_with(db.commit());
        let mut db_path = PathBuf::from(tmp_dir.path());
        db_path.push("_zettel.yaml");
        let file = File::open(db_path).expect("db file wasn't created");
        let read_db: DbFile = unwrap_with(serde_yaml::from_reader(file));
        assert_eq!(db.db_file, read_db)
    }

    #[test]
    fn new_zettel() {
        let tmp_dir = TempDir::new("zk_yaml_test").expect("couldn't create temp dir");
        let cfg = Config {
            root_dir: PathBuf::from(tmp_dir.path()),
        };
        let mut db = unwrap_with(Database::from_config(&cfg));
        let zettel = unwrap_with(db.new_zettel("a new blog post"));
        assert!(zettel.local_path.exists(), "new zettel was not created on fs");
        unwrap_with(db.commit());
        let new_db = unwrap_with(Database::from_config(cfg));
        assert!(
            new_db.db_file.zettels.len() == 1,
            "new zettel should be reflected in db"
        );
    }

    #[inline]
    fn unwrap_with<T, E: std::error::Error>(x: std::result::Result<T, E>) -> T {
        match x {
            Ok(v) => v,
            Err(e) => panic!("{e}"),
        }
    }
}
