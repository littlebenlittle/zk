mod command;
use command::{init, new, sync};
mod database;
mod zettel;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
struct Args {
    #[clap(subcommand)]
    cmd: Command,
}

type Database = database::yaml::Database;

#[derive(Debug, Subcommand)]
enum Command {
    Init(init::Args<Database>),
    New(new::Args<Database>),
    Sync(sync::Args<Database>)
}

#[derive(Debug)]
pub enum Error {
    DatabaseError(database::Error),
    NewCommandError(new::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::DatabaseError(e) => write!(f, "database error: {e}"),
            _ => write!(f, "unhandled error")
        }
    }
}

impl From<database::Error> for Error {
    fn from(e: database::Error) -> Self {
        Self::DatabaseError(e)
    }
}

impl From<new::Error> for Error {
    fn from(e: new::Error) -> Self {
        Self::NewCommandError(e)
    }
}

impl std::error::Error for Error {}
type Result<T> = std::result::Result<T, Error>;

fn main() -> Result<()> {
    let args = Args::parse();
    match args.cmd {
        Command::Init(a) => init::run(a)?,
        Command::New(a) => new::run(a)?,
        Command::Sync(a) => sync::run(a)?,
    }
    Ok(())
}
