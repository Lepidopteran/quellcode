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

pub fn render_template(
    font_db: &fontdb::Database,
    handlebars: &Handlebars,
    themes: &ThemeSet,
    syntaxes: &SyntaxSet,
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

    let syntax = syntaxes
        .find_syntax_by_name(&data.syntax_name)
        .ok_or_eyre("Syntax not found")
        .map_err(|err| RenderErrorReason::NestedError(err.into()))?;

    let mut highlighter = HighlightLines::new(syntax, &theme);
    let mut lines = vec![];

    for line in LinesWithEndings::from(data.code.as_str()) {
        let line_ranges: Vec<_> = highlighter
            .highlight_line(line, syntaxes)
            .map_err(|err| RenderErrorReason::NestedError(err.into()))?
            .iter()
            .map(|r| Token {
                style: r.0,
                text: r.1.to_string(),
            })
            .collect();

        lines.push(line_ranges);
    }

    let result = handlebars.render(
        &template_name,
        &TemplateData {
            font_settings,
            theme,
            lines,
        },
    )?;

    Ok(result)
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
    fn test_render_template() {
        let mut handlebars = ::handlebars::Handlebars::new();
        handlebars.set_strict_mode(true);

        let mut font_db = fontdb::Database::new();
        font_db.load_system_fonts();

        let syntax_set = SyntaxSet::load_defaults_nonewlines();
        let theme_set = ThemeSet::load_defaults();

        let template = r#"
        let font_family = `{{fontSettings.familyName}}`
        let font_size = `{{fontSettings.size}}`

        let font_paths = [{{~#each fontSettings.fonts as |font|}}
            `{{font.path}}`,
        {{~/each}} ];

        let normal_font_face_path = {{#fontFace fontSettings weight="normal"}}
            "{{~path~}}"
        {{/fontFace}}

        let code = `
        {{#each lines as |line|~}}
        {{#each line as |token|}}
        {{{~text~}}}
        {{/each}}
        {{/each}}

        `
        "#;

        handlebars.register_helper("fontFace", Box::new(get_font_face_helper));

        handlebars.register_template_string("t1", template).unwrap();

        let result = render_template(
            &font_db,
            &handlebars,
            &theme_set,
            &syntax_set,
            "t1".to_string(),
            TemplateUserData {
                font_family: font_db.family_name(&fontdb::Family::Monospace).to_string(),
                font_size: 10.0,
                theme_name: "base16-mocha.dark".to_string(),
                syntax_name: "Rust".to_string(),
                code: "fn main() {\n    println!(\"Hello, world!\");\n}".to_string(),
                props: HashMap::new(),
            },
        )
        .expect("Could not render template");

        info!("{result}");
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
