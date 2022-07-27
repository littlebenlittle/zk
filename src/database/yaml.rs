use crate::{
    zettel::{Zettel, Zettelkasten},
    Result,
};
use chrono::prelude::*;
use std::{fs::File, path::PathBuf};

#[derive(Debug)]
pub struct Database {
    root_dir: PathBuf,
}

impl Database {
    pub fn new(root_dir: PathBuf) -> Result<Self> {
        Ok(Self { root_dir })
    }

    pub fn get_zk(&mut self) -> Result<Zettelkasten> {
        let mut path = self.root_dir.clone();
        path.push("_zettel.yaml");
        let zk = if path.is_file() {
            let file = File::open(path)?;
            serde_yaml::from_reader(file)?
        } else {
            Zettelkasten::new()
        };
        return Ok(zk);
    }

    fn make_filename(&self, title: &str) -> PathBuf {
        let mod_title = title.replace(" ", "-");
        let mut path = self.root_dir.clone();
        let date_str = Local::now().format("%Y-%m-%d");
        let filename = format!("{date_str}-{mod_title}.md");
        path.push(filename);
        path
    }

    pub fn commit(&mut self, zk: impl AsRef<Zettelkasten>) -> Result<()> {
        let mut path = self.root_dir.clone();
        path.push("_zettel.yaml");
        serde_yaml::to_writer(File::create(&path)?, zk.as_ref())?;
        Ok(())
    }

    pub fn new_zettel(&mut self, title: impl AsRef<str>, id: impl AsRef<str>) -> Result<Zettel> {
        let now = Local::now().to_rfc3339();
        let rel_path = self.make_filename(title.as_ref());
        let filename = rel_path.file_name().unwrap().to_str().unwrap().to_owned();
        let zettel = Zettel {
            created: now.clone(),
            modified: now.clone(),
            id: id.as_ref().to_owned(),
            title: title.as_ref().to_owned(),
            filename,
            rel_path,
        };
        Ok(zettel)
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn init_db() {
        match run_init_db_test() {
            Ok(v) => v,
            Err(e) => panic!("{e}"),
        }
    }

    fn run_init_db_test() -> Result<()> {
        let tmp_dir = TempDir::new("zk_yaml_test").expect("couldn't create temp dir");
        let mut db = Database::new(PathBuf::from(tmp_dir.path())).expect("could not create db");
        let zk = db.get_zk()?;
        db.commit(&zk)?;
        let mut db_path = PathBuf::from(tmp_dir.path());
        db_path.push("_zettel.yaml");
        let file = File::open(db_path).expect("db file wasn't created");
        let read_db: Zettelkasten = serde_yaml::from_reader(file)?;
        assert_eq!(zk, read_db);
        Ok(())
    }

    #[test]
    fn new_zettel() {
        match run_new_zettel_test() {
            Ok(v) => v,
            Err(e) => panic!("{e}"),
        }
    }

    fn run_new_zettel_test() -> Result<()> {
        let tmp_dir = TempDir::new("zk_yaml_test").expect("couldn't create temp dir");
        let root_dir = PathBuf::from(tmp_dir.path());
        let mut db = Database::new(root_dir.clone())?;
        let mut zk = db.get_zk()?;
        let id = "123456";
        let zettel = db.new_zettel("a new blog post", id)?;
        zk.add(&zettel)?;
        assert!(zettel.rel_path.exists(), "new zettel was not created on fs");
        db.commit(zk)?;
        let new_zk = db.get_zk()?;
        assert!(
            new_zk.zettels.len() == 1,
            "new zettel should be reflected in db"
        );
        Ok(())
    }
}
