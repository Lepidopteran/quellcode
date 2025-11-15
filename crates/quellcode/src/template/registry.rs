use color_eyre::eyre::{OptionExt, Result};
use fontdb::Source as FontSource;
use handlebars::{Handlebars, RenderErrorReason, Template as HandlebarsTemplate};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use syntect::{
    easy::HighlightLines, highlighting::ThemeSet, parsing::SyntaxSet, util::LinesWithEndings,
};
use ts_rs::TS;

use super::{FontFace, FontSettings, TemplateData, TemplateInfo, TemplateUserData, Token};

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind")]
#[ts(export, rename = "TemplateSource")]
pub enum Source {
    Path { path: PathBuf },
    Embedded,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct Template {
    pub source: Source,
    pub data: TemplateInfo,
}

impl Template {
    pub fn handlebar_template<'tpl>(
        &'tpl self,
        handlebars: &'tpl Handlebars,
    ) -> Option<&'tpl HandlebarsTemplate> {
        handlebars.get_template(&self.data.name)
    }
}

pub struct TemplateRegistry<'r> {
    templates: HashMap<String, Template>,
    handlebars: Handlebars<'r>,
}

impl<'r> TemplateRegistry<'r> {
    pub fn register_template(&mut self, source: Source, template: TemplateInfo) -> Result<()> {
        self.handlebars.register_template(
            &template.name,
            HandlebarsTemplate::compile(&template.content)?,
        );

        self.templates.insert(
            template.name.clone(),
            Template {
                source,
                data: template,
            },
        );

        Ok(())
    }

    pub fn unregister_template(&mut self, name: &str) {
        self.templates.remove(name);
        self.handlebars.unregister_template(name);
    }

    pub fn templates(&self) -> &HashMap<String, Template> {
        &self.templates
    }

    #[cfg(test)]
    pub fn handlebars_mut(&mut self) -> &mut Handlebars<'r> {
        &mut self.handlebars
    }

    pub fn render_template(
        &self,
        font_db: &fontdb::Database,
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
                    path: if let FontSource::File(path) = &f.source {
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

        let result = self.handlebars.render(
            &template_name,
            &TemplateData {
                font_settings,
                theme,
                lines,
            },
        )?;

        Ok(result)
    }
}

impl Default for TemplateRegistry<'_> {
    fn default() -> Self {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);
        handlebars.register_helper("fontFace", Box::new(super::get_font_face_helper));
        handlebars.register_helper("colorToHex", Box::new(super::hex_color_helper));
        handlebars.register_helper(
            "colorChannelToFloat",
            Box::new(super::color_channel_to_float_helper),
        );

        Self {
            handlebars,
            templates: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod test {
    use log::info;
    use test_log::test;

    use super::*;

    #[test]
    fn test_render_template() {
        let mut registry = TemplateRegistry::default();
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

        registry
            .handlebars_mut()
            .register_template_string("t1", template)
            .unwrap();

        let result = registry
            .render_template(
                &font_db,
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
}
