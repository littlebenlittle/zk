mod database;
mod frontmatter;
mod zettel;
mod zettelkasten;

pub(crate) use zettel::ZettelMeta;

use std::fs::File;
use std::io::BufReader;
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
    let mut db = database::yaml::Database::new(args.root_dir)?;
    let mut zk = db.get_zk()?;
    match args.cmd {
        Command::Init => db.commit(&zk)?,
        Command::New(args) => {
            use rand::Rng;
            let id: String = rand::thread_rng()
                .sample_iter(&rand::distributions::Alphanumeric)
                .take(18)
                .map(char::from)
                .collect();
            let now = chrono::Local::now();
            let zettel = db.new_zettel(&args.title, &id, now)?;
            zk.add(&zettel)?;
            db.commit(&zk).or_else(|e| {
                println!("couldn't commit to database: {}", e);
                std::fs::remove_file(zettel.meta.path)
            })?;
        }
        Command::Sync => {
            // TODO: update db metata for zettel if it's
            // been updated in the file
            let curdir = std::env::current_dir()?;
            let dir_entries = std::fs::read_dir(curdir)?;
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
                let file = File::open(&path)?;
                let mut buf_reader = BufReader::new(file);
                let fm = match frontmatter::parse_yaml(&mut buf_reader) {
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
                        )
                    }
                    let id = id.unwrap().as_str();
                    if id.is_none() {
                        println!(
                            "skipping {} due to 'id' in frontmatter not being a 'string'",
                            path.to_str().unwrap()
                        )
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
            }
            db.commit(&zk)?;
        }
    }
    Ok(())
}
