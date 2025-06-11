use gtk::glib::{self, prelude::*, subclass::prelude::*};

use crate::app::scraping::package_control::Entry;

#[derive(Debug, Clone, Default)]
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

#[derive(Default)]
pub struct AssetData {
    pub name: String,
    pub description: String,
    pub authors: Vec<String>,
    pub kind: AssetType,
    pub installs: i64,
}

impl From<Entry> for AssetData {
    fn from(package: Entry) -> Self {
        Self {
            name: package.name,
            description: package.description,
            authors: package.authors,
            installs: package.unique_installs.unwrap_or(0),
            kind: if package.labels.contains(&"color scheme".to_string()) {
                AssetType::ColorScheme
            } else if package.labels.contains(&"language syntax".to_string()) {
                AssetType::LanguageSyntax
            } else {
                AssetType::Unknown
            },
        }
    }
}

mod imp {

    use gtk::glib::Properties;
    use std::cell::{Cell, RefCell};

    use super::*;

    #[derive(Properties, Default)]
    #[properties(wrapper_type = super::Asset)]
    pub struct Asset {
        #[property(name = "name", get, set, member = name, type = String)]
        #[property(name = "description", get, set, member = description, type = String)]
        #[property(name = "authors", get, set, member = authors, type = Vec<String>)]
        #[property(name = "installs", get, set, member = installs, type = i64)]
        #[property(name = "kind", get = |a: &Asset| a.data.borrow().kind.clone() as u8, set = |a: &Asset, v: u8| a.data.borrow_mut().kind = AssetType::from(v), type = u8)]
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
        glib::Object::builder()
            .property("name", data.name)
            .property("description", data.description)
            .property("authors", data.authors)
            .property("installs", data.installs)
            .property("kind", data.kind as u8)
            .build()
    }
}

impl From<AssetData> for Asset {
    fn from(data: AssetData) -> Self {
        Asset::new(data)
    }
}
