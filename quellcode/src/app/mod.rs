use std::{cell::RefCell, path::PathBuf, rc::Rc};

use application::QuellcodeApplication;
use gtk::{
    gio,
    glib::{self, closure_local, property::PropertySet},
    prelude::*,
    subclass::prelude::ObjectSubclassIsExt,
    DropDown, StringList,
};
use syntect::parsing::SyntaxSet;
mod application;
mod dir;
mod ui;
mod window;
use quellcode::{
    generating::svg::{generate_svg, SvgOptions},
    ThemeFormat,
};

pub const APP_ID: &str = "org.quellcode.Quellcode";

pub fn new() -> QuellcodeApplication {
    let app = QuellcodeApplication::new(APP_ID);

    app.connect_activate(|app| {
        build_ui(app);
    });

    app
}

pub fn code_theme_files() -> Vec<(ThemeFormat, PathBuf)> {
    let themes_dir = dir::code_theme_dir();

    themes_dir
        .read_dir()
        .expect("Failed to read themes dir")
        .filter_map(|entry| {
            entry.ok().and_then(|entry| {
                let path = entry.path();
                if path.is_file() {
                    ThemeFormat::from_path(&path).map(|format| (format, path))
                } else {
                    None
                }
            })
        })
        .collect()
}

pub fn build_ui(app: &QuellcodeApplication) {
    let window = window::Window::new(app);
    let theme_set = app.theme_set();
    let themes = StringList::new(
        &theme_set
            .themes
            .iter()
            .map(|t| t.0.as_str())
            .collect::<Vec<_>>(),
    );

    let inspector = window.inspector();
    let theme_dropdown = DropDown::builder().model(&themes).build();
    theme_dropdown.connect_selected_notify(glib::clone!(
        #[weak]
        app,
        move |dropdown| {
            app.set_code_theme(
                themes
                    .string(dropdown.selected())
                    .expect("Failed to get string"),
            )
        }
    ));

    inspector.append(&theme_dropdown);

    let syntax_set = app.syntax_set();
    let syntaxes = StringList::new(
        &syntax_set
            .syntaxes()
            .iter()
            .map(|s| s.name.as_str())
            .collect::<Vec<_>>(),
    );

    let syntax_dropdown = DropDown::builder().model(&syntaxes).build();
    syntax_dropdown.connect_selected_notify(glib::clone!(
        #[weak]
        app,
        move |dropdown| {
            app.set_code_syntax(
                syntaxes
                    .string(dropdown.selected())
                    .expect("Failed to get string"),
            )
        }
    ));

    inspector.append(&syntax_dropdown);

    let editor = window.editor().clone();
    let viewer = window.imp().viewer.clone();
    let syntax_set = app.syntax_set();
    editor.set_syntax(syntax_set.find_syntax_by_name("Rust").cloned());
    app.connect_closure(
        "theme-changed",
        false,
        closure_local!(
            move |app: &QuellcodeApplication, old_theme: &str, new_theme: &str| {
                if old_theme == new_theme {
                    return;
                }

                let theme_set = app.theme_set();
                if let Some(theme) = theme_set.themes.get(new_theme) {
                    editor.set_theme(Some(theme.clone()));
                    viewer.set_theme(Some(theme.clone()));
                }
            }
        ),
    );

    let editor = window.editor().clone();
    app.connect_code_syntax_notify(move |app| {
        if let Some(syntax) = app.syntax_set().find_syntax_by_name(&app.code_syntax()) {
            editor.set_syntax(Some(syntax.clone()));
        }
    });

    let viewer = window.imp().viewer.clone();
    viewer.set_syntax(syntax_set.find_syntax_by_name("XML").cloned());
    let gen = SvgOptions::default();

    let editor = window.editor().clone();
    let (sender, receiver) = async_channel::bounded(1);

    editor.buffer().connect_changed(move |buffer| {
        let syntax = editor.syntax().clone();
        let theme = editor.theme().clone();
        let buffer = buffer.clone();
        let sender = sender.clone();

        if let (Some(theme_syntax), Some(editor_syntax)) = (theme, syntax) {
            let text = buffer
                .text(&buffer.start_iter(), &buffer.end_iter(), true)
                .to_string();

            viewer.set_opacity(0.75);
            let syntax_set: SyntaxSet = editor.syntax_set().clone();
            let gen_clone = gen.clone();
            gio::spawn_blocking(move || {
                let generated_svg = generate_svg(
                    &text,
                    &theme_syntax,
                    &editor_syntax,
                    &syntax_set,
                    &gen_clone,
                );

                sender
                    .send_blocking(generated_svg)
                    .expect("Failed to send svg");
            });
        }
    });

    let viewer = window.imp().viewer.clone();
    glib::spawn_future_local(async move {
        while let Ok(svg) = receiver.recv().await {
            if let Ok(svg) = svg {
                viewer.set_opacity(1.0);
                viewer.buffer().set_text(&svg);
            }
        }
    });

    window.present();
}
