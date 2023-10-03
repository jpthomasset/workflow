use crate::{config::ConfigError, git::GitError, jira::JiraError};
use clap::Error;
use inquire::InquireError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WfError {
    #[error("Configuration is not set, please run init command first")]
    ConfigurationNotSet,
    #[error("Jira error: {0}")]
    JiraError(#[from] JiraError),
    #[error("Git error: {0}")]
    GitError(#[from] GitError),
    #[error("Input error: {0}")]
    InquireError(#[from] InquireError),
    #[error("Configuration error: {0}")]
    ConfigError(#[from] ConfigError),
    #[error("{0}")]
    CliArgsError(#[from] Error),
    #[error("Current repository has no working directory !?!")]
    NoGitWorkingDirectory,
}
