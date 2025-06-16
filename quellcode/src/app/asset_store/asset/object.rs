use gtk::glib::{self, prelude::*, subclass::prelude::*};

use super::file::{FileInfo, FileInfoData};
use crate::app::scraping::package_control::Entry;

#[cfg(test)]
use super::init;

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum AssetType {
    #[default]
    Unknown,
    ColorScheme,
    LanguageSyntax,
    VSCodeTheme,
}

impl From<u8> for AssetType {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::ColorScheme,
            2 => Self::LanguageSyntax,
            3 => Self::VSCodeTheme,
            _ => Self::Unknown,
        }
    }
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
        #[property(name = "kind", get = |a: &Asset| a.data.borrow().kind.clone() as u8, set = |a: &Asset, v: u8| a.data.borrow_mut().kind = v.into(), type = u8)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_type_from_u8() {
        init();

        assert_eq!(AssetType::from(1), AssetType::ColorScheme);
        assert_eq!(AssetType::from(2), AssetType::LanguageSyntax);
        assert_eq!(AssetType::from(3), AssetType::VSCodeTheme);
        assert_eq!(AssetType::from(4), AssetType::Unknown);
    }

    #[test]
    fn test_asset_type_to_u8() {
        init();

        assert_eq!(AssetType::Unknown as u8, 0);
        assert_eq!(AssetType::ColorScheme as u8, 1);
        assert_eq!(AssetType::LanguageSyntax as u8, 2);
        assert_eq!(AssetType::VSCodeTheme as u8, 3);
    }
}
