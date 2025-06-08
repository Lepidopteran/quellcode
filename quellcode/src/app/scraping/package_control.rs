use serde::{Deserialize, Serialize};
use super::*;

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
    name: String,
    description: String,
    homepage: String,
    previous_names: Vec<String>,
    labels: Vec<String>,
    platforms: Vec<String>,
    st_versions: Vec<u8>,
    last_modified: String,
    last_seen: String,
    sources: Vec<String>,
    readme: String,
    issues: String,
    donate: Option<String>,
    buy: Option<String>,
    authors: Vec<String>,
    is_missing: bool,
    missing_error: String,
    needs_review: bool,
    removed: bool,
    trending_rank: Option<u32>,
    installs_rank: u32,
    first_seen: String,
    z_value: Option<f64>,
    versions: Vec<Version>,
    platforms_display: Vec<String>,
    installs: Installs,
}

#[derive(Debug, Serialize, Deserialize)]
struct Version {
    version: String,
    prerelease_version: Option<String>,
    platforms: Vec<String>,
    st_versions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Installs {
    total: u32,
    windows: u32,
    osx: u32,
    linux: u32,
}

pub async fn get_packages_by_label(label: &str) -> Result<LabeledPackageList, reqwest::Error> {
    let url = format!("{BASE_URL}/browse/labels/{}.json", label);
    let client = reqwest::Client::new();
    let response = client.get(&url).send().await?;

    response.json().await
}

pub async fn get_package(name: &str) -> Result<Package, reqwest::Error> {
    let url = format!("{BASE_URL}/packages/{}.json", name);
    let client = reqwest::Client::new();
    let response = client.get(&url).send().await?;

    response.json().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_packages_by_label() {
        let package_list = get_packages_by_label(LANGUAGE_SYNTAX).await.unwrap();
        assert!(!package_list.packages.is_empty());
    }

    #[tokio::test]
    async fn test_get_packages_by_label_color_scheme() {
        let package_list = get_packages_by_label(COLOR_SCHEME).await.unwrap();
        assert!(!package_list.packages.is_empty());
    }

    #[tokio::test]
    async fn test_get_package() {
        let package = get_package("Emmet").await.unwrap();
        println!("{:#?}", package);
        assert!(!package.name.is_empty());
    }
}
