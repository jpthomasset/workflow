use std::cmp::Ordering;

use inquire::{
    min_length, required,
    validator::{StringValidator, Validation},
    Select, Text,
};
use url::Url;

use crate::config::{BranchsName, Config, ConfigError, JiraConfig, RepoConfig};

#[derive(Clone, Default)]
pub struct UrlValidator {}

impl StringValidator for UrlValidator {
    fn validate(&self, input: &str) -> Result<Validation, inquire::CustomUserError> {
        match Url::parse(input) {
            Ok(_) => Ok(Validation::Valid),
            Err(e) => Ok(Validation::Invalid(e.into())),
        }
    }
}

pub fn init_config(old_config: Option<&Config>) -> Result<Config, ConfigError> {
    let banner = format!(
        "
    ██╗    ██╗ ██████╗ ██████╗ ██╗  ██╗███████╗██╗      ██████╗ ██╗    ██╗
    ██║    ██║██╔═══██╗██╔══██╗██║ ██╔╝██╔════╝██║     ██╔═══██╗██║    ██║
    ██║ █╗ ██║██║   ██║██████╔╝█████╔╝ █████╗  ██║     ██║   ██║██║ █╗ ██║
    ██║███╗██║██║   ██║██╔══██╗██╔═██╗ ██╔══╝  ██║     ██║   ██║██║███╗██║
    ╚███╔███╔╝╚██████╔╝██║  ██║██║  ██╗██║     ███████╗╚██████╔╝╚███╔███╔╝
     ╚══╝╚══╝  ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝     ╚══════╝ ╚═════╝  ╚══╝╚══╝ v{}

                           <<<< Initialisation >>>>
     ",
        env!("CARGO_PKG_VERSION")
    );
    println!("{}", banner);

    let jira_config = old_config.and_then(|c| c.jira.as_ref());

    let old_host = jira_config.map(|j| j.host.as_ref()).unwrap_or_default();

    let host = Text::new("What's the url of your Jira instance?")
        .with_validator(required!())
        .with_validator(UrlValidator::default())
        .with_default(old_host)
        .prompt()?;

    let old_user = jira_config.map(|j| j.user.as_ref()).unwrap_or_default();

    let user = Text::new("What's your username?")
        .with_validator(required!())
        .with_validator(min_length!(3))
        .with_default(old_user)
        .prompt()?;

    println!("A token is required to authenticate you on Jira. You can create a token from https://id.atlassian.com/manage-profile/security/api-tokens");

    let token = Text::new("What's your token?")
        .with_validator(required!())
        .with_validator(min_length!(3))
        .prompt()?;

    Ok(Config {
        jira: Some(JiraConfig { host, user, token }),
    })
}

pub fn init_repo_config(
    _old_config: Option<&RepoConfig>,
    branches: &[String],
) -> Result<RepoConfig, ConfigError> {
    // let branches_config = old_config.and_then(|c| c.branches.as_ref());

    let sorted_branches = sort_branches(branches);

    let dev = Select::new(
        "What branch do you want to use as base branch for features?",
        sorted_branches,
    )
    .prompt()?;

    Ok(RepoConfig {
        branches: Some(BranchsName { dev }),
    })
}

fn sort_branches<T: AsRef<str>>(branches: &[T]) -> Vec<String> {
    let mut sorted_branches = Vec::from_iter(branches.iter().map(|t| String::from(t.as_ref())));

    sorted_branches.sort_by(|a, b| -> Ordering {
        match (a.as_str(), b.as_str()) {
            ("develop", "main" | "master") => Ordering::Less,
            ("main" | "master", "develop") => Ordering::Greater,
            ("master" | "main" | "develop", _) => Ordering::Less,
            (_, "master" | "main" | "develop") => Ordering::Greater,
            (_, _) => a.cmp(b),
        }
    });

    sorted_branches
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    #[test]
    fn test_sort_branch() {
        let branches = vec!["d", "b", "a", "c"];
        let result = sort_branches(&branches);

        assert_eq!(result, vec!["a", "b", "c", "d"]);
    }

    #[test]
    fn test_sort_branch_master_first() {
        let branches = vec!["d", "master", "a", "c"];
        let result = sort_branches(&branches);

        assert_eq!(result, vec!["master", "a", "c", "d"]);
    }

    #[test]
    fn test_sort_branch_main_first() {
        let branches = vec!["d", "main", "a", "c"];
        let result = sort_branches(&branches);

        assert_eq!(result, vec!["main", "a", "c", "d"]);
    }

    #[test]
    fn test_sort_branch_dev_main_first() {
        let branches = vec!["main", "develop", "a", "c"];
        let result = sort_branches(&branches);

        assert_eq!(result, vec!["develop", "main", "a", "c"]);
    }
}
