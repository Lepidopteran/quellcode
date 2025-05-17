use svg::{
    node::element::{Rectangle, TSpan, Text},
    Document,
};
use syntect::{
    easy::HighlightLines,
    highlighting::{FontStyle, Theme},
    parsing::{SyntaxReference, SyntaxSet},
};
use thiserror::Error;
use usvg::{roxmltree, WriteOptions};

#[derive(Debug, Error)]
pub enum SvgGeneratorError {
    #[error("Highlight error: {0}")]
    HighlightError(#[from] syntect::Error),
    #[error("Parse XML error: {0}")]
    ParseXmlError(#[from] roxmltree::Error),
}

#[derive(Debug, Clone)]
pub struct SvgOptions {
    pub write_options: WriteOptions,
    pub font_size: f32,
    pub line_height: f32,
    pub padding: f32,
    pub font_family: String,
    pub include_background: bool,
}

impl Default for SvgOptions {
    fn default() -> Self {
        Self {
            write_options: WriteOptions::default(),
            font_size: 12.0,
            line_height: 0.0,
            padding: 0.0,
            font_family: "monospace".to_string(),
            include_background: true,
        }
    }
}

pub fn generate_svg(
    text: &str,
    theme: &Theme,
    syntax: &SyntaxReference,
    syntax_set: &SyntaxSet,
    options: &SvgOptions,
) -> Result<String, SvgGeneratorError> {
    let text_size = options.font_size as usize;
    let font_family = options.font_family.as_str();

    let mut highlight = HighlightLines::new(syntax, theme);
    let mut document = Document::new();

    if options.include_background {
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

        let mut clean_font_family = font_family.to_string();
        if clean_font_family
            .chars()
            .any(|char| !char.is_ascii_alphanumeric() && char != '_' && char != '-')
        {
            clean_font_family = format!("'{}'", clean_font_family);
        }

        let mut text_element = Text::new("")
            .set("font-family", clean_font_family)
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

    let height = text.lines().count() * (text_size + options.line_height as usize) + options.padding as usize;
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
        .to_string(&options.write_options))
}
