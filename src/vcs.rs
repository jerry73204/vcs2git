use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReposFile {
    pub repositories: IndexMap<PathBuf, Repo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repo {
    pub r#type: RepoType,
    pub url: Url,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RepoType {
    Git,
    #[serde(untagged)]
    Unknown(String),
}
