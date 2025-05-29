use gtk::{
    gio,
    glib::{self, Object},
    prelude::*,
};

use crate::app::application::QuellcodeApplication;
mod imp;

glib::wrapper! {
    pub struct SettingsWindow(ObjectSubclass<imp::SettingsWindow>)
        @extends gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl SettingsWindow {
    pub fn new(app: &QuellcodeApplication) -> Self {
        let window: Self = Object::builder().build();
        window.set_application(Some(app));

        window
    }
}
