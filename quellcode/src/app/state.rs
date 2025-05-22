use std::{fs, io::Write};

use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};

use super::dir::data_dir;

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct State {
    pub code: CodeState,
    pub window: WindowState
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CodeState {
    pub theme: Option<String>,
    pub syntax: Option<String>,
    pub font_family: Option<String>,
    pub font_size: Option<f64>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct WindowState {
    pub height: Option<i32>,
    pub width: Option<i32>,
    pub maximized: Option<bool>,
}

impl Default for CodeState {
    fn default() -> Self {
        Self {
            theme: None,
            syntax: None,
            font_family: Some("Monospace".to_string()),
            font_size: Some(12.0),
        }
    }
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }
}

pub fn save_state(state: &State) -> Result<()> {
    let mut file = fs::File::create(state_file_path())?;
    file.write_all(toml::to_string_pretty(state)?.as_bytes())?;

    Ok(())
}

pub fn load_state() -> Result<State> {
    let file = fs::read_to_string(state_file_path())?;
    let state = toml::from_str(&file)?;

    Ok(state)
}

fn state_file_path() -> std::path::PathBuf {
    data_dir().join("state.toml")
}
