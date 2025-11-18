use crate::property::{PropertyInfo, PropertyValue};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, str::FromStr};
use syntect::highlighting::{Style as SyntectStyle, Theme};
use ts_rs::TS;

mod helpers;
mod registry;

pub use helpers::*;
pub use registry::*;

type Scripts = HashMap<String, String>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub style: SyntectStyle,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TemplateInfo {
    pub name: String,
    pub content: String,
    pub description: String,
    pub extra_properties: Vec<PropertyInfo>,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct TemplateUserData {
    pub font_size: f32,
    pub font_family: String,
    pub theme_name: String,
    pub syntax_name: String,
    pub code: String,

    #[allow(dead_code)]
    pub props: HashMap<String, PropertyValue>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateData {
    pub font_settings: FontSettings,
    pub theme: Theme,
    pub lines: Vec<Vec<Token>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FontSettings {
    pub family_name: String,
    pub fonts: Vec<FontFace>,
    pub size: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FontFace {
    pub name: String,
    pub path: Option<PathBuf>,
    pub weight: Weight,
    pub style: Style,
    pub monospaced: bool,
}

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub struct Weight(pub u16);

impl Weight {
    pub const THIN: Weight = Weight(100);
    pub const EXTRA_LIGHT: Weight = Weight(200);
    pub const LIGHT: Weight = Weight(300);
    pub const NORMAL: Weight = Weight(400);
    pub const MEDIUM: Weight = Weight(500);
    pub const SEMIBOLD: Weight = Weight(600);
    pub const BOLD: Weight = Weight(700);
    pub const EXTRA_BOLD: Weight = Weight(800);
    pub const BLACK: Weight = Weight(900);
}

impl From<fontdb::Weight> for Weight {
    fn from(value: fontdb::Weight) -> Self {
        Weight(value.0)
    }
}

impl FromStr for Weight {
    type Err = String;

    fn from_str(s: &str) -> Result<Weight, Self::Err> {
        match s.to_lowercase().as_str() {
            "thin" => Ok(Weight::THIN),
            "extra-light" | "extralight" => Ok(Weight::EXTRA_LIGHT),
            "light" => Ok(Weight::LIGHT),
            "normal" => Ok(Weight::NORMAL),
            "medium" => Ok(Weight::MEDIUM),
            "semibold" => Ok(Weight::SEMIBOLD),
            "bold" => Ok(Weight::BOLD),
            "extra-bold" | "extrabold" => Ok(Weight::EXTRA_BOLD),
            "black" => Ok(Weight::BLACK),
            _ => u16::from_str(s)
                .map(Weight)
                .map_err(|e| format!("Could not convert to Weight: \"{e}\"")),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum Style {
    Normal,
    Italic,
    Oblique,
}

impl FromStr for Style {
    type Err = String;

    fn from_str(s: &str) -> Result<Style, Self::Err> {
        match s.to_lowercase().as_str() {
            "normal" => Ok(Style::Normal),
            "italic" => Ok(Style::Italic),
            "oblique" => Ok(Style::Oblique),
            _ => Err("Could not convert to Style".to_string()),
        }
    }
}

impl From<fontdb::Style> for Style {
    fn from(value: fontdb::Style) -> Self {
        match value {
            fontdb::Style::Normal => Style::Normal,
            fontdb::Style::Italic => Style::Italic,
            fontdb::Style::Oblique => Style::Oblique,
        }
    }
}
