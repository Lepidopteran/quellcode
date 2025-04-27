use evalexpr::*;
use glib::subclass::InitializingObject;
use glib::Object;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, CompositeTemplate, TemplateChild};

use super::application::QuellcodeApplication;
use super::ui::{code_view::CodeView, FontFamilyChooser};

const UNITS: &[&str] = &["px", "pt", "pc", "in", "cm", "mm"];
const ROUND_DIGITS: i32 = 4;

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
        pub viewer_loading_spinner: TemplateChild<gtk::Spinner>,

        #[template_child]
        pub viewer_loading_box: TemplateChild<gtk::Box>,

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

        #[template_child]
        pub font_size_entry: TemplateChild<gtk::Entry>,

        #[template_child]
        pub font_size_scale: TemplateChild<gtk::Scale>,

        #[template_child]
        pub generator_box: TemplateChild<gtk::Box>,

        #[template_child]
        pub action_button: TemplateChild<gtk::Button>,
    }

    #[gtk::template_callbacks]
    impl Window {
        #[template_callback]
        fn font_size_scale_changed(&self) {
            self.font_size_entry
                .set_text(&self.font_size_scale.value().to_string());
        }

        #[template_callback]
        fn font_size_entry_activate(&self, entry: &gtk::Entry) {
            let mut text = entry.text().to_lowercase();
            let context: HashMapContext = context_map! {
                "px" => float 1.0,
                "in" => float 96.0,
                "pt" => float 1.3333333333,
                "pc" => float 16.0,
                "cm" => float 37.7952755906,
                "mm" => float 3.7795275591
            }
            .expect("Failed to create context map");

            for (long, short) in [
                ("inch", "in"),
                ("inches", "in"),
                ("millimeter", "mm"),
                ("millimeters", "mm"),
                ("centimeter", "cm"),
                ("centimeters", "cm"),
                ("pica", "pc"),
                ("picas", "pc"),
                ("point", "pt"),
                ("points", "pt"),
                ("pixel", "px"),
                ("pixels", "px"),
            ] {
                text = text.replace(long, short);
            }

            if let Ok(value) = eval_with_context(&preprocess_units(&text), &context) {
                let factor = 10f64.powi(ROUND_DIGITS);
                let value = value.as_number().unwrap();
                self.font_size_scale
                    .set_value((value * factor).round() / factor);
            } else {
                entry.set_text(&self.font_size_scale.value().to_string());
            }
        }
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
            klass.bind_template_callbacks();
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

            self.font_size_entry
                .set_text(&self.font_size_scale.value().to_string());

            let loading_spinner = self.viewer_loading_spinner.clone();

            self.viewer_loading_box
                .connect_visible_notify(move |container| {
                    loading_spinner.set_spinning(container.is_visible());
                });

            for snap_scale in (8..96).step_by(8) {
                self.font_size_scale
                    .add_mark(snap_scale as f64, gtk::PositionType::Top, None);
            }

            self.inspector.set_size_request(300, -1);
        }
    }
    impl WidgetImpl for Window {}
    impl WindowImpl for Window {}
    impl ApplicationWindowImpl for Window {}

    fn preprocess_units(input: &str) -> String {
        let mut output = String::new();
        let mut chars = input.chars().peekable();

        while let Some(c) = chars.next() {
            output.push(c);

            if c.is_ascii_digit() || c == '.' {
                let mut lookahead = String::new();

                while let Some(&next_c) = chars.peek() {
                    if next_c.is_ascii_alphabetic() {
                        lookahead.push(next_c);
                        chars.next();
                    } else {
                        break;
                    }
                }

                if !lookahead.is_empty() && UNITS.contains(&lookahead.as_str()) {
                    output.push_str(" * ");
                    output.push_str(&lookahead);
                } else {
                    output.push_str(&lookahead);
                }
            }
        }

        output
    }
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

    pub fn generator_box(&self) -> &gtk::Box {
        &self.imp().generator_box
    }

    pub fn action_button(&self) -> &gtk::Button {
        &self.imp().action_button
    }

    pub fn editor(&self) -> &CodeView {
        &self.imp().editor
    }

    pub fn viewer_loading_box(&self) -> &gtk::Box {
        &self.imp().viewer_loading_box
    }

    pub fn viewer_loading_spinner(&self) -> &gtk::Spinner {
        &self.imp().viewer_loading_spinner
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

    pub fn font_size_scale(&self) -> &gtk::Scale {
        &self.imp().font_size_scale
    }
}
