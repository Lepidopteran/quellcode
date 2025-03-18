use std::cell::Ref;

use crate::app::window::Window;
use gio::ApplicationFlags;
use glib::clone;
use gtk::glib::Object;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib};
use log::{debug, error, info};
use syntect::html::css_for_theme_with_class_style;
use syntect::{
    highlighting::{Theme, ThemeSet},
    parsing::{SyntaxReference, SyntaxSet},
};

/// Generate gtk css theme for a [Theme]
///
/// > Does not generate scopes as [gtk::TextView] uses [gtk::TextTag] for syntax highlighting.
pub fn theme_to_gtk_css(theme: &Theme) -> String {
    let mut css = String::new();

    css.push_str("/*\n");
    let name = theme
        .name
        .clone()
        .unwrap_or_else(|| "unknown theme".to_string());

    css.push_str(&format!(" * theme \"{}\" generated by Quellcode\n", name));
    css.push_str(" */\n\n");
    css.push_str("codeview.code.view, codeview.code > text {\n");

    if let Some(foreground) = theme.settings.foreground {
        css.push_str(&format!(
            " color: rgb({} {} {});\n",
            foreground.r, foreground.g, foreground.b
        ));
    }

    if let Some(background) = theme.settings.background {
        css.push_str(&format!(
            " background: rgb({} {} {});\n",
            background.r, background.g, background.b
        ));
    }

    css.push_str("}\n\n");

    if let Some(selection) = theme.settings.selection {
        css.push_str("codeview.code text selection {\n");

        css.push_str(&format!(
            " background-color: rgb({} {} {} / 0.5);\n",
            selection.r, selection.g, selection.b
        ));

        css.push_str("}\n\n");
    }

    css
}

pub mod imp {
    use gdk::Display;
    use glib::{
        subclass::{InitializingObject, Signal},
        Properties,
    };

    use super::*;
    use std::cell::RefCell;

    #[derive(Debug, Properties)]
    #[properties(wrapper_type = super::QuellcodeApplication)]
    pub struct QuellcodeApplication {
        pub code_theme_provider: RefCell<gtk::CssProvider>,
        pub theme_set: RefCell<ThemeSet>,
        pub syntax_set: RefCell<SyntaxSet>,
        #[property(get, set)]
        pub code_theme: RefCell<String>,
        #[property(get, set)]
        pub code_syntax: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for QuellcodeApplication {
        const NAME: &'static str = "QuellcodeApplication";
        type Type = super::QuellcodeApplication;
        type ParentType = gtk::Application;

        fn new() -> Self {
            let provider = gtk::CssProvider::new();
            Self {
                code_theme_provider: RefCell::new(provider),
                theme_set: RefCell::new(ThemeSet::load_defaults()),
                syntax_set: RefCell::new(SyntaxSet::load_defaults_nonewlines()),
                code_theme: RefCell::new(String::new()),
                code_syntax: RefCell::new(String::new()),
            }
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for QuellcodeApplication {
        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "code-theme" => {
                    let theme_name = value.get::<String>().expect("Failed to get theme name");
                    let themes = &self.theme_set.borrow().themes;

                    if let Some(theme) = themes.get(&theme_name) {
                        self.code_theme_provider
                            .borrow()
                            .load_from_string(&theme_to_gtk_css(theme));

                        debug!("Loaded theme {}", theme_name);

                        self.obj().emit_by_name::<()>(
                            "theme-changed",
                            &[&self.code_theme.borrow().clone(), &theme_name],
                        );
                        self.code_theme.replace(theme_name);
                    } else {
                        self.obj().emit_by_name::<()>("theme-error", &[]);
                    }
                }
                "code-syntax" => {
                    let syntax_name = value.get::<String>().expect("Failed to get syntax name");
                    self.code_syntax.replace(syntax_name);
                }
                _ => unimplemented!(),
            }
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: once_cell::sync::Lazy<Vec<glib::subclass::Signal>> =
                once_cell::sync::Lazy::new(|| {
                    vec![
                        Signal::builder("syntax-changed").build(),
                        Signal::builder("theme-error").build(),
                        Signal::builder("theme-changed")
                            .param_types([glib::types::Type::STRING, glib::types::Type::STRING])
                            .build(),
                    ]
                });

            SIGNALS.as_ref()
        }
    }
    impl GtkApplicationImpl for QuellcodeApplication {}
    impl ApplicationImpl for QuellcodeApplication {
        fn activate(&self) {
            self.parent_activate();
            let provider: &gtk::CssProvider = &self.code_theme_provider.borrow();
            gtk::style_context_add_provider_for_display(
                &Display::default().expect("Failed to get display"),
                provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }

        fn startup(&self) {
            self.parent_startup();

            let themes = &self.theme_set.borrow().themes;
            let theme = themes.first_key_value().expect("Failed to get theme");
            self.obj().set_code_theme(theme.0.clone());
        }
    }
}

glib::wrapper! {
    pub struct QuellcodeApplication(ObjectSubclass<imp::QuellcodeApplication>)
        @extends gio::Application, gtk::Application,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl QuellcodeApplication {
    pub fn new(application_id: &str) -> Self {
        let app: Self = Object::builder().build();
        app.set_application_id(Some(application_id));

        app
    }

    pub fn theme_set(&self) -> Ref<ThemeSet> {
        self.imp().theme_set.borrow()
    }

    pub fn syntax_set(&self) -> Ref<SyntaxSet> {
        self.imp().syntax_set.borrow()
    }
}
