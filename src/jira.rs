use reqwest::Client;
use reqwest::Error as ReqwestError;
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
    #[error("Cannot create http client {0}")]
    CannotCreateClient(ReqwestError),
    #[error("Remote server error {0}")]
    RemoteServerError(ReqwestError),
    #[error("Invalid server response {0}")]
    ResponseError(ReqwestError),
    #[error("Invalid server url {0}")]
    InvalidUrl(ParseError),
    #[error("Issue not found")]
    IssueNotFound,
}

impl JiraUser {
    pub fn new(name: String) -> Self {
        Self(name)
    }
}

impl JiraToken {
    pub fn new(token: String) -> Self {
        Self(token)
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
        let host = Url::parse(&config.host).map_err(JiraError::InvalidUrl)?;
        let user = JiraUser(config.user.clone());
        let token = JiraToken(config.token.clone());
        Ok(JiraServer {
            host,
            credentials: JiraCredentials { user, token },
        })
    }
}

impl JiraServer {
    pub fn from(config: &JiraConfig) -> Result<JiraServer, JiraError> {
        let host = Url::parse(&config.host).map_err(JiraError::InvalidUrl)?;
        let user = JiraUser(config.user.clone());
        let token = JiraToken(config.token.clone());
        Ok(JiraServer {
            host,
            credentials: JiraCredentials { user, token },
        })
    }

    pub async fn get_issue(&self, key_or_id: String) -> Result<JiraIssue, JiraError> {
        let url = self
            .host
            .join("/rest/api/2/issue/")
            .map_err(parse_error)?
            .join(&key_or_id)
            .map_err(parse_error)?;

        let client = Client::builder()
            .build()
            .map_err(JiraError::CannotCreateClient)?;

        client
            .get(url)
            .basic_auth(&self.credentials.user.0, Some(&self.credentials.token.0))
            .send()
            .await
            .map_err(JiraError::RemoteServerError)?
            .error_for_status()
            .map_err(|_| JiraError::IssueNotFound)?
            .json::<JiraRestIssue>()
            .await
            .map_err(JiraError::ResponseError)
            .map(|rest_issue| -> JiraIssue {
                JiraIssue {
                    id: rest_issue.id,
                    key: rest_issue.key,
                    summary: rest_issue.fields.summary,
                    status: JiraStatus {
                        id: rest_issue.fields.status.id,
                        name: rest_issue.fields.status.name,
                    },
                }
            })
    }
}

fn parse_error(e: ParseError) -> JiraError {
    JiraError::InvalidUrl(e)
}
