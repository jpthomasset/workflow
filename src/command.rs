use crate::{
    config::{Config, RepoConfig},
    errors::WfError,
    git::{to_branch_name, GitRepository},
    init::{self, init_repo_config},
    jira::JiraServer,
};

use inquire::{Confirm, Text};

pub fn command_init_config(config: Config) -> Result<Config, WfError> {
    if config.is_set() &&
        Confirm::new("Warning, your configuration is already defined, do you want to continue and overwrite it?")
            .with_default(false)
            .prompt()? {
        return Ok(config);
    }

    let new_config = init::init_config(Some(&config))?;
    config.save()?;
    Ok(new_config)
}

pub fn command_init_repo(
    repo_config: RepoConfig,
    repo: &impl GitRepository,
) -> Result<RepoConfig, WfError> {
    if repo_config.is_set() &&
        Confirm::new("Warning, your repository configuration is already defined, do you want to continue and overwrite it?")
            .with_default(false)
            .prompt()? {
        return Ok(repo_config);
    }

    let branches = repo.branches()?;

    let new_config = init_repo_config(Some(&repo_config), &branches)?;
    match repo.workdir() {
        Some(path) => {
            new_config.save(path)?;
            Ok(new_config)
        }
        None => Err(WfError::NoGitWorkingDirectory),
    }
}

pub async fn command_start(
    config: &Config,
    repo_config: &RepoConfig,
    repo: &impl GitRepository,
    ticket_id: &str,
) -> Result<(), WfError> {
    let jira = config
        .jira
        .as_ref()
        .ok_or(WfError::ConfigurationNotSet)
        .and_then(|c| JiraServer::try_from(c).map_err(WfError::from))?;

    let dev_branch_name = repo_config
        .branches
        .as_ref()
        .ok_or(WfError::ConfigurationNotSet)?
        .dev
        .as_ref();

    let issue = jira.get_issue(ticket_id).await?;
    let default_branch_name = format!("{}-{}", issue.key, to_branch_name(&issue.summary));

    println!("Found issue {}: {}", issue.key, issue.summary);
    let new_branch_name = Text::new("Branch name:")
        .with_help_message("You can change the default branch name here.")
        .with_initial_value(&default_branch_name)
        .prompt()?;

    repo.create_and_checkout_branch(&new_branch_name, dev_branch_name)?;
    println!(
        "Branch {} created from {} with issue {}",
        new_branch_name, dev_branch_name, ticket_id
    );

    Ok(())
}
