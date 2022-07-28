mod database;
mod zettel;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

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
    IoError(std::io::Error),
    SerializationError(serde_yaml::Error),
    ZettelError(zettel::Error),
    ChronoParseError(chrono::ParseError),
    DirNotEmpty,
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

impl From<serde_yaml::Error> for Error {
    fn from(e: serde_yaml::Error) -> Self {
        Self::SerializationError(e)
    }
}

impl From<zettel::Error> for Error {
    fn from(e: zettel::Error) -> Self {
        Self::ZettelError(e)
    }
}

impl From<chrono::ParseError> for Error {
    fn from(e: chrono::ParseError) -> Self {
        Self::ChronoParseError(e)
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("database error")
    }
}

type Result<T> = std::result::Result<T, Error>;
type DateTime = chrono::DateTime<chrono::Local>;

fn main() -> Result<()> {
    let args = Args::parse();
    let mut db = database::yaml::Database::new(args.root_dir)?;
    let mut zk = db.get_zk()?;
    match args.cmd {
        Command::Init => {}
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
            match db.commit(&zk) {
                Ok(()) => {}
                Err(e) => {
                    zk.rm(zettel)?;
                    return Err(e.into());
                }
            }
        }
        Command::Sync => unimplemented!(),
    }
    db.commit(zk)?;
    Ok(())
}
