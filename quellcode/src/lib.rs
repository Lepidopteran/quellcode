use std::{collections::HashMap, path::PathBuf, sync::Mutex};

use color_eyre::{eyre::Result, owo_colors::OwoColorize};
use log::warn;
use secrecy::SecretString;
use serde::Serialize;
use syntect::{
    highlighting::{Theme, ThemeSet},
    html::{css_for_theme_with_class_style, ClassedHTMLGenerator},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};
use tauri::{Manager, State};
use tauri_plugin_log::fern::colors::{self, ColoredLevelConfig};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use ts_rs::TS;

pub mod asset_store;
pub mod dir;
pub mod generating;
pub mod generator;
pub mod property;
pub mod scraping;
pub mod util;

pub const SYNTECT_PREFIX: &str = "syntect-";

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum ThemeFormat {
    Sublime,
    TmTheme,
    VsCode,
}

impl ThemeFormat {
    pub fn from_extension(ext: &str) -> Option<ThemeFormat> {
        match ext {
            "sublime-color-scheme" => Some(ThemeFormat::Sublime),
            "tmTheme" => Some(ThemeFormat::TmTheme),
            "json" => Some(ThemeFormat::VsCode),
            _ => None,
        }
    }

    pub fn from_path(path: &std::path::Path) -> Option<ThemeFormat> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(ThemeFormat::from_extension)
    }
}

