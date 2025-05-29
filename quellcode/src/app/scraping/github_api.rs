use std::collections::HashMap;

use reqwest::header::{ACCEPT, USER_AGENT};
use serde::{Deserialize, Serialize};

use crate::app::APP_ID;

const ACCEPT_VALUE: &str = "application/vnd.github+json";
const GITHUB_API_VERSION: &str = "X-GitHub-Api-Version";
const GITHUB_API_VERSION_VALUE: &str = "2022-11-28";

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
pub struct ContentTree {
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

impl ContentTree {
    pub fn is_dir(&self) -> bool {
        self.type_ == "dir"
    }

    pub fn is_file(&self) -> bool {
        self.type_ == "file"
    }
}

pub async fn get_content(
    owner: &str,
    repo: &str,
    path: &str,
) -> Result<Vec<ContentTree>, reqwest::Error> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/contents/{path}");
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header(GITHUB_API_VERSION, GITHUB_API_VERSION_VALUE)
        .header(ACCEPT, ACCEPT_VALUE)
        .header(USER_AGENT, APP_ID)
        .send()
        .await?;

    response.json().await
}

pub async fn get_content_tree_from_url(url: &str) -> color_eyre::eyre::Result<Vec<ContentTree>> {
    let parts: Vec<_> = url.split('/').collect();
    if parts.len() != 5 {
        return Err(color_eyre::eyre::eyre!("Invalid URL"));
    }

    Ok(get_content(parts[3], parts[4], "").await?)
}

#[cfg(test)]
mod tests {
    use log::debug;

    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[tokio::test]
    async fn test_get_content_tree() {
        init();
        let content_tree = get_content("octocat", "hello-world", "").await;

        debug!("content_tree: {:#?}", content_tree);
        assert!(content_tree.is_ok());
    }
}
