use log::{debug, trace};
use reqwest::header::{ACCEPT, USER_AGENT};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

use crate::app::APP_ID;

const JSON_MEDIA_TYPE: &str = "application/vnd.github+json";
const RAW_MEDIA_TYPE: &str = "application/vnd.github.raw+json";
const GITHUB_API_VERSION: &str = "X-GitHub-Api-Version";
const GITHUB_API_VERSION_VALUE: &str = "2022-11-28";

mod git_blob_text;
mod git_tree;
mod repo_contents;
mod repo_info;

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
    #[error("Invalid Response")]
    InvalidResponse,
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

pub use git_blob_text::*;
pub use git_tree::*;
pub use repo_contents::*;
pub use repo_info::*;

pub fn get_owner_and_repo_from_url(url: Url) -> Result<(String, String)> {
    let segments = url
        .path_segments()
        .ok_or(GithubApiError::InvalidUrl)?
        .collect::<Vec<_>>();

    if segments.len() < 2 {
        return Err(GithubApiError::InvalidUrl);
    }

    Ok((segments[0].to_string(), segments[1].to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let api = GithubApi::new(reqwest::Client::new(), None);
    }
}
