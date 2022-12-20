use crate::{frontmatter, DateTime};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type Id = String;

#[derive(Debug)]
pub enum Error {
    UnknownField,
    FrontmatterError(frontmatter::Error),
}

impl From<frontmatter::Error> for Error {
    fn from(e: frontmatter::Error) -> Self {
        Self::FrontmatterError(e)
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FrontmatterError(e) => e.fmt(f),
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
            fm.insert(key.to_owned(), new_val);
        }
        Ok(format!(
            "{}\n---{}\n",
            frontmatter::write_str(&fm)?,
            self.content
        ))
    }
}
