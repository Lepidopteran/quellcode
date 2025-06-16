use async_channel::Sender;
use color_eyre::eyre::Result;
use std::sync::Arc;
use tokio::sync::Semaphore;
use url::Url;

use crate::app::{
    scraping::{
        github_api::{GitTree, GithubApi, RepoInfo, TreeItem},
        package_control::{get_package, get_package_from_url},
    },
    util::send_async_channel,
};

use super::asset::{AssetData, AssetType, FileInfoData};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AssetDatabase {
    pub name: String,
    pub created_at: time::OffsetDateTime,
    pub data: Vec<AssetData>,
}

impl AssetDatabase {
    pub fn new(name: String, data: Vec<AssetData>) -> Self {
        Self {
            name,
            created_at: time::OffsetDateTime::now_utc(),
            data,
        }
    }

    pub fn add_asset(&mut self, asset: AssetData) {
        self.data.push(asset);
    }

    pub fn get_assets(&self) -> Vec<AssetData> {
        self.data.clone()
    }

    pub fn get_asset(&self, name: &str) -> Option<AssetData> {
        self.data.iter().find(|a| a.name == name).cloned()
    }
}

impl Default for AssetDatabase {
    fn default() -> Self {
        Self {
            name: String::new(),
            created_at: time::OffsetDateTime::now_utc(),
            data: vec![],
        }
    }
}

pub enum ProgressMessageKind {
    Starting,
    Misc,
    Error,
    Warning,
    Success,
}

pub struct ProgressMessage {
    pub index: usize,
    pub package_name: String,
    pub message: String,
    pub kind: ProgressMessageKind,
}

impl ProgressMessage {
    pub fn new(
        index: usize,
        package_name: String,
        message: String,
        kind: ProgressMessageKind,
    ) -> Self {
        Self {
            index,
            package_name,
            message,
            kind,
        }
    }
}

