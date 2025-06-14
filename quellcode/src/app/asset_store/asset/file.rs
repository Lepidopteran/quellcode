use gtk::glib::{self, prelude::*, subclass::prelude::*};

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FileInfoData {
    pub name: String,
    pub extension: String,
    pub sha256: String,
}

impl FileInfoData {
    pub fn new(name: String, extension: String, sha256: String) -> Self {
        Self {
            name,
            extension,
            sha256,
        }
    }
}

mod imp {
    use gtk::glib::Properties;
    use std::cell::RefCell;

    use super::*;

    #[derive(Properties, Default)]
    #[properties(wrapper_type = super::FileInfo)]
    pub struct FileInfo {
        #[property(name = "name", get, set, member = name, type = String)]
        #[property(name = "extension", get, set, member = extension, type = String)]
        #[property(name = "sha256", get, set, member = sha256, type = String)]
        pub data: RefCell<FileInfoData>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FileInfo {
        const NAME: &'static str = "QuellcodeStoreAssetFileInfo";
        type Type = super::FileInfo;
    }

    #[glib::derived_properties]
    impl ObjectImpl for FileInfo {}
}

glib::wrapper! {
    pub struct FileInfo(ObjectSubclass<imp::FileInfo>);
}

impl FileInfo {
    pub fn new(data: FileInfoData) -> Self {
        glib::Object::builder()
            .property("name", data.name)
            .property("extension", data.extension)
            .property("sha256", data.sha256)
            .build()
    }
}

impl From<FileInfoData> for FileInfo {
    fn from(data: FileInfoData) -> Self {
        FileInfo::new(data)
    }
}
