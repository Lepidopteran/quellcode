use glib::subclass::InitializingObject;
use glib::Object;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, Application, CompositeTemplate, TemplateChild};

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
        pub layout: TemplateChild<gtk::Paned>,

        #[template_child]
        pub editor: TemplateChild<CodeView>,
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

    impl ObjectImpl for Window {}
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
    pub fn new(app: &Application) -> Self {
        Object::builder().property("application", app).build()
    }
}
