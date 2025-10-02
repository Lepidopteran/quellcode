use directories::ProjectDirs;
use std::path::PathBuf;
use tauri::Manager;

pub fn project_dirs(app_handle: &tauri::AppHandle) -> ProjectDirs {
    let mut config = app_handle.config().identifier.split('.');
    ProjectDirs::from(
        config.next().expect("Failed to get organization"),
        config.next().expect("Failed to get domain"),
        config.next().expect("Failed to get app name"),
    )
    .unwrap()
}

pub fn data_dir(app_handle: &tauri::AppHandle) -> PathBuf {
    app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data dir")
        .to_path_buf()
}

pub fn config_dir(app_handle: &tauri::AppHandle) -> PathBuf {
    project_dirs(app_handle).config_dir().to_path_buf()
}

pub fn cache_dir(app_handle: &tauri::AppHandle) -> PathBuf {
    project_dirs(app_handle).cache_dir().to_path_buf()
}

pub fn code_theme_dir(app_handle: &tauri::AppHandle) -> PathBuf {
    data_dir(app_handle).join("themes")
}

pub fn code_syntax_dir(app_handle: &tauri::AppHandle) -> PathBuf {
    data_dir(app_handle).join("syntaxes")
}

pub fn store_cache_dir(app_handle: &tauri::AppHandle) -> PathBuf {
    cache_dir(app_handle).join("asset_store")
}
