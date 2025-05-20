use gtk::{
    gio,
    glib::{self, Object},
    prelude::*,
};

use crate::app::application::QuellcodeApplication;

pub mod imp {
    use super::*;
    use glib::{subclass::InitializingObject, Properties};
    use gtk::{
        glib::subclass::prelude::*,
        subclass::{widget::{
            CompositeTemplateClass, CompositeTemplateInitializingExt, WidgetClassExt, WidgetImpl,
        }, window::WindowImpl},
        CompositeTemplate, Stack, TemplateChild,
    };

    #[derive(CompositeTemplate, Properties, Default)]
    #[template(resource = "/org/quellcode/quellcode/settings.ui")]
    #[properties(wrapper_type = super::SettingsWindow)]
    pub struct SettingsWindow {
        #[template_child]
        stack: TemplateChild<Stack>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SettingsWindow {
        const NAME: &'static str = "QuellcodeSettingsWindow";
        type Type = super::SettingsWindow;
        type ParentType = gtk::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for SettingsWindow {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for SettingsWindow {}
    impl WindowImpl for SettingsWindow {}
}

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
