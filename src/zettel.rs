use crate::frontmatter;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

pub type Id = String;
type DateTime = chrono::DateTime<chrono::Local>;

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

impl ZettelMeta {
    pub fn path(&self) -> &str {
        &self.path
    }
}

impl AsRef<Self> for Zettel {
    fn as_ref<'a>(&'a self) -> &'a Self {
        return &self;
    }
}

impl Zettel {
    pub fn builder() -> ZettelBuilder {
        Default::default()
    }

    pub fn uuid(&self) -> &Id {
        &self.meta.id
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
            fm.insert(key.to_owned(), new_val);
        }
        Ok(format!(
            "{}\n---{}\n",
            frontmatter::write_str(&fm)?,
            self.content
        ))
    }
}

#[derive(Clone)]
pub struct ZettelBuilder {
    title: Option<String>,
    created: Option<DateTime>,
    modified: Option<DateTime>,
    subdir: Option<String>,
    content: Option<String>,
    uuid: Option<Id>,
}

impl Default for ZettelBuilder {
    fn default() -> Self {
        Self {
            title: None,
            created: None,
            modified: None,
            subdir: None,
            content: None,
            uuid: None,
        }
    }
}

impl ZettelBuilder {
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn created(mut self, created: impl Into<DateTime>) -> Self {
        self.created = Some(created.into());
        self
    }

    pub fn subdir(mut self, subdir: impl Into<String>) -> Self {
        self.subdir = Some(subdir.into());
        self
    }

    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    pub fn build(self) -> Zettel {
        Zettel {
            meta: {
                let created = self.created.unwrap_or(chrono::Local::now());
                let title = self.title.unwrap_or("my zettel".into());
                ZettelMeta {
                    created,
                    modified: self.modified.unwrap_or(created),
                    path: {
                        let mut path = PathBuf::from(self.subdir.unwrap_or("".into()));
                        path.push(make_filename(&title));
                        path.as_os_str().to_str().unwrap().into()
                    },
                    title,
                    id: self.uuid.unwrap_or(rand_id()),
                }
            },
            content: self.content.unwrap_or("\n".into()),
        }
    }
}

fn rand_id() -> Id {
    use rand::Rng;
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(18)
        .map(char::from)
        .collect()
}

fn make_filename(title: &str) -> String {
    format!("{}.md", title.replace(" ", "-"))
}
