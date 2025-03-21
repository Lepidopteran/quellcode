pub mod generating;

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
