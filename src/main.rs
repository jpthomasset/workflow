use std::process::ExitCode;

use clap::Parser;

use workflow::{
    cli::{WfArgs, WfCommands, WfTestCommands},
    command,
    config::{Config, RepoConfig},
    errors::WfError,
    git::{GitRepository, LocalGitRepository},
};

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{}", e);
            ExitCode::FAILURE
        }
    }
}

fn load_config(auto_init: bool) -> Result<Config, WfError> {
    let config = Config::load()?;

    if config.is_not_set() && auto_init {
        print!("Configuration is not set, starting initialization");
        command::command_init_config(config)
    } else {
        Ok(config)
    }
}

fn load_repo_config(repo: &impl GitRepository) -> Result<RepoConfig, WfError> {
    let path = repo.workdir().ok_or(WfError::NoGitWorkingDirectory)?;
    let config = RepoConfig::load(path)?;

    if config.is_not_set() {
        print!("Repository configuration is not set, starting initialization");
        command::command_init_repo(config, repo)
    } else {
        Ok(config)
    }
}

async fn run() -> Result<(), WfError> {
    let args = WfArgs::try_parse()?;
    let auto_init: bool = args.command != WfCommands::Init;
    let config = load_config(auto_init)?;

    match args.command {
        WfCommands::Init => {
            command::command_init_config(config)?;
            if let Ok(repo) = LocalGitRepository::discover() {
                let repo_config = load_repo_config(&repo).unwrap_or(RepoConfig::default());
                command::command_init_repo(repo_config, &repo)?;
            }
        }

        WfCommands::Test(test_arg) => match test_arg {
            WfTestCommands::Sub1 => println!("sub 1"),
            WfTestCommands::Sub2 => println!("sub 2"),
            WfTestCommands::All => println!("All"),
        },

        WfCommands::Start { ticket_id } => {
            let repo = LocalGitRepository::discover()?;
            let repo_config = load_repo_config(&repo)?;
            command::command_start(&config, &repo_config, &repo, &ticket_id).await?;
        }

        WfCommands::Push => {
            LocalGitRepository::discover()?.push()?;
        }

        WfCommands::Noop => {
            println!("Doing nothing");
        }
    }
    Ok(())
}
