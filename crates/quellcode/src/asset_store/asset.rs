use crate::scraping::package_control::Entry;

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub extension: String,
    pub sha256: String,
}

impl FileInfo {
    pub fn new(name: String, extension: String, sha256: String) -> Self {
        Self {
            name,
            extension,
            sha256,
        }
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AssetType {
    #[default]
    Unknown,
    ColorScheme,
    LanguageSyntax,
    VSCodeTheme,
}

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AssetData {
    pub name: String,
    pub description: String,
    pub authors: Vec<String>,
    pub source: String,
    pub url: String,
    pub kind: AssetType,
    pub installs: i64,
    pub files: Vec<FileInfo>,
}

impl From<Entry> for AssetData {
    fn from(package: Entry) -> Self {
        Self {
            url: format!("https://packagecontrol.io/packages/{}", package.name),
            name: package.name,
            description: package.description,
            authors: package.authors,
            source: String::from("Package Control"),
            installs: package.unique_installs.unwrap_or(0),
            kind: if package.labels.contains(&"color scheme".to_string()) {
                AssetType::ColorScheme
            } else if package.labels.contains(&"language syntax".to_string()) {
                AssetType::LanguageSyntax
            } else {
                AssetType::Unknown
            },
            ..Default::default()
        }
    }
}

impl From<&Entry> for AssetData {
    fn from(package: &Entry) -> Self {
        Self {
            url: format!("https://packagecontrol.io/packages/{}", package.name),
            name: package.name.clone(),
            description: package.description.clone(),
            authors: package.authors.clone(),
            source: String::from("Package Control"),
            installs: package.unique_installs.unwrap_or(0),
            kind: if package.labels.contains(&"color scheme".to_string()) {
                AssetType::ColorScheme
            } else if package.labels.contains(&"language syntax".to_string()) {
                AssetType::LanguageSyntax
            } else {
                AssetType::Unknown
            },
            ..Default::default()
        }
    }
}
