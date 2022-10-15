use std::io::prelude::*;
use std::io::BufReader;
use std::collections::HashMap;

#[derive(Debug)]
pub enum Error {
    MissingInitialDelimiter,
    MissingFinalDelimiter,
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
            Self::MissingInitialDelimiter => f.write_str("missing initial delimiter ---"),
            Self::MissingFinalDelimiter => f.write_str("missing final delimiter ---"),
            Self::IoError(e) => e.fmt(f),
            Self::SerializationError(e) => e.fmt(f),
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

pub fn parse_yaml<T: Read>(buf_reader: &mut BufReader<T>) -> Result<serde_yaml::Mapping> {
    let mut lines = buf_reader.lines().peekable();
    if !lines.next().unwrap()?.eq("---") {
        return Err(Error::MissingInitialDelimiter);
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
            return Err(Error::MissingFinalDelimiter);
        }
    }
    Ok(serde_yaml::from_str(&frontmatter)?)
}

pub fn write_str(frontmatter: &HashMap<String, String>) -> Result<String> {
    Ok(serde_yaml::to_string(frontmatter)?)
}
