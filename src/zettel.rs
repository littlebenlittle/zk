use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub created: String,
    pub modified: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Zettel {
    pub metadata: Metadata,
    pub local_path: PathBuf,
}
