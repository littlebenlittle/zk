use crate::zettel;
use crate::{zettel::Zettel, DateTime, ZettelMeta};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::prelude::*, path::Path};

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    SerializationError(serde_yaml::Error),
    ZettelError(zettel::Error),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => e.fmt(f),
            Self::ZettelError(e) => e.fmt(f),
            Self::SerializationError(e) => e.fmt(f),
        }
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

type Result<T> = std::result::Result<T, Error>;

/// Store of zettels on the filesystem
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Zettelkasten {
    pub meta: ZkMeta,
    pub default_frontmatter: HashMap<String, String>,
    // TODO: should be BTreeMap because ID is already totally ordered
    pub zettels: HashMap<zettel::Id, ZettelMeta>,
}

impl AsRef<Self> for Zettelkasten {
    fn as_ref<'a>(&'a self) -> &'a Self {
        return &self;
    }
}

impl Zettelkasten {
    pub fn new(meta: ZkMeta, default_frontmatter: HashMap<String, String>) -> Self {
        Self {
            meta,
            default_frontmatter,
            zettels: HashMap::new(),
        }
    }

    pub fn add(&mut self, zettel: impl AsRef<Zettel>) -> Result<()> {
        let zettel = zettel.as_ref();
        let path = Path::new(&zettel.meta.path);
        if path.exists() {
            return Err(std::io::ErrorKind::AlreadyExists.into());
        }
        let mut file = File::create(&path)?;
        let zettel_str = zettel.as_string(&self.default_frontmatter)?;
        file.write_all(zettel_str.as_bytes())?;
        self.zettels
            .insert(zettel.meta.id.clone(), zettel.meta.clone());
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
