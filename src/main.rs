mod frontmatter;
mod zettel;
mod zettelkasten;

pub(crate) use zettel::{Zettel, ZettelMeta};
use zettelkasten::Zettelkasten;

use chrono::prelude::*;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

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
    ZettelError(zettel::Error),
    ZettelkastenError(zettelkasten::Error),
    IoError(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
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
            // Self::YamlDatabaseError(e) => e.fmt(f),
            Self::ZettelError(e) => e.fmt(f),
            Self::ZettelkastenError(e) => e.fmt(f),
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

fn main() -> Result<()> {
    let args = Args::parse();
    if let Command::Init = args.cmd {
        Zettelkasten::builder().build()?.commit()?;
    }
    let mut zk = match Zettelkasten::open(args.root_dir)? {
        Some(zk) => zk,
        None => match confirm_db_creation()? {
            Some(zk) => zk,
            None => return Ok(()),
        },
    };
    match args.cmd {
        Command::New(new_args) => {
            zk.add(
                Zettel::builder()
                    .title(new_args.title)
                    .created(chrono::Local.timestamp(1431648000, 0))
                    .content("\n")
                    .build(),
            )?;
        }
        Command::Sync => zk.sync()?,
        _ => unreachable!(),
    }
    zk.commit()?;
    Ok(())
}

fn confirm_db_creation() -> Result<Option<Zettelkasten>> {
    if dialoguer::Confirm::new()
        .with_prompt("Database does not exist. Create it?")
        .interact()?
    {
        Ok(Some(Zettelkasten::builder().build()?))
    } else {
        Ok(None)
    }
}
