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

use super::ui::SettingsWindow;
use super::{
    code_theme_files, dir,
    generator::{
        svg::SvgGenerator, Generator as GeneratorTrait, GeneratorInfo, Info as GeneratorDetails,
    },
    state::{load_state, CodeState, State},
    ThemeFormat, Window,
};

type GeneratorFactory = Box<dyn Fn() -> Box<dyn GeneratorTrait>>;
type Generator = (GeneratorDetails, GeneratorFactory);

pub mod imp {

    use super::*;
    use gdk::Display;
    use std::{cell::RefMut, io, rc::Rc};

    pub struct QuellcodeApplication {
        pub code_theme_provider: RefCell<gtk::CssProvider>,
        pub theme_set: RefCell<ThemeSet>,
        pub syntax_set: RefCell<SyntaxSet>,
        pub state: Rc<RefCell<State>>,
        pub generator_registry: RefCell<Vec<(GeneratorDetails, GeneratorFactory)>>,
    }

    impl QuellcodeApplication {
        pub fn set_config(&self, config: State) {
            *self.state.borrow_mut() = config;
        }

        pub fn config(&self) -> Ref<State> {
            self.state.borrow()
        }

        pub fn config_mut(&self) -> RefMut<State> {
            self.state.borrow_mut()
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
                state: Rc::new(RefCell::new(State::new())),
                generator_registry: RefCell::new(Vec::new()),
            }
        }
    }

    impl ObjectImpl for QuellcodeApplication {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_actions();
        }
    }
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

            let config = match load_state() {
                Ok(config) => config,
                Err(err) => {
                    let theme = theme_set
                        .themes
                        .first_key_value()
                        .expect("Failed to get theme");
                    let config = State {
                        code: CodeState {
                            theme: Some(theme.0.clone()),
                            syntax: Some(self.syntax_set.borrow().syntaxes()[0].name.clone()),
                            ..Default::default()
                        },
                        ..Default::default()
                    };
                    if let Some(io_err) = err.downcast_ref::<io::Error>() {
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

            let generators = &mut self.generator_registry.borrow_mut();

            generators.push((
                SvgGenerator::information(),
                Box::new(|| Box::new(SvgGenerator::new())),
            ));

            self.state.replace(config);
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

    fn setup_actions(&self) {
        let open_preferences = gio::ActionEntry::builder("open-preferences")
            .activate(move |app: &QuellcodeApplication, _, _| {
                for window in app.windows() {
                    if let Ok(settings_window) = window.downcast::<SettingsWindow>() {
                        settings_window.present();

                        return;
                    }
                }

                let settings_window = SettingsWindow::new(app);
                settings_window.present();
            })
            .build();
        self.add_action_entries([open_preferences]);
    }

    pub fn generator_registry(&self) -> Ref<Vec<Generator>> {
        self.imp().generator_registry.borrow()
    }

    pub fn theme_set(&self) -> Ref<ThemeSet> {
        self.imp().theme_set.borrow()
    }

    pub fn syntax_set(&self) -> Ref<SyntaxSet> {
        self.imp().syntax_set.borrow()
    }

    pub fn state(&self) -> Ref<State> {
        self.imp().state.borrow()
    }
}
