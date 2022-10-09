use crate::{DateTime, Result, Error};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io::prelude::*,
    path::{Path, PathBuf},
};

type Id = String;

/// Metadata for a zettel
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Zettel {
    pub created: DateTime,
    pub modified: DateTime,
    pub title: String,
    /// relative path to file from directory containing _zettel
    pub path: String,
    #[serde(skip)] // stored in Zettelkasten.zettels
    pub id: Id,
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
    // TODO: should be BTreeMap
    pub zettels: HashMap<String, Zettel>,
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
        let path = Path::new(&zettel.path);
        if path.exists() {
            return Err(std::io::ErrorKind::AlreadyExists.into());
        }
        let mut file = File::create(&path)?;
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
        std::fs::remove_file(Path::new(&zettel.path))?;
        Ok(())
    }

    fn parse_global(&self, val: &str, zettel: &Zettel) -> Result<String> {
        if !val.starts_with("@") {
            return Ok(val.to_owned());
        }
        match &val[1..] {
            "title" => Ok(zettel.title.to_owned()),
            "id" => Ok(zettel.id.to_owned()),
            "created" => Ok(zettel.created.format("%Y-%m-%d").to_string()),
            _ => Err(Error::UnknownField),
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

pub fn parse_meta_yaml(path: &PathBuf) -> Result<Option<serde_yaml::Mapping>> {
    use std::io::{BufReader};
    let file = std::fs::File::open(path)?;
    let mut lines = BufReader::new(file).lines().peekable();
    if !lines.next().unwrap()?.eq("---") {
        println!(
            "file {} doesn't appear to contain frontmatter; first line should be '---'",
            path.to_str().unwrap()
        );
        return Ok(None)
    }
    let mut frontmatter = String::new();
    while lines.peek().is_some() {
        let line: String = lines.next().unwrap()?;
        if line.eq("---") {
            break;
        }
        frontmatter.push_str(&line);
        frontmatter.push_str("\n");
    }
    match serde_yaml::from_str(&frontmatter) {
        Ok(fm) => Ok(fm),
        Err(e) => Err(e.into())
    }
}
