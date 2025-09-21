use gtk::glib::{self, prelude::*, subclass::prelude::*};

#[cfg(test)]
use super::init;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum ProgressMessageKind {
    Starting,
    #[default]
    Misc,
    Error,
    Warning,
    Success,
}

impl From<u8> for ProgressMessageKind {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Starting,
            1 => Self::Misc,
            2 => Self::Error,
            3 => Self::Warning,
            4 => Self::Success,
            _ => Self::Misc,
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct ProgressMessage {
    pub index: u32,
    pub package_name: String,
    pub message: String,
    pub kind: ProgressMessageKind,
}

impl ProgressMessage {
    pub fn new(
        index: u32,
        package_name: String,
        message: String,
        kind: ProgressMessageKind,
    ) -> Self {
        Self {
            index,
            package_name,
            message,
            kind,
        }
    }
}

mod imp {
    use super::*;
    use gtk::glib::Properties;
    use std::cell::RefCell;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::ProgressMessageObject)]
    pub struct ProgressMessageObject {
        #[property(name = "package-name", get, set, member = package_name, type = String)]
        #[property(name = "message", get, set, member = message, type = String)]
        #[property(name = "index", get, set, member = index, type = u32)]
        #[property(name = "kind", get = |a: &ProgressMessageObject| a.data.borrow().kind.clone() as u8, set = |a: &ProgressMessageObject, v: u8| a.data.borrow_mut().kind = v.into(), type = u8)]
        pub data: RefCell<ProgressMessage>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProgressMessageObject {
        const NAME: &'static str = "QuellcodeStoreProgressMessage";
        type Type = super::ProgressMessageObject;
    }

    #[glib::derived_properties]
    impl ObjectImpl for ProgressMessageObject {}
}

glib::wrapper! {
    pub struct ProgressMessageObject(ObjectSubclass<imp::ProgressMessageObject>);
}

impl ProgressMessageObject {
    pub fn new(data: ProgressMessage) -> Self {
        glib::Object::builder()
            .property("index", data.index)
            .property("package-name", data.package_name)
            .property("message", data.message)
            .property("kind", data.kind as u8)
            .build()
    }
}

impl From<ProgressMessage> for ProgressMessageObject {
    fn from(data: ProgressMessage) -> Self {
        ProgressMessageObject::new(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_message_kind_from_u8() {
        init();

        assert_eq!(ProgressMessageKind::from(0), ProgressMessageKind::Starting);
        assert_eq!(ProgressMessageKind::from(1), ProgressMessageKind::Misc);
        assert_eq!(ProgressMessageKind::from(2), ProgressMessageKind::Error);
        assert_eq!(ProgressMessageKind::from(3), ProgressMessageKind::Warning);
        assert_eq!(ProgressMessageKind::from(4), ProgressMessageKind::Success);
        assert_eq!(ProgressMessageKind::from(5), ProgressMessageKind::Misc);
    }

    #[test]
    fn test_progress_message_kind_to_u8() {
        init();

        assert_eq!(ProgressMessageKind::Starting as u8, 0);
        assert_eq!(ProgressMessageKind::Misc as u8, 1);
        assert_eq!(ProgressMessageKind::Error as u8, 2);
        assert_eq!(ProgressMessageKind::Warning as u8, 3);
        assert_eq!(ProgressMessageKind::Success as u8, 4);
    }

    #[test]
    fn test_progress_message_object_new() {
        init();

        let message = ProgressMessage::new(
            1,
            "foo".to_string(),
            "bar".to_string(),
            ProgressMessageKind::Error,
        );

        let object = ProgressMessageObject::new(message);
        assert_eq!(object.index(), 1);
        assert_eq!(object.package_name(), "foo");
        assert_eq!(object.message(), "bar");
        assert_eq!(
            ProgressMessageKind::from(object.kind()),
            ProgressMessageKind::Error
        );
    }
}
