use std::path::PathBuf;

use std::collections::HashMap;

#[derive(Debug)]
pub struct Metadata(HashMap<String, serde_json::Value>);

impl Metadata {
    pub fn to_string(&self) -> serde_yaml::Result<String> {
        Ok(serde_yaml::to_string(&self.0)?)
    }
}

#[derive(Debug)]
pub struct Zettel {
    pub metadata: Metadata,
    pub local_path: PathBuf,
}
