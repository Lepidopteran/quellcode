use std::{fs, io::{Read, Write}};

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
    pub core: Core,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Core {
    pub theme: String,
    pub syntax: String,
    pub font_family: String,
    pub font_size: f64,
}

impl Default for Core {
    fn default() -> Self {
        Self {
            theme: String::new(),
            syntax: String::new(),
            font_family: String::from("monospace"),
            font_size: 12.0,
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self {
            core: Core::default(),
        }
    }
}

pub fn save_config(config: &Config) -> Result<()> {
    let contents = fs::read_to_string(config_file_path())?;
    let mut doc = contents.parse::<DocumentMut>()?;

    doc["core"]["theme"] = config.core.theme.clone().into();
    doc["core"]["syntax"] = config.core.syntax.clone().into();
    doc["core"]["font_family"] = config.core.font_family.clone().into();
    doc["core"]["font_size"] = config.core.font_size.into();

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
        theme: &config.core.theme,
        syntax: &config.core.syntax,
        font_family: &config.core.font_family,
        font_size: config.core.font_size,
    };

    std::fs::write(config_file_path(), template.render()?)?;
    Ok(())
}

fn config_file_path() -> std::path::PathBuf {
    config_dir().join("config.toml")
}
