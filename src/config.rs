use confy::ConfyError;
use inquire::{
    min_length, required,
    validator::{StringValidator, Validation},
    InquireError, Text,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

use crate::adapt_err::AdaptErr;

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

#[derive(Clone, Default)]
pub struct UrlValidator {}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Configuration error {0}")]
    ConfyError(#[from] ConfyError),
    #[error("Input error {0}")]
    InquireError(#[from] InquireError),
}

impl StringValidator for UrlValidator {
    fn validate(&self, input: &str) -> Result<Validation, inquire::CustomUserError> {
        match Url::parse(input) {
            Ok(_) => Ok(Validation::Valid),
            Err(e) => Ok(Validation::Invalid(e.into())),
        }
    }
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

    pub fn init(old_config: Option<&Self>) -> Result<Self, ConfigError> {
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

        Ok(Self {
            jira: Some(JiraConfig { host, user, token }),
        })
    }
}
