mod database;
mod frontmatter;
mod zettel;
mod zettelkasten;

pub(crate) use zettel::ZettelMeta;
use zettelkasten::Zettelkasten;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

type DateTime = chrono::DateTime<chrono::Local>;

#[derive(Debug, Parser)]
struct Args {
    #[clap(default_value = ".", long)]
    root_dir: PathBuf,
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Initialize a new database
    Init,
    /// Create a new zettel
    New(NewArgs),
    /// Sync changes to zettels with the database
    Sync,
}

#[derive(Debug, clap::Args)]
pub struct NewArgs {
    pub title: String,
}

#[derive(Debug)]
pub enum Error {
    YamlDatabaseError(database::yaml::Error),
    ZettelError(zettel::Error),
    ZettelkastenError(zettelkasten::Error),
    IoError(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<database::yaml::Error> for Error {
    fn from(e: database::yaml::Error) -> Self {
        Self::YamlDatabaseError(e)
    }
}

impl From<zettel::Error> for Error {
    fn from(e: zettel::Error) -> Self {
        Self::ZettelError(e)
    }
}

impl From<zettelkasten::Error> for Error {
    fn from(e: zettelkasten::Error) -> Self {
        Self::ZettelkastenError(e)
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => e.fmt(f),
            Self::YamlDatabaseError(e) => e.fmt(f),
            Self::ZettelError(e) => e.fmt(f),
            Self::ZettelkastenError(e) => e.fmt(f),
        }
    }
}

type Result = std::result::Result<(), Error>;

fn main() -> Result {
    let args = Args::parse();
    let db = database::yaml::Database::new(args.root_dir)?;
    match args.cmd {
        Command::Init => {
            let zk = Zettelkasten::default();
            db.commit(zk)?;
        }
        Command::New(args) => new(db, args.title, chrono::Local::now())?,
        Command::Sync => sync(db)?,
    }
    Ok(())
}

fn new(db: database::yaml::Database, title: String, date: DateTime) -> Result {
    let mut zk = match db.get_zk()? {
        Some(zk) => zk,
        None => {
            if dialoguer::Confirm::new()
                .with_prompt("Database does not exist. Create it?")
                .interact()?
            {
                Default::default()
            } else {
                return Ok(())
            }
        }
    };
    use rand::Rng;
    let id: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(18)
        .map(char::from)
        .collect();
    let zettel = db.new_zettel(&title, &id, date)?;
    zk.add(&zettel)?;
    db.commit(&zk).or_else(|e| {
        println!("couldn't commit to database: {}", e);
        std::fs::remove_file(&zettel.meta.path)
    })?;
    Ok(())
}

fn sync(db: database::yaml::Database) -> Result {
    let mut zk = match db.get_zk()? {
        Some(zk) => zk,
        None => {
            println!("Database does not exist. Use `init` first.");
            return Ok(())
        }
    };
    let dir_entries = std::fs::read_dir(db.root_dir())?;
    for entry in dir_entries {
        let entry: std::fs::DirEntry = entry.unwrap();
        let path = entry.path();
        if path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned()
            .starts_with("_zettel")
        {
            continue;
        }
        let fm = match frontmatter::parse_yaml_path(&path) {
            Ok(meta) => meta,
            Err(e) => {
                println!(
                    "skipping {} due to frontmatter error: {}",
                    path.to_str().unwrap(),
                    e
                );
                continue;
            }
        };
        let id: zettel::Id = {
            let id = fm.get(&"id".into());
            if id.is_none() {
                println!(
                    "skipping {} due to missing key 'id' in frontmatter",
                    path.to_str().unwrap()
                );
                continue;
            }
            let id = id.unwrap().as_str();
            if id.is_none() {
                println!(
                    "skipping {} due to 'id' in frontmatter not being a 'string'",
                    path.to_str().unwrap()
                );
                continue;
            }
            id.unwrap().to_owned()
        };
        let current_meta = zk.zettels.get_mut(&id);
        if current_meta.is_none() {
            println!(
                "no metadata with id {} for zettel at {}; skipping",
                id,
                path.to_str().unwrap(),
            );
            continue;
        }
        let current_meta = current_meta.unwrap();
        current_meta.path = path
            .strip_prefix(&db.root_dir())
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
        if let Some(title) = fm.get(&"title".into()).and_then(|t| t.as_str()) {
            current_meta.title = title.to_owned()
        }
    }
    Ok(db.commit(&zk)?)
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::prelude::*;

    #[test]
    fn create_and_sync() -> Result {
        let tmp_dir = tempdir::TempDir::new("zk_command_test").expect("couldn't create temp dir");
        let dir_path = tmp_dir.path().to_path_buf();
        let db = database::yaml::Database::new(dir_path.clone())?;
        let zk = Zettelkasten::default();
        db.commit(zk)?;
        let dt = chrono::Local.timestamp(1431648000, 0);
        super::new(db, "my blog post".to_owned(), dt)?;
        let mut zettel_path = dir_path.clone();
        let dt_str = dt.format("%Y-%m-%d-my-blog-post.md").to_string();
        zettel_path.push(dt_str);
        assert!(zettel_path.exists());
        let meta = frontmatter::parse_yaml_path(&zettel_path).unwrap();
        let id = meta.get(&"id".into()).unwrap();
        let title = meta.get(&"title".into()).unwrap();
        let date = meta.get(&"date".into()).unwrap();
        let mut new_zettel_path = dir_path.clone();
        new_zettel_path.push(dt.format("new_path.md").to_string());
        std::fs::hard_link(&zettel_path, &new_zettel_path)?;
        std::fs::remove_file(&zettel_path)?;
        let db = database::yaml::Database::new(dir_path.clone())?;
        super::sync(db)?;
        let meta = frontmatter::parse_yaml_path(&new_zettel_path).unwrap();
        assert_eq!(meta.get(&"id".into()), Some(id));
        assert_eq!(meta.get(&"title".into()), Some(title));
        assert_eq!(meta.get(&"date".into()), Some(date));
        Ok(())
    }
}
