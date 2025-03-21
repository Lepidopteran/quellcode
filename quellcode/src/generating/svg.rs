use std::collections::HashMap;

use svg::{
    node::element::{Rectangle, TSpan, Text},
    Document,
};
use syntect::{easy::HighlightLines, highlighting::FontStyle};
use usvg::{roxmltree, WriteOptions};

use super::{Generator, Properties, PropertyType, RenderType};

pub struct SvgGenerator {
    write_options: WriteOptions,
    properties: Properties,
    font_size: f32,
    font_family: String,
    include_background: bool,
    padding: f32,
}

impl Default for SvgGenerator {
    fn default() -> Self {
        Self {
            properties: HashMap::from([
                ("include_background", PropertyType::Bool),
                ("padding", PropertyType::Float),
                ("font_family", PropertyType::String),
                ("font_size", PropertyType::Float),
                ("line_height", PropertyType::Float),
                ("bake_fonts", PropertyType::Bool),
            ]),
            write_options: WriteOptions::default(),
            font_size: 12.0,
            padding: 0.0,
            font_family: "monospace".to_string(),
            include_background: true,
        }
    }
}

impl Generator for SvgGenerator {
    fn name(&self) -> &str {
        "svg"
    }

    fn kind(&self) -> &RenderType {
        &RenderType::Both
    }

    fn description(&self) -> &str {
        "Generate's svg files to view code in vector programs"
    }

    fn saveable(&self) -> &bool {
        &true
    }

    fn properties(&self) -> &Properties {
        &self.properties
    }

    fn get_property(&self, name: &str) -> Result<super::PropertyValue, super::GeneratorError> {
        match name {
            "include_background" => Ok(super::PropertyValue::Bool(self.include_background)),
            "padding" => Ok(super::PropertyValue::Float(self.padding)),
            "font_family" => Ok(super::PropertyValue::String(self.font_family.clone())),
            "font_size" => Ok(super::PropertyValue::Float(self.font_size)),
            "bake_fonts" => Ok(super::PropertyValue::Bool(self.write_options.preserve_text)),
            _ => Err(super::GeneratorError::PropertyError(
                super::PropertyError::UnknownProperty,
            )),
        }
    }

    fn set_property<T: Into<super::PropertyValue>>(
        &mut self,
        name: &str,
        value: T,
    ) -> Result<(), super::GeneratorError> {
        match name {
            "include_background" => match value.into() {
                super::PropertyValue::Bool(v) => self.include_background = v,
                _ => {
                    return Err(super::GeneratorError::PropertyError(
                        super::PropertyError::InvalidValueType,
                    ))
                }
            },
            "padding" => match value.into() {
                super::PropertyValue::Float(v) => self.padding = v,
                _ => {
                    return Err(super::GeneratorError::PropertyError(
                        super::PropertyError::InvalidValueType,
                    ))
                }
            },
            "font_family" => match value.into() {
                super::PropertyValue::String(v) => self.font_family = v,
                _ => {
                    return Err(super::GeneratorError::PropertyError(
                        super::PropertyError::InvalidValueType,
                    ))
                }
            },
            "font_size" => match value.into() {
                super::PropertyValue::Float(v) => self.font_size = v,
                _ => {
                    return Err(super::GeneratorError::PropertyError(
                        super::PropertyError::InvalidValueType,
                    ))
                }
            },
            "bake_fonts" => match value.into() {
                super::PropertyValue::Bool(v) => self.write_options.preserve_text = v,
                _ => {
                    return Err(super::GeneratorError::PropertyError(
                        super::PropertyError::InvalidValueType,
                    ))
                }
            },
            _ => {
                return Err(super::GeneratorError::PropertyError(
                    super::PropertyError::UnknownProperty,
                ))
            }
        }

        Ok(())
    }

    fn generate(
        &self,
        text: &str,
        theme: &syntect::highlighting::Theme,
        syntax: &syntect::parsing::SyntaxReference,
        syntax_set: &syntect::parsing::SyntaxSet,
    ) -> Result<super::RenderOutput, super::GeneratorError> {
        let text_size = self.font_size as usize;
        let font_family = self.font_family.as_str();

        let mut highlight = HighlightLines::new(syntax, theme);
        let mut document = Document::new();

        if self.include_background {
            let background = theme
                .settings
                .background
                .unwrap_or(syntect::highlighting::Color::WHITE);
            let background_element = Rectangle::new()
                .set("width", "100%")
                .set("height", "100%")
                .set(
                    "fill",
                    format!(
                        "#{:02x}{:02x}{:02x}",
                        background.r, background.g, background.b
                    ),
                );

            document = document.add(background_element);
        }

        for (index, line) in text.lines().enumerate() {
            let ranges = highlight.highlight_line(line, syntax_set).unwrap();

            let mut text_element = Text::new("")
                .set("font-family", font_family)
                .set("font-size", format!("{text_size}px"))
                .set("font-weight", "normal")
                .set("y", ((index + 1) * text_size).to_string());

            for &(ref style, text) in ranges.iter() {
                let mut tspan = TSpan::new(text.replace('\t', "")).set("xml:space", "preserve");

                tspan = tspan.set(
                    "fill",
                    format!(
                        "#{:02x}{:02x}{:02x}",
                        style.foreground.r, style.foreground.g, style.foreground.b
                    ),
                );

                if style.font_style.contains(FontStyle::BOLD) {
                    tspan = tspan.set("font-weight", "bold");
                }

                text_element = text_element.add(tspan);
            }

            let tabs = line.match_indices('\t').count();
            if tabs > 0 {
                text_element = text_element.set("x", (tabs * text_size).to_string());
            }

            document = document.add(text_element);
        }

        let height = text.lines().count() * text_size;
        let width = text.lines().map(|line| line.len()).max().unwrap_or(0) * text_size;

        document = document.set("viewBox", format!("0 0 {} {}", width, height));

        let document = document.to_string().replace("\n", "");
        let tree = roxmltree::Document::parse_with_options(
            &document,
            roxmltree::ParsingOptions {
                allow_dtd: true,
                ..Default::default()
            },
        )
        .unwrap();

        let mut options = usvg::Options {
            font_size: text_size as f32,
            dpi: 96.0,
            ..usvg::Options::default()
        };

        options.fontdb_mut().load_system_fonts();

        Ok(super::RenderOutput::Both(
            usvg::Tree::from_xmltree(&tree, &options)
                .unwrap()
                .to_string(&self.write_options),
            None,
        ))
    }
}
