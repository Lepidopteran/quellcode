use color_eyre::eyre::Result;
use log::warn;
use secrecy::SecretString;

pub mod asset_store;
pub mod generating;
pub mod generator;
pub mod property;
pub mod scraping;
pub mod util;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![font_families])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn font_families() -> Vec<String> {
    let mut db = usvg::fontdb::Database::new();
    db.load_system_fonts();

    let mut families = Vec::new();

    for face in db.faces() {
        if let Some((family, _)) = face.families.first() {
            if families.contains(family) {
                continue;
            }

            families.push(family.to_string());
        }
    }

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

// pub fn code_theme_files() -> Vec<(ThemeFormat, PathBuf)> {
//     let themes_dir = dir::code_theme_dir();
//
//     themes_dir
//         .read_dir()
//         .expect("Failed to read themes dir")
//         .filter_map(|entry| {
//             entry.ok().and_then(|entry| {
//                 let path = entry.path();
//                 if path.is_file() {
//                     ThemeFormat::from_path(&path).map(|format| (format, path))
//                 } else {
//                     None
//                 }
//             })
//         })
//         .collect()
// }

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
