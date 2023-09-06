use std::fmt::Display;

use reqwest::Client;
use reqwest::Error as ReqwestError;
use reqwest::StatusCode;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::ParseError;

use crate::config::JiraConfig;

pub struct JiraServer {
    host: Url,
    credentials: JiraCredentials,
}

pub struct JiraCredentials {
    user: JiraUser,
    token: JiraToken,
}

pub struct JiraUser(String);
pub struct JiraToken(String);

#[derive(Debug)]
pub struct JiraIssue {
    pub id: String,
    pub key: String,
    pub summary: String,
    pub status: JiraStatus,
}

#[derive(Debug, Serialize, Deserialize)]
struct JiraRestIssue {
    id: String,
    key: String,
    fields: JiraRestFields,
}

#[derive(Debug, Serialize, Deserialize)]
struct JiraRestFields {
    summary: String,
    status: JiraRestStatus,
}

#[derive(Debug, Serialize, Deserialize)]
struct JiraRestStatus {
    id: String,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JiraStatus {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Error)]
pub enum JiraError {
    #[error("{0}")]
    RequestError(#[from] ReqwestError),
    #[error("Invalid server url {0}")]
    InvalidUrl(#[from] ParseError),
    #[error("Issue {0} not found")]
    IssueNotFound(String),
}

impl JiraUser {
    pub fn new(name: String) -> Self {
        Self(name)
    }
}

impl Display for JiraUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl JiraToken {
    pub fn new(token: String) -> Self {
        Self(token)
    }
}

impl Display for JiraToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl JiraCredentials {
    pub fn new(user: JiraUser, token: JiraToken) -> Self {
        Self { user, token }
    }
}

impl TryFrom<&JiraConfig> for JiraServer {
    type Error = JiraError;

    fn try_from(config: &JiraConfig) -> Result<Self, Self::Error> {
        let host = Url::parse(&config.host)?;
        let user = JiraUser(config.user.clone());
        let token = JiraToken(config.token.clone());
        Ok(JiraServer {
            host,
            credentials: JiraCredentials { user, token },
        })
    }
}

impl JiraServer {
    pub async fn get_issue(&self, key_or_id: &str) -> Result<JiraIssue, JiraError> {
        let url = self.host.join("/rest/api/2/issue/")?.join(key_or_id)?;

        let client = Client::builder().build()?;

        let response = client
            .get(url)
            .basic_auth(&self.credentials.user, Some(&self.credentials.token))
            .send()
            .await?;

        if response.status() == StatusCode::NOT_FOUND {
            return Err(JiraError::IssueNotFound(key_or_id.to_string()));
        }

        let rest_issue = response.error_for_status()?.json::<JiraRestIssue>().await?;

        Ok(JiraIssue {
            id: rest_issue.id,
            key: rest_issue.key,
            summary: rest_issue.fields.summary,
            status: JiraStatus {
                id: rest_issue.fields.status.id,
                name: rest_issue.fields.status.name,
            },
        })
    }
}