pub struct AppState {
    pub theme_files: HashMap<PathBuf, ThemeFormat>,
    pub syntect_themes: ThemeSet,
    pub syntect_syntaxes: SyntaxSet,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Warn)
                .level_for("quellcode_lib", log::LevelFilter::Debug)
                .level_for("quellcode", log::LevelFilter::Debug)
                .format(|out, message, record| {
                    let mut colors = ColoredLevelConfig::new();
                    colors.debug = colors::Color::Blue;
                    colors.info = colors::Color::Green;

                    out.finish(format_args!(
                        "{} {:>5} {} {message}",
                        OffsetDateTime::now_utc().format(&Rfc3339).unwrap().dimmed(),
                        colors.color(record.level()),
                        format_args!("{}:", record.target()).dimmed(),
                    ))
                })
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_css_for_theme,
            generate_html,
            font_families,
            theme_files,
            syntaxes,
            themes
        ])
        .setup(|app| {
            let syntax_set = SyntaxSet::load_defaults_nonewlines();

            for path in [
                dir::code_theme_dir(app.app_handle()),
                dir::code_syntax_dir(app.app_handle()),
                dir::config_dir(app.app_handle()),
            ] {
                if !path.exists() {
                    std::fs::create_dir_all(&path).expect("Failed to ensure directory exists");
                }
            }

            let theme_files = code_theme_files(app.app_handle());
            app.manage(Mutex::new(AppState {
                syntect_themes: load_themes(&theme_files),
                syntect_syntaxes: syntax_set,
                theme_files,
            }));

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn load_themes(theme_files: &HashMap<PathBuf, ThemeFormat>) -> ThemeSet {
    let mut theme_set = ThemeSet::load_defaults();

    for (path, format) in theme_files.iter() {
        match format {
            ThemeFormat::VsCode => {
                let vscode_theme = syntect_vscode::parse_vscode_theme_file(path);

                if let Ok(vscode_theme) = vscode_theme {
                    let theme_name = vscode_theme
                        .name
                        .clone()
                        .unwrap_or(path.file_stem().unwrap().to_string_lossy().to_string());

                    let theme = Theme::try_from(vscode_theme).expect("Failed to parse theme");

                    theme_set.themes.insert(theme_name, theme);
                }
            }
            ThemeFormat::Sublime => {
                let color_scheme = sublime_color_scheme::parse_color_scheme_file(path);

                if let Ok(color_scheme) = color_scheme {
                    let theme_name = color_scheme
                        .name
                        .clone()
                        .unwrap_or(path.file_stem().unwrap().to_string_lossy().to_string());

                    let theme = Theme::try_from(color_scheme).expect("Failed to parse theme");

                    theme_set.themes.insert(theme_name, theme);
                }
            }
            ThemeFormat::TmTheme => {
                let theme = ThemeSet::get_theme(path);
                if let Ok(theme) = theme {
                    let theme_name = theme
                        .clone()
                        .name
                        .unwrap_or(path.file_stem().unwrap().to_string_lossy().to_string());

                    theme_set.themes.insert(theme_name, theme);
                }
            }
        }
    }

    theme_set
}

#[tauri::command]
fn syntaxes(state: State<Mutex<AppState>>) -> Vec<String> {
    state
        .lock()
        .expect("Failed to lock state")
        .syntect_syntaxes
        .syntaxes()
        .iter()
        .map(|s| s.name.to_string())
        .collect()
}

#[tauri::command]
fn get_css_for_theme(state: State<Mutex<AppState>>, theme: String) -> String {
    let theme_set = &state.lock().expect("Failed to lock state").syntect_themes;
    let theme = theme_set.themes.get(&theme).expect("Failed to get theme");

    css_for_theme_with_class_style(
        theme,
        syntect::html::ClassStyle::SpacedPrefixed {
            prefix: SYNTECT_PREFIX,
        },
    )
    .expect("Failed to generate css")
}

#[tauri::command]
fn themes(state: State<Mutex<AppState>>) -> Vec<String> {
    state
        .lock()
        .expect("Failed to lock state")
        .syntect_themes
        .themes
        .iter()
        .map(|t| t.0.to_string())
        .collect()
}

#[derive(Debug, TS, Serialize, Clone)]
#[ts(export)]
pub struct FontFamily {
    pub name: String,
    pub monospace: bool,
}

#[tauri::command]
fn font_families() -> Vec<FontFamily> {
    let mut db = usvg::fontdb::Database::new();
    db.load_system_fonts();

    let mut families: Vec<FontFamily> = Vec::new();

    for face in db.faces() {
        if let Some((family, _)) = face.families.first() {
            if families.iter().any(|f| f.name.as_str() == family) {
                continue;
            }

            families.push(FontFamily {
                name: family.to_string(),
                monospace: face.monospaced,
            });
        }
    }

    log::info!("Found {} font families", families.len());

    families
}

pub fn github_token() -> Result<Option<SecretString>> {
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        warn!("Using Github Token from environment variable, if you are a developer you can ignore this warning");
        return Ok(Some(SecretString::from(token)));
    }

    match keyring::Entry::new("quellcode", "github_token") {
        Ok(entry) => match entry.get_password() {
            Ok(token) => Ok(Some(SecretString::from(token))),
            Err(err) => {
                if err.to_string() != *"No matching entry found in secure storage" {
                    Err(err.into())
                } else {
                    Ok(None)
                }
            }
        },
        Err(err) => Err(err.into()),
    }
}

#[tauri::command]
fn generate_html(state: State<Mutex<AppState>>, code: String, syntax: String) -> String {
    let syntax_set = &state.lock().expect("Failed to lock state").syntect_syntaxes;

    let syntax = syntax_set
        .find_syntax_by_name(&syntax)
        .expect("Failed to get syntax");

    let mut generator = ClassedHTMLGenerator::new_with_class_style(
        syntax,
        syntax_set,
        syntect::html::ClassStyle::SpacedPrefixed {
            prefix: SYNTECT_PREFIX,
        },
    );

    for line in LinesWithEndings::from(code.as_str()) {
        let _ = generator.parse_html_for_line_which_includes_newline(line);
    }

    generator.finalize()
}

#[tauri::command]
fn theme_files(state: State<'_, Mutex<AppState>>) -> HashMap<PathBuf, ThemeFormat> {
    state
        .lock()
        .expect("Failed to lock state")
        .theme_files
        .clone()
}

pub fn code_theme_files(app_handle: &tauri::AppHandle) -> HashMap<PathBuf, ThemeFormat> {
    let themes_dir = dir::code_theme_dir(app_handle);

    themes_dir
        .read_dir()
        .expect("Failed to read themes dir")
        .filter_map(|entry| {
            entry.ok().and_then(|entry| {
                let path = entry.path();
                if path.is_file() {
                    ThemeFormat::from_path(&path).map(|format| (path, format))
                } else {
                    None
                }
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::{debug, warn};

    #[test_log::test]
    fn fetch_github_token() {
        if std::env::var("CI").is_ok() {
            debug!("Skipping Github token fetch in CI");
            return;
        }

        let token = github_token().unwrap();

        if token.is_none() {
            warn!("No Github token found");
        }
    }
}
