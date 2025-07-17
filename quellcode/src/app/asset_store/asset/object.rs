use gtk::glib::{self, prelude::*, subclass::prelude::*};

use super::file::{FileInfo, FileInfoData};
use crate::app::scraping::package_control::Entry;

#[cfg(test)]
use super::init;

#[derive(
    Debug, Copy, Clone, Default, PartialEq, Eq, glib::Enum, serde::Serialize, serde::Deserialize,
)]
#[enum_type(name = "QuellcodeAssetType")]
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
    pub files: Vec<FileInfoData>,
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

mod imp {

    use gtk::glib::Properties;
    use std::cell::RefCell;

    use super::*;

    #[derive(Properties, Default)]
    #[properties(wrapper_type = super::Asset)]
    pub struct Asset {
        #[property(name = "name", get, set, member = name, type = String)]
        #[property(name = "description", get, set, member = description, type = String)]
        #[property(name = "authors", get, set, member = authors, type = Vec<String>)]
        #[property(name = "installs", get, set, member = installs, type = i64)]
        #[property(name = "source", get, set, member = source, type = String)]
        #[property(name = "url", get, set, member = url, type = String)]
        #[property(name = "kind", get, set, member = kind, type = AssetType, builder(AssetType::Unknown))]
        pub data: RefCell<AssetData>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Asset {
        const NAME: &'static str = "QuellcodeStoreAsset";
        type Type = super::Asset;
    }

    #[glib::derived_properties]
    impl ObjectImpl for Asset {}
}

glib::wrapper! {
    pub struct Asset(ObjectSubclass<imp::Asset>);
}

impl Asset {
    pub fn new(data: AssetData) -> Self {
        let asset: Self = glib::Object::builder().build();
        asset.imp().data.replace(data);

        asset
    }

    pub fn data(&self) -> AssetData {
        self.imp().data.borrow().clone()
    }
}

impl From<AssetData> for Asset {
    fn from(data: AssetData) -> Self {
        Asset::new(data)
    }
}
