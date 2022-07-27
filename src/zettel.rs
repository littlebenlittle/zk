use crate::Result;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::prelude::*, path::PathBuf};

/// Metadata for a zettel
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Zettel {
    pub created: String,
    pub modified: String,
    pub title: String,
    pub filename: String,
    #[serde(skip)]
    pub id: String,
    #[serde(skip)]
    pub rel_path: PathBuf,
}

impl AsRef<Self> for Zettel {
    fn as_ref<'a>(&'a self) -> &'a Self {
        return &self;
    }
}

/// Store of zettels on the filesystem
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Zettelkasten {
    pub meta: ZkMeta,
    pub zettels: HashMap<String, Zettel>,
}

#[derive(Debug)]
enum Error {}

impl AsRef<Self> for Zettelkasten {
    fn as_ref<'a>(&'a self) -> &'a Self {
        return &self;
    }
}

impl Zettelkasten {
    pub fn new() -> Self {
        let now = chrono::Local::now().to_rfc3339();
        Self {
            meta: ZkMeta {
                created: now.clone(),
                modified: now,
            },
            zettels: HashMap::new(),
        }
    }

    pub fn add(&mut self, zettel: impl AsRef<Zettel>) -> Result<()> {
        let zettel = zettel.as_ref();
        if zettel.rel_path.exists() {
            return Err(std::io::ErrorKind::AlreadyExists.into());
        }
        let mut file = File::create(&zettel.rel_path)?;
        let fm = self.default_frontmatter(&zettel)?;
        file.write_all(fm.as_bytes())?;
        self.zettels.insert(zettel.id.clone(), zettel.clone());
        Ok(())
    }

    pub fn default_frontmatter(&self, zettel: &Zettel) -> Result<String> {
        let mut meta = serde_yaml::Mapping::new();
        meta.insert("id".into(), zettel.id.clone().into());
        meta.insert("title".into(), zettel.title.clone().into());
        let fm = format!("{}---\n\n", serde_yaml::to_string(&meta)?);
        Ok(fm)
    }

    pub fn rm(&self, zettel: impl AsRef<Zettel>) -> Result<()> {
        let zettel: &Zettel = zettel.as_ref();
        std::fs::remove_file(zettel.rel_path.clone())?;
        Ok(())
    }
}

/// Metadata about the database
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ZkMeta {
    /// database creation time
    created: String,
    /// last modificiation time
    modified: String,
}
