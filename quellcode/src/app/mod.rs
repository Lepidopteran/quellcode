mod application;
mod asset_store;
mod dir;
mod generator;
mod property;
mod scraping;
mod settings;
mod ui;
mod window;
mod util;

pub mod state;

use application::QuellcodeApplication;
use color_eyre::eyre::Result;
use log::{debug, warn};
use secrecy::SecretString;
use std::{path::PathBuf, sync::OnceLock};
use tokio::runtime::Runtime;

pub use window::Window;

use quellcode::ThemeFormat;
pub const APP_ID: &str = "org.quellcode.Quellcode";

pub fn new() -> QuellcodeApplication {
    QuellcodeApplication::new(APP_ID)
}

pub fn tokio_runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| Runtime::new().expect("Setting up tokio runtime needs to succeed."))
}

pub fn github_token() -> Result<Option<SecretString>> {
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        warn!("Using Github Token from environment variable, if you are a developer you can ignore this warning");
        return Ok(Some(SecretString::from(token)));
    }

    match keyring::Entry::new("quellcode", "github_token") {
        Ok(entry) => {
            match entry.get_password() {
                Ok(token) => Ok(Some(SecretString::from(token))),
                Err(err) => {
                    if err.to_string() != *"No matching entry found in secure storage" {
                        Err(err.into())
                    } else {
                        Ok(None)
                    }
                }
            }
        }
        Err(err) => Err(err.into()),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    pub fn init() {
        dotenvy::dotenv().ok();
        let _ = env_logger::builder().is_test(true).try_init();
        let _ = color_eyre::install();
    }

    #[test]
    fn fetch_github_token() {
        init();

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
