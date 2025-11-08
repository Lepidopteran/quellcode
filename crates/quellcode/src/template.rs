use color_eyre::eyre::OptionExt;
use fontdb::Source;
use handlebars::{
    Context, Handlebars, Helper, Output, RenderContext, RenderErrorReason, Renderable,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, str::FromStr};
use syntect::{
    highlighting::{Theme, ThemeSet},
};
use ts_rs::TS;

use crate::property::{PropertyInfo, PropertyValue};

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

    #[allow(dead_code)]
    pub props: HashMap<String, PropertyValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateData {
    pub font_settings: FontSettings,
    pub theme: Theme,
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

pub fn get_font_face_helper<'reg, 'rc>(
    h: &Helper<'rc>,
    r: &'reg Handlebars<'reg>,
    ctx: &'rc Context,
    rc: &mut RenderContext<'reg, 'rc>,
    out: &mut dyn Output,
) -> ::handlebars::HelperResult {
    let font = find_font_from_helper(h)?;

    if let Some(font) = font {
        rc.set_context(Context::wraps(font)?);

        h.template()
            .map(|template| template.render(r, ctx, rc, out))
            .unwrap_or(Ok(()))
    } else {
        h.inverse()
            .map(|inverse| inverse.render(r, ctx, rc, out))
            .unwrap_or(Ok(()))
    }
}

pub fn render_template(
    font_db: &fontdb::Database,
    handlebars: &Handlebars,
    themes: &ThemeSet,
    template_name: String,
    data: TemplateUserData,
) -> Result<String, RenderErrorReason> {
    let font_settings = FontSettings {
        family_name: data.font_family.clone(),
        size: data.font_size,
        fonts: font_db
            .faces()
            .filter(|f| f.families.iter().any(|(fam, _)| *fam == data.font_family))
            .map(|f| FontFace {
                name: f.post_script_name.clone(),
                path: if let Source::File(path) = &f.source {
                    Some(path.to_path_buf())
                } else {
                    None
                },
                weight: f.weight.into(),
                style: f.style.into(),
                monospaced: f.monospaced,
            })
            .collect(),
    };

    let theme = themes
        .themes
        .iter()
        .find_map(|t| {
            if *t.0 == data.theme_name {
                Some(t.1.clone())
            } else {
                None
            }
        })
        .ok_or_eyre("Theme not found")
        .map_err(|err| RenderErrorReason::NestedError(err.into()))?;

    let result = handlebars.render(
        &template_name,
        &TemplateData {
            font_settings,
            theme,
        },
    )?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use test_log::test;

    use super::*;

    fn font_settings() -> FontSettings {
        FontSettings {
            size: 10.0,
            family_name: "test".to_string(),
            fonts: vec![FontFace {
                name: "test".to_string(),
                path: Some(PathBuf::from("test.ttf")),
                weight: Weight::NORMAL,
                style: Style::Normal,
                monospaced: false,
            }],
        }
    }

    #[test]
    fn test_get_font_face() {
        let font = font_settings();

        let template = r#"{{#fontFace fontSettings weight="normal" style="normal"}}{{name}} {{weight}}{{/fontFace}}"#;

        let mut handlebars = ::handlebars::Handlebars::new();
        handlebars.set_strict_mode(true);

        handlebars.register_helper("fontFace", Box::new(get_font_face_helper));
        handlebars.register_template_string("t1", template).unwrap();

        let data = json!({"fontSettings": font});

        let rendered = handlebars.render("t1", &data);

        assert!(rendered.is_ok());
    }
}
