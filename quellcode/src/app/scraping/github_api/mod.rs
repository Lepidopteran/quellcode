use log::{debug, trace};
use reqwest::header::{ACCEPT, USER_AGENT};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

use crate::app::APP_ID;

const ACCEPT_VALUE: &str = "application/vnd.github+json";
const GITHUB_API_VERSION: &str = "X-GitHub-Api-Version";
const GITHUB_API_VERSION_VALUE: &str = "2022-11-28";

mod repo_contents;

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

pub use repo_contents::*;
