use std::{fs, io::Write};

use askama::Template;
use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};
use toml_edit::DocumentMut;

use super::dir::config_dir;

#[derive(Template)]
#[template(path = "config.toml", escape = "none")]
struct ConfigTemplate<'a> {
    theme: &'a str,
    syntax: &'a str,
    font_family: &'a str,
    font_size: f64,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    pub code: CodeSettings,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CodeSettings {
    pub theme: String,
    pub syntax: String,
    pub font_family: String,
    pub font_size: f64,
}

impl Default for CodeSettings {
    fn default() -> Self {
        Self {
            theme: String::new(),
            syntax: String::new(),
            font_family: String::from("Monospace"),
            font_size: 12.0,
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self {
            code: CodeSettings::default(),
        }
    }
}

pub fn save_config(config: &Config) -> Result<()> {
    let contents = fs::read_to_string(config_file_path())?;
    let mut doc = contents.parse::<DocumentMut>()?;

    doc["code"]["theme"] = config.code.theme.clone().into();
    doc["code"]["syntax"] = config.code.syntax.clone().into();
    doc["code"]["font_family"] = config.code.font_family.clone().into();
    doc["code"]["font_size"] = config.code.font_size.into();

    let mut file = fs::File::create(config_file_path())?;
    file.write_all(doc.to_string().as_bytes())?;
    Ok(())
}

pub fn load_config() -> Result<Config> {
    let file = fs::read_to_string(config_file_path())?;
    Ok(toml::from_str(&file)?)
}

pub fn write_default_config_file(config: &Config) -> Result<()> {
    let template = ConfigTemplate {
        theme: &config.code.theme,
        syntax: &config.code.syntax,
        font_family: &config.code.font_family,
        font_size: config.code.font_size,
    };

    std::fs::write(config_file_path(), template.render()?)?;
    Ok(())
}

fn config_file_path() -> std::path::PathBuf {
    config_dir().join("config.toml")
}
