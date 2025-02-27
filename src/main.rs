use std::io::Write;

use gtk::gdk::{Paintable, Texture};
use gtk::glib::value::FromValue;
use gtk::glib::Bytes;
use gtk::{glib, Application, ApplicationWindow};
use gtk::{prelude::*, Image, Picture};
use svg::node::element::{Rectangle, TSpan, Text};
use svg::Document;
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, ThemeSet};
use syntect::parsing::SyntaxSet;
use usvg::{roxmltree, WriteOptions};

const APP_ID: &str = "org.gtk_rs.HelloWorld2";

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);

    app.run()
}

struct RenderedCode {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

fn build_ui(app: &Application) {
    let RenderedCode {
        width,
        height,
        data,
    } = render_test_code();

    let bytes = Bytes::from(&data);
    let texture = Texture::from_bytes(&bytes).unwrap();
    let picture = Picture::builder().paintable(&texture).build();

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Code Preview Test")
        .default_width(width as i32)
        .default_height(height as i32)
        .child(&picture)
        .build();

    window.present();
}

fn render_test_code() -> RenderedCode {
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let syntax = syntax_set.find_syntax_by_extension("lua").unwrap();
    let text_size = 12;
    let offset = 1;
    let text = include_str!("../assets/test-code2.txt");

    let mut theme_set = ThemeSet::load_defaults();
    theme_set
        .add_from_folder("./assets/themes")
        .expect("Could not load themes");
    println!(
        "{:?}",
        theme_set
            .themes
            .iter()
            .map(|theme| theme.0)
            .collect::<Vec<_>>()
    );
    let theme = &theme_set.themes["Dracula"];
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
        let ranges = highlight.highlight_line(line, &syntax_set).unwrap();

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
    let real_tree = usvg::Tree::from_xmltree(&tree, &options).unwrap();

    let mut pixmap = resvg::tiny_skia::Pixmap::new(width as u32, height as u32).unwrap();
    resvg::render(&real_tree, usvg::Transform::default(), &mut pixmap.as_mut());

    RenderedCode {
        width: width as u32,
        height: height as u32,
        data: pixmap.encode_png().unwrap(),
    }
}
