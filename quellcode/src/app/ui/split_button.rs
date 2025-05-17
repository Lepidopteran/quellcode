use gtk::{
    glib::{self, Object},
    prelude::*,
};

pub mod imp {
    use std::cell::{Cell, RefCell};

    use super::*;
    use glib::{
        property::PropertySet,
        subclass::{InitializingObject, Signal},
        Properties,
    };
    use gtk::{
        gio::MenuModel,
        glib::subclass::prelude::*,
        subclass::widget::{
            CompositeTemplateCallbacksClass, CompositeTemplateClass,
            CompositeTemplateInitializingExt, WidgetClassExt, WidgetImpl, WidgetImplExt,
        },
        template_callbacks, Allocation, CompositeTemplate, Popover, TemplateChild, Widget,
    };

    #[derive(CompositeTemplate, Properties, Default)]
    #[template(resource = "/org/quellcode/quellcode/split_button.ui")]
    #[properties(wrapper_type = super::SplitButton)]
    pub struct SplitButton {
        #[template_child]
        pub button: TemplateChild<gtk::Button>,
        #[template_child]
        pub menu_button: TemplateChild<gtk::MenuButton>,
        #[template_child]
        container: TemplateChild<gtk::Box>,
        #[property(get, set)]
        pub shrink_button: Cell<bool>,
        #[property(get, set)]
        pub menu_model: RefCell<Option<MenuModel>>,
        #[property(get, set)]
        pub popover: RefCell<Option<Popover>>,
        #[property(get, set)]
        pub label: RefCell<Option<String>>,
        #[property(get, set)]
        pub dropdown_tooltip: RefCell<Option<String>>,
        #[property(get, set)]
        pub child: RefCell<Option<Widget>>,
    }

    #[template_callbacks]
    impl SplitButton {
        #[template_callback]
        fn button_clicked(&self, _button: &gtk::Button) {
            self.obj().emit_by_name::<()>("clicked", &[]);
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SplitButton {
        const NAME: &'static str = "QuellcodeSplitButton";
        type Type = super::SplitButton;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("split-button");
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for SplitButton {
        fn constructed(&self) {
            self.parent_constructed();
            let self_obj = self.obj();
            let button = self.button.clone();
            let menu_button = self.menu_button.clone();

            self_obj
                .bind_property("shrink-button", &button, "hexpand")
                .transform_to(|_, boolean: bool| Some(!boolean))
                .sync_create()
                .build();

            self_obj
                .bind_property("menu-model", &menu_button, "menu-model")
                .sync_create()
                .build();

            self_obj
                .bind_property("popover", &menu_button, "popover")
                .sync_create()
                .build();

            self_obj
                .bind_property("label", &button, "label")
                .sync_create()
                .build();

            self_obj
                .bind_property("dropdown-tooltip", &menu_button, "tooltip-text")
                .sync_create()
                .build();

            self_obj
                .bind_property("child", &button, "child")
                .sync_create()
                .build();

            self_obj.set_accessible_role(gtk::AccessibleRole::Group);
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: once_cell::sync::Lazy<Vec<glib::subclass::Signal>> =
                once_cell::sync::Lazy::new(|| vec![Signal::builder("clicked").build()]);

            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for SplitButton {
        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            self.container.measure(orientation, for_size)
        }
        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            self.parent_size_allocate(width, height, baseline);
            self.container
                .size_allocate(&Allocation::new(0, 0, width, height), baseline);
        }
    }
}

glib::wrapper! {
    pub struct SplitButton(ObjectSubclass<imp::SplitButton>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl SplitButton {
    pub fn new() -> Self {
        Object::builder().build()
    }
}

impl Default for SplitButton {
    fn default() -> Self {
        Self::new()
    }
}
