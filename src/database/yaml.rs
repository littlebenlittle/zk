use super::{Database as DbTrait, Error, Result};
use crate::zettel::{Zettel, Metadata};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug)]
pub struct Database {
    root_dir: PathBuf,
    zettels: Vec<Zettel>,
}

impl DbTrait for Database {
    type Config = Config;

    fn from_config(cfg: impl AsRef<Self::Config>) -> Self {
        let cfg = cfg.as_ref();
        Self {root_dir: cfg.root_dir, zettels: Vec::new()}
    }

    fn init(&mut self) -> Result<()> {
        unimplemented!()
    }

    fn commit(&mut self) -> Result<()> {
        // no op
        Ok(())
    }

    fn new_zettel(&mut self, title: String) -> Result<()> {
        let path = self.root_dir.clone();
        use std::io::prelude::*;
        let fm = format!("---\n{}---\n\n", self.default_metadata().to_string()?);
        let file = std::fs::File::create(self.make_filename(&title)?)?;
        file.write_all(fm.as_bytes());
        Ok(())
    }
}

impl Database {
    fn make_filename(&self, title: &str) -> Result<PathBuf> {}
    fn default_metadata(&self) -> Metadata {}
}

#[derive(Debug, clap::Args, Clone)]
pub struct Config {
    root_dir: PathBuf,
}

impl FromStr for Config {
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let root_dir = PathBuf::from(s);
        // TODO validate dir
        Ok(Self {
            root_dir,
        })
    }
}

impl AsRef<Self> for Config {
    fn as_ref<'a>(&'a self) -> &'a Self {
        return &self;
    }
}
