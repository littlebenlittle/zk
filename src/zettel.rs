use crate::DateTime;
use serde::{Deserialize, Serialize};
use std::io::BufReader;
use std::{collections::HashMap, fs::File, io::prelude::*, path::Path};

pub type Id = String;

#[derive(Debug)]
pub enum Error {
    UnknownField,
    MissingFrontmatterDelimiter,
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
        match self {
            Self::MissingFrontmatterDelimiter => f.write_str("missing frontmatter delimiter ---"),
            Self::IoError(e) => e.fmt(f),
            Self::SerializationError(e) => e.fmt(f),
            Self::UnknownField => f.write_str("unknown field"),
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ZettelMeta {
    pub created: DateTime,
    pub modified: DateTime,
    pub title: String,
    /// relative path to file from directory containing _zettel
    pub path: String,
    #[serde(skip)] // stored in Zettelkasten.zettels
    pub id: Id,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Zettel {
    pub meta: ZettelMeta,
    pub content: String,
}

impl AsRef<Self> for ZettelMeta {
    fn as_ref<'a>(&'a self) -> &'a Self {
        return &self;
    }
}

impl AsRef<Self> for Zettel {
    fn as_ref<'a>(&'a self) -> &'a Self {
        return &self;
    }
}

impl Zettel {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        Self::from_buf_reader(BufReader::new(File::open(&path)?))
    }

    pub fn from_str(buf: &str) -> Result<Self> {
        Self::from_buf_reader(BufReader::new(buf.as_bytes()))
    }

    pub fn from_buf_reader<T: Read>(mut buf_reader: BufReader<T>) -> Result<Self> {
        let meta = ZettelMeta::parse_yaml(&mut buf_reader)?;
        let mut content = String::new();
        buf_reader.read_to_string(&mut content)?;
        Ok(Self { meta, content })
    }

    /// write zettel with frontmatter to string
    ///
    /// use '@key_name' to include metadata keys in fronmatter
    /// supported key names are 'title', 'id', 'created'
    pub fn as_string(&self, frontmatter: &HashMap<String, String>) -> Result<String> {
        let mut fm = HashMap::new();
        for (key, val) in frontmatter {
            let new_val = if !val.starts_with("@") {
                val.to_owned()
            } else {
                match &val[1..] {
                    "title" => self.meta.title.clone(),
                    "id" => self.meta.id.clone(),
                    "created" => self.meta.created.format("%Y-%m-%d").to_string(),
                    _ => return Err(Error::UnknownField),
                }
            };
            fm.insert(key, new_val);
        }
        Ok(format!(
            "{}\n---{}\n",
            serde_yaml::to_string(&fm)?,
            self.content
        ))
    }
}

impl ZettelMeta {
    pub fn parse_yaml<T: Read>(buf_reader: &mut BufReader<T>) -> Result<Self> {
        let mut lines = buf_reader.lines().peekable();
        if !lines.next().unwrap()?.eq("---") {
            return Err(Error::MissingFrontmatterDelimiter);
        }
        let mut frontmatter = String::new();
        loop {
            if let Some(line) = lines.next() {
                let line = line?;
                if line.eq("---") {
                    break;
                }
                frontmatter.push_str(&line);
                frontmatter.push_str("\n");
            } else {
                return Err(Error::MissingFrontmatterDelimiter);
            }
        }
        Ok(serde_yaml::from_str(&frontmatter)?)
    }
}
