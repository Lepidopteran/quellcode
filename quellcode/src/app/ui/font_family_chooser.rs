use gtk::{
    glib::{self, Object},
    prelude::*,
    subclass::prelude::ObjectSubclassIsExt,
};

pub mod imp {
    use std::{cell::RefCell, rc::Rc};

    use super::*;
    use glib::{
        property::PropertySet,
        subclass::{InitializingObject, Signal},
        GString, Properties,
    };
    use gtk::{
        gio::ListStore,
        glib::subclass::prelude::*,
        pango::{AttrFontDesc, AttrList, FontFamily},
        subclass::widget::{
            CompositeTemplateCallbacksClass, CompositeTemplateClass,
            CompositeTemplateInitializingExt, WidgetClassExt, WidgetImpl, WidgetImplExt,
        },
        Allocation, CompositeTemplate, Label, ListItem, SignalListItemFactory, TemplateChild,
        Widget,
    };

    #[derive(CompositeTemplate, Properties, Default)]
    #[template(resource = "/org/quellcode/quellcode/font_family_chooser.ui")]
    #[properties(wrapper_type = super::FontFamilyChooser)]
    pub struct FontFamilyChooser {
        #[template_child]
        pub button: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub label: TemplateChild<gtk::Label>,
        #[template_child]
        pub popover: TemplateChild<gtk::Popover>,
        #[template_child]
        monospace_toggle: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub list: TemplateChild<gtk::ListView>,
        #[template_child]
        pub selection: TemplateChild<gtk::SingleSelection>,
        #[template_child]
        search: TemplateChild<gtk::SearchEntry>,
        #[template_child]
        name_filter: TemplateChild<gtk::StringFilter>,
        #[template_child]
        filter_model: TemplateChild<gtk::FilterListModel>,
        #[template_child]
        monospace_filter: TemplateChild<gtk::CustomFilter>,
        #[property(get, set)]
        pub selected_family: RefCell<Option<FontFamily>>,
        #[property(get, name = "monospace-filter")]
        pub monospace: Rc<RefCell<bool>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FontFamilyChooser {
        const NAME: &'static str = "QuellcodeFontFamilyChooser";
        type Type = super::FontFamilyChooser;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("font-family-chooser");
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[gtk::template_callbacks]
    impl FontFamilyChooser {
        #[template_callback]
        fn button_toggled(&self, button: &gtk::ToggleButton) {
            if button.is_active() {
                self.popover.popup();
            } else {
                self.popover.popdown();
            }
        }

        #[template_callback]
        fn popover_closed(&self) {
            self.button.set_active(false);
        }

        #[template_callback]
        fn row_activated(&self) {
            let selected_item = self.selection.selected_item();
            let family = selected_item
                .and_downcast_ref::<FontFamily>()
                .expect("Expected FontFamily");

            self.label.set_text(&family.name());
            self.obj().set_selected_family(family.clone());
            self.popover.popdown();
            self.name_filter.set_search(None);
            self.obj().emit_by_name::<()>("font-activated", &[family]);
        }

        #[template_callback]
        fn selection_changed(&self) {
            if let Some(selected_item) = self.selection.selected_item() {
                let family = selected_item
                    .downcast_ref::<FontFamily>()
                    .expect("Expected FontFamily");

                self.obj().emit_by_name::<()>("font-selected", &[family]);
            };
        }

        #[template_callback]
        fn get_font_name(family: FontFamily) -> GString {
            family.name()
        }

        #[template_callback]
        fn monospace_toggled(&self, button: &gtk::CheckButton) {
            self.monospace.set(button.is_active());
            self.monospace_filter.changed(gtk::FilterChange::Different);
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for FontFamilyChooser {
        fn constructed(&self) {
            self.parent_constructed();
            self.monospace.set(true);

            let self_obj = self.obj();
            self_obj.set_accessible_role(gtk::AccessibleRole::ComboBox);

            let label = self.label.clone();
            self_obj.connect_selected_family_notify(move |obj| {
                if let Some(family) = obj.selected_family() {
                    label.set_text(&family.name());
                } else {
                    label.set_text("");
                }
            });

            let pango_context = self.obj().create_pango_context();
            let factory = SignalListItemFactory::default();
            let model = ListStore::new::<FontFamily>();
            model.extend_from_slice(&pango_context.list_families());

            self.filter_model.set_model(Some(&model));

            let only_monospace = self.monospace.clone();

            self.monospace_filter.set_filter_func(move |obj| {
                let family = obj
                    .downcast_ref::<FontFamily>()
                    .expect("Expected FontFamily");
                if *only_monospace.borrow() {
                    family.is_monospace()
                } else {
                    true
                }
            });

            factory.connect_setup(setup_item);
            factory.connect_bind(bind_item);

            self.list.set_factory(Some(&factory));
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: once_cell::sync::Lazy<Vec<glib::subclass::Signal>> =
                once_cell::sync::Lazy::new(|| {
                    vec![
                        Signal::builder("font-activated")
                            .param_types([FontFamily::static_type()])
                            .build(),
                        Signal::builder("font-selected")
                            .param_types([FontFamily::static_type()])
                            .build(),
                    ]
                });

            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for FontFamilyChooser {
        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            self.button.measure(orientation, for_size)
        }

        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            self.parent_size_allocate(width, height, baseline);
            self.button
                .size_allocate(&Allocation::new(0, 0, width, height), baseline);
            self.popover.set_size_request(width, -1);
            self.popover.queue_resize();
        }
    }

    fn setup_item(_: &SignalListItemFactory, obj: &Object) {
        let list_item = obj.downcast_ref::<ListItem>().expect("Expected ListItem");

        let label = Label::builder()
            .xalign(0.0)
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .build();

        list_item.set_child(Some(&label));
        list_item
            .property_expression("item")
            .chain_property::<FontFamily>("name")
            .bind(&label, "label", Widget::NONE);
    }

    fn bind_item(_: &SignalListItemFactory, obj: &Object) {
        let list_item = obj.downcast_ref::<ListItem>().expect("Expected ListItem");

        let item = list_item
            .item()
            .and_downcast::<FontFamily>()
            .expect("Expected FontFamily");

        let label = list_item
            .child()
            .expect("Expected Child")
            .downcast::<Label>()
            .expect("Expected Label");

        let attr_list = AttrList::new();
        if let Some(font_face) = item.face(None) {
            attr_list.insert(AttrFontDesc::new(&font_face.describe()));
        }

        label.set_attributes(Some(&attr_list));
    }
}

glib::wrapper! {
    pub struct FontFamilyChooser(ObjectSubclass<imp::FontFamilyChooser>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl FontFamilyChooser {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn popover(&self) -> &gtk::Popover {
        &self.imp().popover
    }

    pub fn list(&self) -> &gtk::ListView {
        &self.imp().list
    }

    pub fn selection(&self) -> &gtk::SingleSelection {
        &self.imp().selection
    }
}

impl Default for FontFamilyChooser {
    fn default() -> Self {
        Self::new()
    }
}
