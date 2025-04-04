use glib::subclass::InitializingObject;
use glib::Object;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, CompositeTemplate, TemplateChild};

use super::application::QuellcodeApplication;
use super::ui::code_view::CodeView;
use super::ui::FontFamilyChooser;

pub mod imp {
    use crate::app::ui::code_view::CodeView;

    use super::*;
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/org/quellcode/quellcode/window.ui")]
    pub struct Window {
        #[template_child]
        pub content: TemplateChild<gtk::Box>,

        #[template_child]
        pub inspector: TemplateChild<gtk::Box>,

        #[template_child]
        pub layout: TemplateChild<gtk::Box>,

        #[template_child]
        pub editor: TemplateChild<CodeView>,

        #[template_child]
        pub viewer: TemplateChild<CodeView>,

        #[template_child]
        theme_label: TemplateChild<gtk::Label>,

        #[template_child]
        pub theme_dropdown: TemplateChild<gtk::DropDown>,

        #[template_child]
        syntax_label: TemplateChild<gtk::Label>,

        #[template_child]
        pub syntax_dropdown: TemplateChild<gtk::DropDown>,

        #[template_child]
        pub font_family_chooser: TemplateChild<FontFamilyChooser>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        // `NAME` needs to match `class` attribute of template
        const NAME: &'static str = "QuellCodeWindow";
        type Type = super::Window;
        type ParentType = gtk::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Window {
        fn constructed(&self) {
            self.parent_constructed();
            self.theme_label
                .set_mnemonic_widget(Some(&self.theme_dropdown.clone()));

            self.syntax_label
                .set_mnemonic_widget(Some(&self.syntax_dropdown.clone()));

            self.inspector.set_size_request(300, -1);
        }
    }
    impl WidgetImpl for Window {}
    impl WindowImpl for Window {}
    impl ApplicationWindowImpl for Window {}
}

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
    pub fn new(app: &QuellcodeApplication) -> Self {
        let window: Self = Object::builder().build();
        window.set_application(Some(app));

        window
    }

    pub fn inspector(&self) -> &gtk::Box {
        &self.imp().inspector
    }

    pub fn editor(&self) -> &CodeView {
        &self.imp().editor
    }

    pub fn viewer(&self) -> &CodeView {
        &self.imp().viewer
    }

    pub fn theme_dropdown(&self) -> &gtk::DropDown {
        &self.imp().theme_dropdown
    }

    pub fn syntax_dropdown(&self) -> &gtk::DropDown {
        &self.imp().syntax_dropdown
    }

    pub fn font_family_chooser(&self) -> &FontFamilyChooser {
        &self.imp().font_family_chooser
    }
}
