use async_channel::Sender;
use color_eyre::eyre::Result;
use url::Url;

use crate::app::{
    scraping::{
        github_api::{GitTree, GithubApi, RepoInfo},
        package_control::{get_package, get_package_from_url},
    },
    util::send_async_channel,
};

#[derive(Debug)]
pub enum ThemeType {
    TmTheme,
    SublimeColorScheme,
}

pub async fn get_syntax_files(
    api: &GithubApi,
    query: &str,
    progress: &Sender<String>,
) -> Result<Option<Vec<String>>> {
    send_async_channel(progress, "Checking if query is a URL or name".to_string()).await;
    let package = if let Ok(url) = Url::parse(query) {
        send_async_channel(progress, "Query is a URL".to_string()).await;
        get_package_from_url(url.as_str()).await?
    } else {
        send_async_channel(progress, "Query is a name".to_string()).await;
        get_package(query).await?
    };

    if let Some(url) = package.sources.iter().find(|source| {
        source
            .host_str()
            .is_some_and(|host| host.contains("github.com"))
    }) {
        find_syntax_files_from_github_url(api, url, progress).await
    } else {
        send_async_channel(progress, "Package has no github source".to_string()).await;
        Ok(None)
    }
}

async fn find_syntax_files_from_github_url(
    api: &GithubApi,
    url: &Url,
    progress: &Sender<String>,
) -> Result<Option<Vec<String>>> {
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
        .filter(|item| item.is_file() && item.path.ends_with(".sublime-syntax"))
        .map(|item| item.sha.to_string())
        .collect::<Vec<_>>();

    send_async_channel(progress, format!("Found {} files", syntax_files.len())).await;

    if syntax_files.is_empty() {
        return Ok(None);
    }

    Ok(Some(syntax_files))
}

pub async fn get_theme_files(
    api: &GithubApi,
    query: &str,
    progress: &Sender<String>,
) -> Result<Option<Vec<(ThemeType, String)>>> {
    send_async_channel(progress, "Checking if query is a URL or name".to_string()).await;
    let package = if let Ok(url) = Url::parse(query) {
        send_async_channel(progress, "Query is a URL".to_string()).await;
        get_package_from_url(url.as_str()).await?
    } else {
        send_async_channel(progress, "Query is a name".to_string()).await;
        get_package(query).await?
    };

    if let Some(url) = package.sources.iter().find(|source| {
        source
            .host_str()
            .is_some_and(|host| host.contains("github.com"))
    }) {
        find_theme_files_from_github_url(api, url, progress).await
    } else {
        send_async_channel(progress, "Package has no github source".to_string()).await;
        Ok(None)
    }
}

pub async fn find_theme_files_from_github_url(
    api: &GithubApi,
    url: &Url,
    progress: &Sender<String>,
) -> Result<Option<Vec<(ThemeType, String)>>> {
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

    let theme_files = tree
        .tree
        .iter()
        .filter(|item| item.is_file())
        .filter_map(|item| {
            if item.path.ends_with(".tmTheme") {
                Some((ThemeType::TmTheme, item.sha.to_string()))
            } else if item.path.ends_with(".sublime-color-scheme") {
                Some((ThemeType::SublimeColorScheme, item.sha.to_string()))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    send_async_channel(progress, format!("Found {} files", theme_files.len())).await;

    if theme_files.is_empty() {
        return Ok(None);
    }

    Ok(Some(theme_files))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_find_syntax_files_from_github_url() {
        let api = GithubApi::new(reqwest::Client::new(), None);
        let url = Url::parse("https://github.com/asbjornenge/Docker.tmbundle").unwrap();
        let syntax_files =
            find_syntax_files_from_github_url(&api, &url, &async_channel::unbounded().0)
                .await
                .unwrap();

        assert!(syntax_files.is_some());
    }

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
        let theme_files = get_theme_files(
            &api,
            "1337 Color Scheme",
            &async_channel::unbounded().0,
        )
        .await
        .unwrap();

        assert!(theme_files.is_some());
    }

    #[tokio::test]
    async fn test_get_theme_files_from_url() {
        let api = GithubApi::new(reqwest::Client::new(), None);
        let url = Url::parse("https://github.com/MarkMichos/1337-Scheme").unwrap();
        let theme_files = find_theme_files_from_github_url(
            &api,
            &url,
            &async_channel::unbounded().0,
        )
        .await
        .unwrap();

        assert!(theme_files.is_some());
    }
}
