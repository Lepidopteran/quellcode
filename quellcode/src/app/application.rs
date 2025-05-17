use std::cell::{Ref, RefCell};

use gtk::glib::Object;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib};
use log::{debug, error};
use syntect::{
    highlighting::{Theme, ThemeSet},
    parsing::SyntaxSet,
};

pub const FALLBACK_FONT_FAMILY: &str = "Monospace";

use super::{
    code_theme_files,
    config::{load_config, write_default_config_file, CodeSettings, Config},
    dir, ThemeFormat, Window,
};

pub mod imp {
    use std::{cell::RefMut, io, rc::Rc};

    use super::*;
    use gdk::Display;

    #[derive(Debug)]
    pub struct QuellcodeApplication {
        pub code_theme_provider: RefCell<gtk::CssProvider>,
        pub theme_set: RefCell<ThemeSet>,
        pub syntax_set: RefCell<SyntaxSet>,
        pub config: Rc<RefCell<Config>>,
    }

    impl QuellcodeApplication {
        pub fn set_config(&self, config: Config) {
            *self.config.borrow_mut() = config;
        }

        pub fn config(&self) -> Ref<Config> {
            self.config.borrow()
        }

        pub fn config_mut(&self) -> RefMut<Config> {
            self.config.borrow_mut()
        }
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
                config: Rc::new(RefCell::new(Config::new())),
            }
        }
    }

    impl ObjectImpl for QuellcodeApplication {}
    impl GtkApplicationImpl for QuellcodeApplication {}
    impl ApplicationImpl for QuellcodeApplication {
        fn activate(&self) {
            self.parent_activate();

            let base = gtk::CssProvider::new();
            base.load_from_string(include_str!("../../data/css/base.css"));
            gtk::style_context_add_provider_for_display(
                &Display::default().expect("Failed to get display"),
                &base,
                gtk::STYLE_PROVIDER_PRIORITY_THEME,
            );

            let style = gtk::CssProvider::new();
            style.load_from_string(include_str!("../../data/css/style.css"));
            gtk::style_context_add_provider_for_display(
                &Display::default().expect("Failed to get display"),
                &style,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );

            let window = Window::new(&self.obj());
            window.present();
        }

        fn startup(&self) {
            self.parent_startup();
            ensure_app_directories_exist();

            let theme_set = &mut self.theme_set.borrow_mut();

            load_custom_themes(theme_set);

            let config = match load_config() {
                Ok(config) => config,
                Err(err) => {
                    let theme = theme_set
                        .themes
                        .first_key_value()
                        .expect("Failed to get theme");
                    let config = Config {
                        code: CodeSettings {
                            theme: theme.0.clone(),
                            syntax: self.syntax_set.borrow().syntaxes()[0].name.clone(),
                            ..Default::default()
                        },
                    };
                    if let Some(io_err) = err.downcast_ref::<io::Error>() {
                        if io_err.kind() == io::ErrorKind::NotFound {
                            let result = write_default_config_file(&config);
                            if let Err(err) = result {
                                error!("Failed to write default config file, Error:\n{}", err);
                            }
                            return;
                        }

                        error!(
                            "Failed to read config, using default config instead, Error:\n{}",
                            io_err
                        );
                    } else {
                        error!(
                            "Failed to load config, using default config instead, Error:\n{}",
                            err
                        );
                    }

                    config
                }
            };

            debug!("Loaded config");
            self.config.replace(config);
        }
    }

    fn ensure_app_directories_exist() {
        for dir in [
            dir::data_dir(),
            dir::config_dir(),
            dir::cache_dir(),
            dir::code_theme_dir(),
            dir::code_syntax_dir(),
        ] {
            if dir.exists() {
                continue;
            }

            std::fs::create_dir_all(dir).unwrap();
        }
    }

    fn load_custom_themes(theme_set: &mut ThemeSet) {
        for (format, path) in code_theme_files() {
            match format {
                ThemeFormat::VsCode => {
                    let vscode_theme = syntect_vscode::parse_vscode_theme_file(&path);

                    if let Ok(vscode_theme) = vscode_theme {
                        let theme_name = vscode_theme
                            .name
                            .clone()
                            .unwrap_or(path.file_stem().unwrap().to_string_lossy().to_string());

                        let theme = Theme::try_from(vscode_theme).expect("Failed to parse theme");

                        theme_set.themes.insert(theme_name, theme);
                    }
                }
                ThemeFormat::Sublime => {
                    let color_scheme = sublime_color_scheme::parse_color_scheme_file(&path);

                    if let Ok(color_scheme) = color_scheme {
                        let theme_name = color_scheme
                            .name
                            .clone()
                            .unwrap_or(path.file_stem().unwrap().to_string_lossy().to_string());

                        let theme = Theme::try_from(color_scheme).expect("Failed to parse theme");

                        theme_set.themes.insert(theme_name, theme);
                    }
                }
                ThemeFormat::TmTheme => {
                    let theme = ThemeSet::get_theme(&path);
                    if let Ok(theme) = theme {
                        let theme_name = theme
                            .clone()
                            .name
                            .unwrap_or(path.file_stem().unwrap().to_string_lossy().to_string());

                        theme_set.themes.insert(theme_name, theme);
                    }
                }
            }
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

    pub fn config(&self) -> Ref<Config> {
        self.imp().config.borrow()
    }
}
