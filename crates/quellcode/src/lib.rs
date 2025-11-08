use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{mpsc::channel, Arc, Mutex},
};

use color_eyre::{eyre::Result, owo_colors::OwoColorize};
use handlebars::Handlebars;
use log::{debug, warn};
use secrecy::SecretString;
use serde::Serialize;
use syntect::{
    highlighting::{Theme as SnytectTheme, ThemeSet},
    html::{css_for_theme_with_class_style, ClassedHTMLGenerator},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};
use tauri::{Emitter, Manager, State};
use tauri_plugin_fs::FsExt;
use tauri_plugin_log::fern::colors::{self, ColoredLevelConfig};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use ts_rs::TS;

use crate::{
    dir::{cache_dir, config_dir, templates_dir},
    generator::{
        FusionGenerator, Generator, GeneratorContext, GeneratorExt, GeneratorInfo, SvgGenerator,
    },
    template::{TemplateInfo, TemplateUserData},
};

mod app;
pub mod asset_store;
pub mod dir;
pub mod generator;
pub mod property;
pub mod scraping;
mod settings;
mod template;
pub mod util;

use app::generate_code;

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
    theme_files: HashMap<PathBuf, ThemeFormat>,
    syntect_themes: ThemeSet,
    syntect_syntaxes: SyntaxSet,
    generators: Vec<(GeneratorInfo, Arc<dyn Generator>)>,
    generator_context: GeneratorContext,
    template_files: HashMap<PathBuf, TemplateInfo>,
    handlebars: Handlebars<'static>,
    font_db: fontdb::Database,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
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
            generate_code,
            generate_html,
            font_families,
            theme_files,
            generators,
            syntaxes,
            themes,
            templates,
            add_template,
            remove_template,
            render_template,
        ])
        .setup(|app| {
            let syntax_set = SyntaxSet::load_defaults_nonewlines();
            let scope = app.fs_scope();

            let config_dir = config_dir(app.app_handle());
            let cache_dir = cache_dir(app.app_handle());

            let _ = scope.allow_directory(&config_dir, true);
            let _ = scope.allow_directory(&cache_dir, true);

            for path in [
                dir::code_theme_dir(app.app_handle()),
                dir::code_syntax_dir(app.app_handle()),
                dir::templates_dir(app.app_handle()),
                cache_dir,
                config_dir,
            ] {
                if !path.exists() {
                    std::fs::create_dir_all(&path).expect("Failed to ensure directory exists");
                }
            }

            let generators: Vec<(GeneratorInfo, Arc<dyn Generator>)> = vec![
                (
                    FusionGenerator::information(),
                    Arc::new(FusionGenerator::new()),
                ),
                (SvgGenerator::information(), Arc::new(SvgGenerator::new())),
            ];

            let theme_files = code_theme_files(app.app_handle());

            let (tx, rx) = channel();
            let app_handle = app.app_handle().clone();

            std::thread::spawn(move || {
                while let Ok(event) = rx.recv() {
                    debug!("Generator event: {event:?}");
                    let _ = app_handle.emit("generator-event", event);
                }
            });

            let template_files = template_files(app.app_handle());
            let handlebars = setup_handlebars(&template_files);

            let mut fontdb = fontdb::Database::new();
            fontdb.load_system_fonts();

            app.manage(Mutex::new(AppState {
                syntect_themes: load_themes(&theme_files),
                syntect_syntaxes: syntax_set,
                theme_files,
                generators,
                generator_context: GeneratorContext::new(tx.clone()),
                template_files,
                handlebars,
                font_db: fontdb,
            }));

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_handlebars(files: &HashMap<PathBuf, TemplateInfo>) -> Handlebars<'static> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(true);

    handlebars.register_helper(
        "fontFace",
        Box::new(template::get_font_face_helper),
    );

    for (_, template) in files.iter() {
        handlebars
            .register_template_string(&template.name, &template.content)
            .unwrap();
    }

    handlebars
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

                    let theme =
                        SnytectTheme::try_from(vscode_theme).expect("Failed to parse theme");

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

                    let theme =
                        SnytectTheme::try_from(color_scheme).expect("Failed to parse theme");

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
fn render_template(
    state: State<Mutex<AppState>>,
    template_name: String,
    data: TemplateUserData,
) -> Result<String, String> {
    let state = state.lock().expect("Failed to lock state");
    let handlebars = &state.handlebars;
    let font_db = &state.font_db;
    let theme_set = &state.syntect_themes;

    template::render_template(font_db, handlebars, theme_set, template_name, data)
        .map_err(|e| format!("Failed to render template: {e}"))
}

#[tauri::command]
fn generators(state: State<Mutex<AppState>>) -> Vec<GeneratorInfo> {
    let mut generators = state
        .lock()
        .expect("Failed to lock state")
        .generators
        .iter()
        .map(|(info, _)| info.clone())
        .collect::<Vec<_>>();

    generators.sort_by(|a, b| a.name().cmp(b.name()));
    generators
}

#[tauri::command]
fn generate_html(
    state: State<Mutex<AppState>>,
    code: String,
    syntax: String,
) -> Result<String, String> {
    let syntax_set = &state.lock().expect("Failed to lock state").syntect_syntaxes;

    let syntax = syntax_set
        .find_syntax_by_name(&syntax)
        .ok_or(format!("Could not find syntax \"{}\"", syntax))?;

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

    Ok(generator.finalize())
}

#[tauri::command]
fn add_template(app: tauri::AppHandle, state: State<Mutex<AppState>>, template: TemplateInfo) {
    state
        .lock()
        .expect("Failed to lock state")
        .handlebars
        .register_template_string(&template.name, &template.content)
        .unwrap();

    state
        .lock()
        .expect("Failed to lock state")
        .template_files
        .insert(
            templates_dir(&app).join(format!("{}.json", template.name)),
            template,
        );
}

#[tauri::command]
fn remove_template(app: tauri::AppHandle, state: State<Mutex<AppState>>, name: String) {
    state
        .lock()
        .expect("Failed to lock state")
        .template_files
        .remove(&templates_dir(&app).join(format!("{}.json", name)));

    state
        .lock()
        .expect("Failed to lock state")
        .handlebars
        .unregister_template(&name);
}

#[tauri::command]
fn templates(state: State<Mutex<AppState>>) -> Vec<TemplateInfo> {
    state
        .lock()
        .expect("Failed to lock state")
        .template_files
        .values()
        .cloned()
        .collect()
}

#[tauri::command]
fn theme_files(state: State<'_, Mutex<AppState>>) -> HashMap<PathBuf, ThemeFormat> {
    state
        .lock()
        .expect("Failed to lock state")
        .theme_files
        .clone()
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
fn font_families(state: State<Mutex<AppState>>) -> Vec<FontFamily> {
    let db = &state.lock().expect("Failed to lock state").font_db;

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

pub fn template_files(app_handle: &tauri::AppHandle) -> HashMap<PathBuf, TemplateInfo> {
    let templates_dir = templates_dir(app_handle);

    templates_dir
        .read_dir()
        .expect("Failed to read templates dir")
        .filter_map(|entry| {
            entry.ok().and_then(|entry| {
                let path = entry.path();
                if path.is_file() {
                    serde_json::from_str::<TemplateInfo>(
                        &std::fs::read_to_string(&path).expect("Failed to read template file"),
                    )
                    .ok()
                    .map(|template| (path, template))
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
