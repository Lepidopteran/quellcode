use std::path::PathBuf;
use directories::ProjectDirs;

pub fn project_dirs() -> ProjectDirs {
    ProjectDirs::from("org", "quellcode", "Quellcode").unwrap()
}

pub fn config_dir() -> PathBuf {
    project_dirs().config_dir().to_path_buf()
}

pub fn data_dir() -> PathBuf {
    project_dirs().data_dir().to_path_buf()
}

pub fn cache_dir() -> PathBuf {
    project_dirs().cache_dir().to_path_buf()
}

pub fn code_theme_dir() -> PathBuf {
    data_dir().join("themes")
}

pub fn code_syntax_dir() -> PathBuf {
    data_dir().join("syntaxes")
}
