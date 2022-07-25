pub mod yaml;

use std::str::FromStr;

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    SerializationError(serde_yaml::Error),
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

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("database error")
    }
}

type Result<T> = std::result::Result<T, Error>;

pub trait Database: std::fmt::Debug {
    type Config: FromStr<Err = Error> + AsRef<Self::Config> + Clone;

    /// create a new database with the given config
    fn from_config(cfg: impl AsRef<Self::Config>) -> Self;

    /// initialize the database
    fn init(&mut self) -> Result<()>;

    /// commit any changes to the database
    fn commit(&mut self) -> Result<()>;

    /// create a new zettel with the given title. Returns
    /// the path
    fn new_zettel(&mut self, title: String) -> Result<()>;
}
