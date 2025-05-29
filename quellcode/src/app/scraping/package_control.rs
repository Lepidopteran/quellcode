use serde::{Deserialize, Serialize};
use super::*;

pub const LANGUAGE_SYNTAX: &str = "language syntax";
pub const COLOR_SCHEME: &str = "color scheme";

#[derive(Debug, Serialize, Deserialize)]
pub struct LabeledPackageList {
    pub name: String,
    pub packages: Vec<Package>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Package {
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

pub async fn get_packages_by_label(label: &str) -> Result<LabeledPackageList, reqwest::Error> {
    let url = format!("https://packagecontrol.io/browse/labels/{}.json", label);
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
}
