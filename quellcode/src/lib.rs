use svg::node::element::{Rectangle, TSpan, Text};
use svg::Document;
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, Theme};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use usvg::roxmltree;

pub mod escape;

pub fn generate_highlighted_code_svg(
    text: &str,
    syntax_set: &SyntaxSet,
    syntax: &SyntaxReference,
    theme: &Theme,
) -> usvg::Tree {
    let text_size = 12;
    let offset = 1;

    let mut highlight = HighlightLines::new(syntax, theme);
    let mut document = Document::new();

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

    for (index, line) in text.lines().enumerate() {
        let ranges = highlight.highlight_line(line, syntax_set).unwrap();

        let mut text_element = Text::new("")
            .set("font-family", "JetBrains Mono")
            .set("font-size", format!("{text_size}"))
            .set("font-weight", "normal")
            .set("y", ((index + offset) * text_size).to_string());

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
    let mut options = usvg::Options {
        font_size: text_size as f32,
        dpi: 96.0,
        ..usvg::Options::default()
    };

    options.fontdb_mut().load_system_fonts();

    let document = document.to_string().replace("\n", "");
    let tree = roxmltree::Document::parse_with_options(
        &document,
        roxmltree::ParsingOptions {
            allow_dtd: true,
            ..Default::default()
        },
    )
    .unwrap();

    usvg::Tree::from_xmltree(&tree, &options).unwrap()
}