pub async fn index_assets_to_database<T: IntoIterator<Item = AssetData>>(
    api: &GithubApi,
    assets: T,
    progress: &Sender<ProgressMessage>,
) -> Result<AssetDatabase> {
    let mut database = AssetDatabase::default();

    let semaphore = Arc::new(Semaphore::new(5));
    let mut handles = Vec::new();

    for (index, asset) in assets.into_iter().enumerate() {
        let api = api.clone();
        let progress = progress.clone();
        let semaphore = semaphore.clone();
        let index = index + 1;

        let handle = tokio::task::spawn(async move {
            let _permit = semaphore.acquire_owned().await.unwrap();

            let asset_name = asset.name.clone();
            send_async_channel(
                &progress,
                ProgressMessage::new(
                    index,
                    asset_name.clone(),
                    String::new(),
                    ProgressMessageKind::Starting,
                ),
            )
            .await;

            let (tx, rx) = async_channel::bounded(1);
            let asset_name_clone = asset.name.clone();
            let async_progress = progress.clone();

            tokio::task::spawn(async move {
                while let Ok(message) = rx.recv().await {
                    send_async_channel(
                        &async_progress,
                        ProgressMessage::new(
                            index,
                            asset_name_clone.clone(),
                            message,
                            ProgressMessageKind::Misc,
                        ),
                    )
                    .await;
                }
            });

            let mut result_asset = None;

            match asset.kind {
                AssetType::LanguageSyntax => match get_syntax_files(&api, &asset.url, &tx).await {
                    Ok(Some(files)) => {
                        let mut asset = asset;
                        asset.files = files;
                        result_asset = Some(asset);
                    }
                    Ok(None) => {
                        send_async_channel(
                            &progress,
                            ProgressMessage::new(
                                index,
                                asset_name,
                                "No syntax files found... skipping".to_string(),
                                ProgressMessageKind::Warning,
                            ),
                        )
                        .await;
                    }
                    Err(e) => {
                        send_async_channel(
                            &progress,
                            ProgressMessage::new(
                                index,
                                asset_name,
                                format!("Error: {e}"),
                                ProgressMessageKind::Error,
                            ),
                        )
                        .await;
                    }
                },
                AssetType::ColorScheme => match get_theme_files(&api, &asset.url, &tx).await {
                    Ok(Some(files)) => {
                        let mut asset = asset;
                        asset.files = files;
                        result_asset = Some(asset);
                    }
                    Ok(None) => {
                        send_async_channel(
                            &progress,
                            ProgressMessage::new(
                                index,
                                asset_name,
                                "No color scheme files found... skipping".to_string(),
                                ProgressMessageKind::Warning,
                            ),
                        )
                        .await;
                    }
                    Err(e) => {
                        send_async_channel(
                            &progress,
                            ProgressMessage::new(
                                index,
                                asset_name,
                                format!("Error: {e}"),
                                ProgressMessageKind::Error,
                            ),
                        )
                        .await;
                    }
                },
                _ => {}
            }

            result_asset
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete and collect results
    for handle in handles {
        if let Ok(Some(asset)) = handle.await {
            database.add_asset(asset);
        }
    }

    Ok(database)
}

#[derive(Debug)]
pub enum ThemeType {
    TmTheme,
    SublimeColorScheme,
}

pub async fn get_syntax_files(
    api: &GithubApi,
    query: &str,
    progress: &Sender<String>,
) -> Result<Option<Vec<FileInfoData>>> {
    send_async_channel(progress, "Checking if query is a URL or name".to_string()).await;
    let package = if Url::parse(query).is_ok() {
        send_async_channel(progress, "Query is a URL".to_string()).await;
        get_package_from_url(query).await?
    } else {
        send_async_channel(progress, "Query is a name".to_string()).await;
        get_package(query).await?
    };

    if let Some(url) = package.sources.iter().find(|source| {
        source
            .host_str()
            .is_some_and(|host| host.contains("github.com"))
    }) {
        find_files_from_github_url(api, url, progress, |item| {
            item.path.ends_with(".sublime-syntax")
        })
        .await
    } else {
        Ok(None)
    }
}

pub async fn get_theme_files(
    api: &GithubApi,
    query: &str,
    progress: &Sender<String>,
) -> Result<Option<Vec<FileInfoData>>> {
    send_async_channel(progress, "Checking if query is a URL or name".to_string()).await;
    let package = if Url::parse(query).is_ok() {
        send_async_channel(progress, "Query is a URL".to_string()).await;
        get_package_from_url(query).await?
    } else {
        send_async_channel(progress, "Query is a name".to_string()).await;
        get_package(query).await?
    };

    if let Some(url) = package.sources.iter().find(|source| {
        source
            .host_str()
            .is_some_and(|host| host.contains("github.com"))
    }) {
        find_files_from_github_url(api, url, progress, |item| {
            matches!(
                item.path.rsplit('.').next(),
                Some(ext) if ext == "tmTheme" || ext == "sublime-color-scheme"
            )
        })
        .await
    } else {
        Ok(None)
    }
}

async fn find_files_from_github_url(
    api: &GithubApi,
    url: &Url,
    progress: &Sender<String>,
    filter: fn(&TreeItem) -> bool,
) -> Result<Option<Vec<FileInfoData>>> {
    send_async_channel(progress, "Getting repo info".to_string()).await;
    let repo_info = api.get_repo_info_from_url(url).await?;

    send_async_channel(progress, "Got repo info".to_string()).await;
    send_async_channel(progress, "Getting tree".to_string()).await;

    let tree = api
        .get_tree(
            &repo_info.owner.login,
            &repo_info.name,
            &repo_info.default_branch,
        )
        .await?;

    send_async_channel(progress, "Got tree".to_string()).await;

    let syntax_files = tree
        .tree
        .iter()
        .filter(|item| item.is_file() && filter(item))
        .map(tree_item_to_file_info)
        .collect::<Vec<_>>();

    send_async_channel(progress, format!("Found {} files", syntax_files.len())).await;

    if syntax_files.is_empty() {
        return Ok(None);
    }

    Ok(Some(syntax_files))
}

fn tree_item_to_file_info(item: &TreeItem) -> FileInfoData {
    let end_path = item.path.split('/').next_back().unwrap();
    let (name, extension) = end_path.split_once('.').unwrap();

    FileInfoData {
        name: name.to_string(),
        extension: extension.to_string(),
        sha256: item.sha.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_syntax_files() {
        let api = GithubApi::new(reqwest::Client::new(), None);
        let syntax_files = get_syntax_files(
            &api,
            "Dockerfile Syntax Highlighting",
            &async_channel::unbounded().0,
        )
        .await
        .unwrap();

        assert!(syntax_files.is_some());
    }

    #[tokio::test]
    async fn test_get_theme_files() {
        let api = GithubApi::new(reqwest::Client::new(), None);
        let theme_files = get_theme_files(&api, "1337 Color Scheme", &async_channel::unbounded().0)
            .await
            .unwrap();

        assert!(theme_files.is_some());
    }
}
