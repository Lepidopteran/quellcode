use async_channel::Sender;
use color_eyre::eyre::Result;
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
    pub created_at: time::OffsetDateTime,
    pub data: Vec<AssetData>,
}

impl AssetDatabase {
    pub fn new(data: Vec<AssetData>) -> Self {
        Self {
            created_at: time::OffsetDateTime::now_utc(),
            data,
        }
    }

    pub fn from_data(data: Vec<AssetData>) -> Self {
        Self {
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
            created_at: time::OffsetDateTime::now_utc(),
            data: vec![],
        }
    }
}

pub async fn index_assets_to_database<T: IntoIterator<Item = AssetData>>(
    api: &GithubApi,
    assets: T,
    progress: &Sender<(usize, String, String)>,
) -> Result<AssetDatabase> {
    let mut database = AssetDatabase::default();
    for (index, asset) in assets.into_iter().enumerate() {
        let index = index + 1;
        let asset_name = asset.name.to_string();
        send_async_channel(progress, (index, asset_name, "Starting".to_string())).await;

        let (tx, rx) = async_channel::bounded(1);
        let asset_name = asset.name.clone();
        let progress = progress.clone();
        tokio::spawn(async move {
            while let Ok(message) = rx.recv().await {
                send_async_channel(&progress, (index, asset_name.to_string(), message)).await;
            }
        });

        match asset.kind {
            AssetType::LanguageSyntax => {
                if let Some(files) = get_syntax_files(api, &asset.url, &tx).await? {
                    let mut asset = asset;
                    asset.files = files;

                    database.add_asset(asset);
                }
            }
            AssetType::ColorScheme => {
                if let Some(files) = get_theme_files(api, &asset.url, &tx).await? {
                    let mut asset = asset;
                    asset.files = files;

                    database.add_asset(asset);
                }
            }
            _ => {}
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
        send_async_channel(progress, "Package has no github source".to_string()).await;
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
