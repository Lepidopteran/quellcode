use ::handlebars::{handlebars_helper, HelperDef};
use handlebars::RenderErrorReason;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, str::FromStr};
use ts_rs::TS;

use crate::property::PropertyInfo;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Template {
    pub name: String,
    pub template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateData {
    pub font_settings: FontSettings,
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
    pub path: PathBuf,
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

impl std::fmt::Display for Style {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Style::Normal => write!(f, "normal"),
            Style::Italic => write!(f, "italic"),
            Style::Oblique => write!(f, "oblique"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FontSettingsQuery {
    weight: Option<String>,
    style: Option<String>,
    monospaced: Option<bool>,
}

fn find_font_from_helper(
    h: &::handlebars::Helper<'_>,
) -> Result<Option<FontFace>, RenderErrorReason> {
    let font_family: FontSettings = serde_json::from_value(h.param(0).unwrap().value().clone())
        .map_err(|e| RenderErrorReason::NestedError(e.into()))?;

    let query = FontSettingsQuery {
        weight: h
            .hash_get("weight")
            .as_ref()
            .map(|v| v.value().as_str().unwrap().to_string()),

        style: h
            .hash_get("style")
            .as_ref()
            .map(|v| v.value().as_str().unwrap().to_string()),

        monospaced: h
            .hash_get("monospaced")
            .as_ref()
            .map(|v| v.value().as_bool().unwrap()),
    };

    Ok(font_family
        .fonts
        .iter()
        .find(|f| {
            query
                .weight
                .as_ref()
                .is_none_or(|w| Weight::from_str(w).expect("Invalid weight") == f.weight)
                && query
                    .style
                    .as_ref()
                    .is_none_or(|w| Style::from_str(w).expect("Invalid style") == f.style)
                && query.monospaced.as_ref().is_none_or(|m| *m == f.monospaced)
                && (query.style.is_some() || query.weight.is_some() || query.monospaced.is_some())
        })
        .cloned())
}

pub fn get_font_face_path_helper(
    h: &::handlebars::Helper<'_>,
    _: &::handlebars::Handlebars,
    _: &::handlebars::Context,
    _: &mut ::handlebars::RenderContext,
    out: &mut dyn ::handlebars::Output,
) -> ::handlebars::HelperResult {
    let font = find_font_from_helper(h)?;

    out.write(
        &font
            .map(|f| f.path.to_string_lossy().to_string())
            .unwrap_or_default(),
    )?;

    Ok(())
}

pub fn get_font_face_name_helper(
    h: &::handlebars::Helper<'_>,
    _: &::handlebars::Handlebars,
    _: &::handlebars::Context,
    _: &mut ::handlebars::RenderContext,
    out: &mut dyn ::handlebars::Output,
) -> ::handlebars::HelperResult {
    let font = find_font_from_helper(h)?;

    out.write(&font.map(|f| f.name.to_string()).unwrap_or_default())?;

    Ok(())
}

pub fn get_font_face_weight_helper(
    h: &::handlebars::Helper<'_>,
    _: &::handlebars::Handlebars,
    _: &::handlebars::Context,
    _: &mut ::handlebars::RenderContext,
    out: &mut dyn ::handlebars::Output,
) -> ::handlebars::HelperResult {
    let font = find_font_from_helper(h)?;

    out.write(&font.map(|f| f.weight.0.to_string()).unwrap_or_default())?;

    Ok(())
}

pub fn get_font_face_style_helper(
    h: &::handlebars::Helper<'_>,
    _: &::handlebars::Handlebars,
    _: &::handlebars::Context,
    _: &mut ::handlebars::RenderContext,
    out: &mut dyn ::handlebars::Output,
) -> ::handlebars::HelperResult {
    let font = find_font_from_helper(h)?;
    out.write(&font.map(|f| f.style.to_string()).unwrap_or_default())?;

    Ok(())
}

pub fn get_font_face_monospaced_helper(
    h: &::handlebars::Helper<'_>,
    _: &::handlebars::Handlebars,
    _: &::handlebars::Context,
    _: &mut ::handlebars::RenderContext,
    out: &mut dyn ::handlebars::Output,
) -> ::handlebars::HelperResult {
    let font = find_font_from_helper(h)?;
    out.write(&font.map(|f| f.monospaced.to_string()).unwrap_or_default())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use test_log::test;

    use super::*;

    #[test]
    fn test_get_font_face_weight() {
        let font = FontSettings {
            size: 10.0,
            family_name: "test".to_string(),
            fonts: vec![FontFace {
                name: "test".to_string(),
                path: PathBuf::from("test.ttf"),
                weight: Weight::NORMAL,
                style: Style::Normal,
                monospaced: false,
            }],
        };

        let template = "{{fontFaceWeight fontSettings weight=\"normal\" style=\"normal\"}}";
        let empty_template = "{{fontFaceWeight fontSettings weight=\"BOLD\" style=\"normal\"}}";

        let mut handlebars = ::handlebars::Handlebars::new();
        handlebars.set_strict_mode(true);

        handlebars.register_helper("fontFaceWeight", Box::new(get_font_face_weight_helper));
        handlebars.register_template_string("t1", template).unwrap();
        handlebars
            .register_template_string("t2", empty_template)
            .unwrap();

        let data = json!({"fontSettings": font});

        let t1 = handlebars.render("t1", &data).unwrap();
        let t2 = handlebars.render("t2", &data).unwrap();

        assert_eq!(t1, "400");
        assert_eq!(t2, "");
    }

    #[test]
    fn test_get_font_face_style() {
        let font = FontSettings {
            size: 10.0,
            family_name: "test".to_string(),
            fonts: vec![FontFace {
                name: "test".to_string(),
                path: PathBuf::from("test.ttf"),
                weight: Weight::NORMAL,
                style: Style::Normal,
                monospaced: false,
            }],
        };

        let template = "{{fontFaceStyle fontSettings weight=\"normal\" style=\"normal\"}}";
        let empty_template = "{{fontFaceStyle fontSettings weight=\"BOLD\" style=\"normal\"}}";

        let mut handlebars = ::handlebars::Handlebars::new();
        handlebars.set_strict_mode(true);

        handlebars.register_helper("fontFaceStyle", Box::new(get_font_face_style_helper));
        handlebars.register_template_string("t1", template).unwrap();
        handlebars
            .register_template_string("t2", empty_template)
            .unwrap();

        let data = json!({"fontSettings": font});

        let t1 = handlebars.render("t1", &data).unwrap();
        let t2 = handlebars.render("t2", &data).unwrap();

        assert_eq!(t1, "normal");
        assert_eq!(t2, "");
    }

    #[test]
    fn test_get_font_face_name() {
        let font = FontSettings {
            size: 10.0,
            family_name: "test".to_string(),
            fonts: vec![FontFace {
                name: "test".to_string(),
                path: PathBuf::from("test.ttf"),
                weight: Weight::NORMAL,
                style: Style::Normal,
                monospaced: false,
            }],
        };

        let template = "{{fontFaceName fontSettings weight=\"normal\" style=\"normal\"}}";
        let empty_template = "{{fontFaceName fontSettings weight=\"BOLD\" style=\"normal\"}}";

        let mut handlebars = ::handlebars::Handlebars::new();
        handlebars.set_strict_mode(true);

        handlebars.register_helper("fontFaceName", Box::new(get_font_face_name_helper));
        handlebars.register_template_string("t1", template).unwrap();
        handlebars
            .register_template_string("t2", empty_template)
            .unwrap();

        let data = json!({"fontSettings": font});

        let rendered = handlebars.render("t1", &data).unwrap();
        let empty_render = handlebars.render("t2", &data).unwrap();

        assert_eq!(rendered, "test");
        assert_eq!(empty_render, "");
    }

    #[test]
    fn test_get_font_face_monospaced() {
        let font = FontSettings {
            size: 10.0,
            family_name: "test".to_string(),
            fonts: vec![FontFace {
                name: "test".to_string(),
                path: PathBuf::from("test.ttf"),
                weight: Weight::NORMAL,
                style: Style::Normal,
                monospaced: true,
            }],
        };

        let template = "{{fontFaceMonospaced fontSettings weight=\"normal\" style=\"normal\"}}";
        let empty_template = "{{fontFaceMonospaced fontSettings weight=\"BOLD\" style=\"normal\"}}";

        let mut handlebars = ::handlebars::Handlebars::new();
        handlebars.set_strict_mode(true);

        handlebars.register_helper(
            "fontFaceMonospaced",
            Box::new(get_font_face_monospaced_helper),
        );
        handlebars.register_template_string("t1", template).unwrap();
        handlebars
            .register_template_string("t2", empty_template)
            .unwrap();

        let data = json!({"fontSettings": font});

        let rendered = handlebars.render("t1", &data).unwrap();
        let empty_render = handlebars.render("t2", &data).unwrap();

        assert_eq!(rendered, "true");
        assert_eq!(empty_render, "");
    }

    #[test]
    fn test_get_font_face_path() {
        let font = FontSettings {
            size: 10.0,
            family_name: "test".to_string(),
            fonts: vec![FontFace {
                name: "test".to_string(),
                path: PathBuf::from("test.ttf"),
                weight: Weight::NORMAL,
                style: Style::Normal,
                monospaced: false,
            }],
        };

        let template = "{{fontFacePath fontSettings weight=\"normal\"}}";
        let template2 = "{{fontFacePath fontSettings weight=\"Normal\"}}";
        let template3 = "{{fontFacePath fontSettings weight=\"Bold\"}}";

        let mut handlebars = ::handlebars::Handlebars::new();
        handlebars.set_strict_mode(true);

        handlebars.register_helper("fontFacePath", Box::new(get_font_face_path_helper));
        handlebars.register_template_string("t1", template).unwrap();
        handlebars
            .register_template_string("t2", template2)
            .unwrap();

        handlebars
            .register_template_string("t3", template3)
            .unwrap();

        let data = json!({
            "fontSettings": font,
        });

        let rendered = handlebars.render("t1", &data).expect("Failed to render");
        let rendered2 = handlebars.render("t2", &data).expect("Failed to render");
        let empty_render = handlebars.render("t3", &data).expect("Failed to render");

        assert_eq!(rendered, "test.ttf".to_string());
        assert_eq!(rendered2, "test.ttf".to_string());
        assert!(empty_render.is_empty());
    }
}
