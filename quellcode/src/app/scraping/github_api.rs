use std::collections::HashMap;

use log::{debug, trace};
use reqwest::header::{ACCEPT, USER_AGENT};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::app::APP_ID;

const ACCEPT_VALUE: &str = "application/vnd.github+json";
const GITHUB_API_VERSION: &str = "X-GitHub-Api-Version";
const GITHUB_API_VERSION_VALUE: &str = "2022-11-28";

#[derive(Debug, Error)]
pub enum ContentError {
    #[error("Response error: {0}")]
    ResponseError(#[from] reqwest::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Invalid URL")]
    InvalidUrl,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Entry {
    #[serde(rename = "type")]
    pub type_: String,
    pub size: i64,
    pub name: String,
    pub path: String,
    pub sha: String,
    pub url: String,
    pub git_url: Option<String>,
    pub html_url: Option<String>,
    pub download_url: Option<String>,
    pub _links: Links,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Links {
    pub git: Option<String>,
    pub html: Option<String>,
    #[serde(rename = "self")]
    pub self_: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Content {
    #[serde(rename = "type")]
    pub type_: String,
    pub size: i64,
    pub name: String,
    pub path: String,
    pub sha: String,
    pub content: Option<String>,
    pub url: String,
    pub git_url: Option<String>,
    pub html_url: Option<String>,
    pub download_url: Option<String>,
    pub entries: Option<Vec<Entry>>,
    pub encoding: Option<String>,
    pub _links: Links,
}

impl Content {
    pub fn is_dir(&self) -> bool {
        self.type_ == "dir"
    }

    pub fn is_file(&self) -> bool {
        self.type_ == "file"
    }
}

#[derive(Debug)]
pub enum ContentResponse {
    Dir(Vec<Content>),
    File(Content),
}

impl From<Content> for ContentResponse {
    fn from(content: Content) -> Self {
        ContentResponse::File(content)
    }
}

impl From<Vec<Content>> for ContentResponse {
    fn from(contents: Vec<Content>) -> Self {
        ContentResponse::Dir(contents)
    }
}

impl ContentResponse {
    pub fn is_dir(&self) -> bool {
        match self {
            Self::Dir(_) => true,
            Self::File(_) => false,
        }
    }

    pub fn is_file(&self) -> bool {
        match self {
            Self::Dir(_) => false,
            Self::File(_) => true,
        }
    }
}

pub async fn get_content(
    owner: &str,
    repo: &str,
    path: &str,
) -> Result<ContentResponse, ContentError> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/contents/{path}");
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header(GITHUB_API_VERSION, GITHUB_API_VERSION_VALUE)
        .header(ACCEPT, ACCEPT_VALUE)
        .header(USER_AGENT, APP_ID)
        .send()
        .await?;

    trace!("Github API Response: {:#?}", response);
    let bytes = response.bytes().await?;
    let content_response: ContentResponse = {
        if let Ok(content) = serde_json::from_slice::<Content>(&bytes) {
            content.into()
        } else {
            let contents = serde_json::from_slice::<Vec<Content>>(&bytes)?;

            contents.into()
        }
    };

    Ok(content_response)
}

pub async fn get_content_from_url(url: &str) -> Result<ContentResponse, ContentError> {
    let parts: Vec<_> = url.split('/').collect();
    if parts.len() < 5 || !url.contains("github.com") {
        return Err(ContentError::InvalidUrl);
    }

    let path = parts
        .iter()
        .skip(5)
        .copied()
        .collect::<Vec<&str>>()
        .join("/");

    debug!(
        "Getting content from Owner: {}, Repo: {}, Path: {}",
        parts[3], parts[4], path
    );

    get_content(parts[3], parts[4], &path).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[tokio::test]
    async fn test_get_content_tree() {
        init();
        let content_tree = get_content("octocat", "hello-world", "README").await;

        debug!("content_tree: {:#?}", content_tree);
        assert!(content_tree.is_ok());
    }

    #[tokio::test]
    async fn test_get_content_from_url() {
        init();
        for url in [
            "https://github.com/microsoft/vscode",
            "https://github.com/microsoft/vscode/README.md",
        ]
        .iter()
        {
            let content = get_content_from_url(url).await;
            debug!("content_tree: {:#?}", content);
            assert!(content.is_ok());
        }
    }
}
