use crate::{DateTime, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::prelude::*, path::PathBuf};

/// Metadata for a zettel
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Zettel {
    pub created: DateTime,
    pub modified: DateTime,
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
    pub default_frontmatter: serde_yaml::Mapping,
    pub zettels: HashMap<String, Zettel>,
}

#[derive(Debug)]
pub enum Error {
    UnknownField
}

impl AsRef<Self> for Zettelkasten {
    fn as_ref<'a>(&'a self) -> &'a Self {
        return &self;
    }
}

impl Zettelkasten {
    pub fn new(meta: ZkMeta, default_frontmatter: serde_yaml::Mapping) -> Self {
        Self {
            meta,
            default_frontmatter,
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
        for (key, val) in &self.default_frontmatter {
            let key = key
                .as_str()
                .expect("default_frontmatter keys should be of type str");
            let val = val
                .as_str()
                .expect("default_frontmatter vals should be of type str");
            meta.insert(key.into(), self.parse_global(val, zettel)?.into());
        }
        let fm = format!("{}---\n\n", serde_yaml::to_string(&meta)?);
        Ok(fm)
    }

    pub fn rm(&self, zettel: impl AsRef<Zettel>) -> Result<()> {
        let zettel: &Zettel = zettel.as_ref();
        std::fs::remove_file(zettel.rel_path.clone())?;
        Ok(())
    }

    fn parse_global(&self, val: &str, zettel: &Zettel) -> Result<String> {
        if ! val.starts_with("@") {
            return Ok(val.to_owned())
        }
        match &val[1..] {
            "title" => Ok(zettel.title.to_owned()),
            "id" => Ok(zettel.id.to_owned()),
            "created" => Ok(zettel.created.format("%Y-%m-%d").to_string()),
            _ => Err(Error::UnknownField.into())
        }
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
