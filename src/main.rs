use std::process::ExitCode;

use clap::{Parser, Subcommand};
use confy::ConfyError;
use inquire::{Confirm, InquireError};
use thiserror::Error;
use workflow::{
    config::{Config, ConfigError},
    git::{to_branch_name, GitError, GitRepository},
    jira::{JiraError, JiraServer},
};

#[derive(Debug, Parser)]
#[command(name = "wf")]
#[command(about = "A tool to automate some common dev tasks", long_about = None)]
struct WfArgs {
    #[command(subcommand)]
    command: WfCommands,
}

#[derive(Debug, Subcommand, PartialEq, Eq)]
enum WfCommands {
    /// Initialize workflow app settings
    Init,
    /// Start a new workflow in the current repository
    Start {
        #[arg(required = true, help = "Ticket id to create the branch from")]
        ticket_id: String,
    },
    /// Push current work branch to remote repository
    Push,
    /// Do nothing, just to test
    Noop,
}

#[derive(Debug, Error)]
pub enum WfError {
    #[error("Configuration is not set, please run init command first")]
    ConfigurationNotSet,
    #[error("{0}")]
    InitializationError(ConfigError),
    #[error("Configuration cannot be loaded: {0}")]
    CannotLoadConfiguration(ConfyError),
    #[error("Configuration cannot be saved: {0}")]
    CannotSaveConfiguration(ConfyError),
    #[error("Jira error: {0}")]
    JiraError(JiraError),
    #[error("Git error: {0}")]
    GitError(GitError),
    #[error("Input error: {0}")]
    InquireError(InquireError),
    #[error("Configuration error: {0}")]
    ConfigError(ConfigError),
}

impl From<JiraError> for WfError {
    fn from(value: JiraError) -> Self {
        WfError::JiraError(value)
    }
}

impl From<GitError> for WfError {
    fn from(value: GitError) -> Self {
        WfError::GitError(value)
    }
}

impl From<ConfigError> for WfError {
    fn from(value: ConfigError) -> Self {
        WfError::ConfigError(value)
    }
}

impl From<InquireError> for WfError {
    fn from(value: InquireError) -> Self {
        WfError::InquireError(value)
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            println!("{}", e);
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), WfError> {
    let args = WfArgs::parse();
    let config = Config::load()?;

    if config.is_not_set() && args.command != WfCommands::Init {
        return Err(WfError::ConfigurationNotSet);
    }

    match args.command {
        WfCommands::Init => {
            if config.is_set() && !Confirm::new("Warning, your configuration is already defined, do you want to continue and overwrite it?")
                    .with_default(false)
                    .prompt()? {
                return Ok(());
            }
            let config = Config::init(Some(&config)).map_err(WfError::InitializationError)?;
            config.save()?;
        }

        WfCommands::Start { ticket_id } => {
            let jira = config
                .jira
                .as_ref()
                .ok_or(WfError::ConfigurationNotSet)
                .and_then(|c| JiraServer::try_from(c).map_err(WfError::from))?;

            let issue = jira.get_issue(&ticket_id).await?;
            println!("Issue: {:#?}", issue);
            let branch_name = format!("{}-{}", issue.key, to_branch_name(&issue.summary));
            GitRepository::discover()?.create_and_checkout_branch(&branch_name, "develop")?;
            println!("Branch {} created from issue {}", branch_name, ticket_id);
        }

        WfCommands::Push => {
            println!("Pushing");
            GitRepository::discover()?.push()?;
        }

        WfCommands::Noop => {
            println!("Doing nothing");
        }
    }
    Ok(())
}
