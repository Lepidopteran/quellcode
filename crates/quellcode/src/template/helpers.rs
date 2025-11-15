use std::str::FromStr;

use color_eyre::eyre::OptionExt;
use fontdb::Source;
use handlebars::{BlockContext, Context, Handlebars, Helper, HelperResult, Output, PathAndJson, RenderContext, RenderErrorReason, Renderable};
use serde::{Deserialize, Serialize};
use syntect::{easy::HighlightLines, highlighting::{Color, ThemeSet}, parsing::SyntaxSet, util::LinesWithEndings};

use crate::template::{Style, TemplateData, TemplateUserData, Token};

use super::{FontFace, FontSettings, Weight};


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
        let mut block = BlockContext::new();
        block.set_base_value(serde_json::to_value(font).expect("Could not serialize font"));
        rc.push_block(block);

        h.template()
            .map(|template| template.render(r, ctx, rc, out))
            .unwrap_or(Ok(()))?;

        rc.pop_block();

        Ok(())
    } else {
        h.inverse()
            .map(|inverse| inverse.render(r, ctx, rc, out))
            .unwrap_or(Ok(()))
    }
}

pub fn hex_color_helper(
    h: &Helper<'_>,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> HelperResult {
    let color: Color = serde_json::from_value(
        h.param(0)
            .ok_or_else(|| RenderErrorReason::ParamNotFoundForIndex("color", 0))?
            .value()
            .clone(),
    )
    .expect("Could not deserialize color");

    if color.a < 255 {
        write!(
            out,
            "#{:02x}{:02x}{:02x}{:02x}",
            color.r, color.g, color.b, color.a
        )?;
    } else {
        write!(out, "#{:02x}{:02x}{:02x}", color.r, color.g, color.b)?;
    }

    Ok(())
}

pub fn color_channel_to_float_helper(
    h: &Helper<'_>,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> HelperResult {
    let channel = h
        .param(0)
        .ok_or_else(|| RenderErrorReason::MissingVariable(Some(String::from("channel"))))?
        .value()
        .clone()
        .as_u64()
        .ok_or_else(|| {
            RenderErrorReason::InvalidParamType("channel must be an unsigned integer")
        })?;

    let max_value = h
        .hash_get("max")
        .unwrap_or(&PathAndJson::new(
            None,
            handlebars::ScopedJson::Derived(serde_json::Value::Number(255.into())),
        ))
        .value()
        .as_u64()
        .ok_or_else(|| RenderErrorReason::InvalidParamType("max must be an positive integer"))?;

    let decimals = h
        .hash_get("decimals")
        .unwrap_or(&PathAndJson::new(
            None,
            handlebars::ScopedJson::Derived(serde_json::Value::Number(2.into())),
        ))
        .value()
        .as_u64()
        .ok_or_else(|| {
            RenderErrorReason::InvalidParamType("decimals must be an positive integer")
        })? as usize;

    write!(out, "{:.decimals$}", channel as f64 / max_value as f64)?;

    Ok(())
}


#[cfg(test)]
mod tests {
    use std::{collections::HashMap, path::PathBuf};

    use log::info;
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
    fn test_hex_color() {
        let mut handlebars = ::handlebars::Handlebars::new();
        handlebars.set_strict_mode(true);
        handlebars.register_helper("colorToHex", Box::new(hex_color_helper));

        let color = Color {
            r: 230,
            g: 0,
            b: 200,
            a: 255,
        };

        let result = handlebars
            .render_template("{{colorToHex color}}", &json!({"color": color}))
            .unwrap();

        assert_eq!(result, "#e600c8");
    }

    #[test]
    fn test_color_channel() {
        let mut handlebars = ::handlebars::Handlebars::new();
        handlebars.set_strict_mode(true);
        handlebars.register_helper(
            "colorChannelToFloat",
            Box::new(color_channel_to_float_helper),
        );

        let color = Color {
            r: 230,
            g: 0,
            b: 200,
            a: 255,
        };

        let result = handlebars
            .render_template("{{colorChannelToFloat color.a }}", &json!({"color": color}))
            .unwrap();

        assert_eq!(result, "1.00");
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
