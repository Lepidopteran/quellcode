#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub extension: String,
    pub sha256: String,
}

impl FileInfo {
    pub fn new(name: String, extension: String, sha256: String) -> Self {
        Self {
            name,
            extension,
            sha256,
        }
    }
}
