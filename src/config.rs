use std::path::Path;

use confy::ConfyError;
use inquire::InquireError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::adapt_err::Adapt;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub jira: Option<JiraConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JiraConfig {
    pub host: String,
    pub user: String,
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RepoConfig {
    pub branches: Option<BranchsName>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BranchsName {
    pub dev: String,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Configuration error {0}")]
    ConfyError(#[from] ConfyError),
    #[error("Input error {0}")]
    InquireError(#[from] InquireError),
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        confy::load(env!("CARGO_PKG_NAME"), None).adapt()
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        confy::store(env!("CARGO_PKG_NAME"), None, self).adapt()
    }

    pub fn is_set(&self) -> bool {
        self.jira.is_some()
    }

    pub fn is_not_set(&self) -> bool {
        !self.is_set()
    }
}

impl RepoConfig {
    pub fn load(repo_workdir: &Path) -> Result<Self, ConfigError> {
        let path = repo_workdir.join(".workflow");
        confy::load_path(path).adapt()
    }

    pub fn save(&self, repo_workdir: &Path) -> Result<(), ConfigError> {
        let path = repo_workdir.join(".workflow");
        confy::store_path(path, self).adapt()
    }

    pub fn is_set(&self) -> bool {
        self.branches.is_some()
    }

    pub fn is_not_set(&self) -> bool {
        !self.is_set()
    }
}
