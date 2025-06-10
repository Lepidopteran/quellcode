use log::{debug, trace};
use reqwest::header::{ACCEPT, USER_AGENT};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

use crate::app::APP_ID;

const ACCEPT_VALUE: &str = "application/vnd.github+json";
const GITHUB_API_VERSION: &str = "X-GitHub-Api-Version";
const GITHUB_API_VERSION_VALUE: &str = "2022-11-28";

mod repo_contents;
mod git_tree;

#[derive(Debug, Error)]
pub enum GithubApiError {
    #[error("Response error: {0}")]
    ResponseError(#[from] reqwest::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Url Parse error: {0}")]
    UrlParseError(#[from] url::ParseError),
    #[error("Invalid URL")]
    InvalidUrl,
}

type Result<T, E = GithubApiError> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct GithubApi {
    client: reqwest::Client,
    token: Option<secrecy::SecretString>,
}

impl GithubApi {
    pub fn new(client: reqwest::Client, token: Option<secrecy::SecretString>) -> Self {
        Self { client, token }
    }
}

pub use repo_contents::*;
pub use git_tree::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let api = GithubApi::new(reqwest::Client::new(), None);
    }
}
