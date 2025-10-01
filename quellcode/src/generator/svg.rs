use crate::generator::GeneratorOptions;

use super::{Generator, GeneratorExt, GeneratorInfo, Property, PropertyType, PropertyValue};

use color_eyre::eyre::Result;
use svg::{
    node::element::{Rectangle, TSpan, Text},
    Document,
};

use syntect::{easy::HighlightLines, highlighting::FontStyle};

use usvg::{roxmltree, WriteOptions};

#[derive(Clone, Debug, Default)]
pub struct SvgGenerator {}

impl SvgGenerator {
    pub fn new() -> SvgGenerator {
        SvgGenerator::default()
    }
}

impl Generator for SvgGenerator {
    fn generate_code(
        &self,
        text: &str,
        theme: &syntect::highlighting::Theme,
        syntax: &syntect::parsing::SyntaxReference,
        syntax_set: &syntect::parsing::SyntaxSet,
        options: &GeneratorOptions,
    ) -> Result<String> {
        let text_size = options.font_size as usize;
        let font_family = options.font_family.as_str();
        let write_options = WriteOptions {
            preserve_text: !options
                .extra
                .get("bake_font")
                .and_then(|value| value.clone().try_into().ok())
                .unwrap_or(true),
            ..Default::default()
        };

        log::debug!(
            "Generating svg with font family {} and font size {}",
            font_family,
            text_size
        );

        let mut highlight = HighlightLines::new(syntax, theme);
        let mut document = Document::new();

        if options
            .extra
            .get("include_background")
            .and_then(|value| value.clone().try_into().ok())
            .unwrap_or(true)
        {
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
            let ranges = highlight.highlight_line(line, syntax_set)?;

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
        )?;

        let mut usvg_options = usvg::Options {
            font_size: text_size as f32,
            dpi: 96.0,
            ..usvg::Options::default()
        };

        usvg_options.fontdb_mut().load_system_fonts();

        Ok(usvg::Tree::from_xmltree(&tree, &usvg_options)
            .unwrap()
            .to_string(&write_options))
    }
}

impl GeneratorExt for SvgGenerator {
    fn information() -> GeneratorInfo {
        GeneratorInfo {
            name: "SVG",
            description: "Generates code into svg format",
            extensions: Some(vec!["svg"]),
            properties: Some(vec![
                Property {
                    name: "include_background",
                    description: "Include a background for the code",
                    kind: PropertyType::Bool,
                    default: Some(PropertyValue::Bool(true)),
                    ..Default::default()
                },
                Property {
                    name: "bake_font",
                    description: "Whether to convert the font to points",
                    kind: PropertyType::Bool,
                    default: Some(PropertyValue::Bool(true)),
                    ..Default::default()
                },
            ]),
            syntax: Some("XML"),
            saveable: true,
        }
    }
}
