mod application;
mod dir;
mod generator;
mod ui;
mod window;

use std::path::PathBuf;
use application::QuellcodeApplication;

pub use window::Window;

use quellcode::ThemeFormat;

pub const APP_ID: &str = "org.quellcode.Quellcode";

pub fn new() -> QuellcodeApplication {
    QuellcodeApplication::new(APP_ID)
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
