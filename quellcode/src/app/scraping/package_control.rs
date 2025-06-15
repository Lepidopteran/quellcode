use super::*;
use log::{debug, trace};
use serde::{Deserialize, Serialize};
use url::Url;

pub const LANGUAGE_SYNTAX: &str = "language syntax";
pub const COLOR_SCHEME: &str = "color scheme";
const BASE_URL: &str = "https://packagecontrol.io";

#[derive(Debug, Serialize, Deserialize)]
pub struct LabeledPackageList {
    pub name: String,
    pub packages: Vec<Entry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Entry {
    pub name: String,
    pub description: String,
    pub authors: Vec<String>,
    pub labels: Vec<String>,
    pub platforms: Vec<String>,
    pub st_versions: Vec<i64>,
    pub last_modified: String,
    pub last_seen: String,
    pub is_missing: bool,
    pub trending_rank: Option<i64>,
    pub needs_review: bool,
    pub installs_rank: Option<i64>,
    pub first_seen: String,
    pub z_value: Option<f64>,
    pub unique_installs: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub description: String,
    pub homepage: Option<Url>,
    pub previous_names: Vec<String>,
    pub labels: Vec<String>,
    pub platforms: Vec<String>,
    pub st_versions: Vec<u8>,
    pub last_modified: String,
    pub last_seen: String,
    pub sources: Vec<Url>,
    pub readme: Option<String>,
    pub issues: Option<String>,
    pub donate: Option<String>,
    pub buy: Option<String>,
    pub authors: Vec<String>,
    pub is_missing: bool,
    pub missing_error: String,
    pub needs_review: bool,
    pub removed: bool,
    pub trending_rank: Option<u32>,
    pub installs_rank: u32,
    pub first_seen: String,
    pub z_value: Option<f64>,
    pub versions: Vec<Version>,
    pub platforms_display: Vec<String>,
    pub installs: Installs,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Version {
    pub version: String,
    pub prerelease_version: Option<String>,
    pub platforms: Vec<String>,
    pub st_versions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Installs {
    pub total: u32,
    pub windows: u32,
    pub osx: u32,
    pub linux: u32,
}

#[derive(thiserror::Error, Debug)]
pub enum PackageControlError {
    #[error("Failed to fetch package: {0}")]
    FetchError(#[from] reqwest::Error),
    #[error("Failed to parse package: {0}")]
    ParseError(#[from] serde_json::Error),
    #[error("Failed to parse url: {0}")]
    UrlError(#[from] url::ParseError),
    #[error("Invalid url")]
    InvalidUrl,
}

type Result<T> = std::result::Result<T, PackageControlError>;

pub async fn get_packages_by_label(label: &str) -> Result<LabeledPackageList> {
    let url = format!("{BASE_URL}/browse/labels/{}.json", label);

    get_packages_by_label_from_url(&url).await
}

pub async fn get_packages_by_label_from_url(url: &str) -> Result<LabeledPackageList> {
    let _ = Url::parse(url)?;
    if !url.contains("packagecontrol.io") {
        return Err(PackageControlError::InvalidUrl);
    }

    let mut encoded_url = url.replace('%', "%25");

    if !encoded_url.ends_with(".json") {
        encoded_url.push_str(".json");
    }

    let client = reqwest::Client::new();
    let response = client.get(&encoded_url).send().await?;

    trace!("PackageControl Response: {:#?}", response);

    Ok(response.json().await?)
}

pub async fn get_package(name: &str) -> Result<Package> {
    let url = format!("{BASE_URL}/packages/{}.json", name);

    get_package_from_url(&url).await
}

pub async fn get_package_from_url(url: &str) -> Result<Package> {
    let _ = Url::parse(url)?;
    if !url.contains("packagecontrol.io") {
        return Err(PackageControlError::InvalidUrl);
    }

    let mut encoded_url = url.replace('%', "%25");

    if !encoded_url.ends_with(".json") {
        encoded_url.push_str(".json");
    }

    let client = reqwest::Client::new();
    let response = client.get(&encoded_url).send().await?;

    trace!("PackageControl Response: {:#?}", response);

    Ok(response.json().await?)
}

#[cfg(test)]
mod tests {
    use log::debug;

    use super::test_util::*;
    use super::*;

    #[tokio::test]
    async fn test_get_packages_by_label() {
        init();

        let package_list = get_packages_by_label(LANGUAGE_SYNTAX).await.unwrap();
        assert!(!package_list.packages.is_empty());
    }

    #[tokio::test]
    async fn test_get_packages_by_label_color_scheme() {
        init();

        let package_list = get_packages_by_label(COLOR_SCHEME).await.unwrap();
        assert!(!package_list.packages.is_empty());
    }

    #[tokio::test]
    async fn test_get_packages_by_label_from_url() {
        init();

        let package_list =
            get_packages_by_label_from_url("https://packagecontrol.io/browse/labels/color scheme")
                .await
                .unwrap();
        assert!(!package_list.packages.is_empty());
    }

    #[tokio::test]
    async fn test_get_package() {
        init();

        let package = get_package("Emmet").await.unwrap();
        debug!("{:?}", package);

        assert!(!package.name.is_empty());
    }

    #[tokio::test]
    async fn test_get_package_from_url() {
        init();

        let package = get_package_from_url("https://packagecontrol.io/packages/Emmet")
            .await
            .unwrap();
        debug!("{:?}", package);

        assert!(!package.name.is_empty());
    }

    #[tokio::test]
    async fn test_get_package_from_special_character_url() {
        init();
        let package = get_package_from_url(
            "https://packagecontrol.io/packages/10% Too Dull for My Tastes Color Scheme",
        )
        .await
        .unwrap();

        debug!("{:?}", package);
        assert!(!package.name.is_empty());
    }
}
