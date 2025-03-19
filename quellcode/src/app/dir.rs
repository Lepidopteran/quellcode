use std::path::PathBuf;

pub use directories::ProjectDirs;

pub fn get_project_dirs() -> ProjectDirs {
    ProjectDirs::from("org", "quellcode", "Quellcode").unwrap()
}

pub fn app_config_dir() -> PathBuf {
    get_project_dirs().config_dir().to_path_buf()
}

pub fn app_data_dir() -> PathBuf {
    get_project_dirs().data_dir().to_path_buf()
}

pub fn app_cache_dir() -> PathBuf {
    get_project_dirs().cache_dir().to_path_buf()
}
