use std::cell::{Ref, RefCell};
use std::path::PathBuf;

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

use super::{
    code_theme_files,
    config::{load_config, save_config, write_default_config_file, Config, Core},
    dir, ThemeFormat, Window,
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
    css.push_str("codeview {\n");

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
    use std::{cell::Cell, io, rc::Rc, sync::Arc};

    use gdk::Display;
    use glib::{
        closure_local,
        property::PropertySet,
        subclass::{InitializingObject, Signal},
        GString, Properties,
    };
    use log::warn;
    use serde::Serialize;

    use crate::app::generator::{svg::SvgGenerator, Generator, RenderOutput};

    use super::*;

    #[derive(Properties)]
    #[properties(wrapper_type = super::QuellcodeApplication)]
    pub struct QuellcodeApplication {
        pub main_window: RefCell<Option<Window>>,
        pub generator: RefCell<Option<Arc<dyn Generator>>>,
        pub code_theme_provider: RefCell<gtk::CssProvider>,
        pub theme_set: RefCell<ThemeSet>,
        pub syntax_set: RefCell<SyntaxSet>,
        pub config: Rc<RefCell<Config>>,
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
                generator: RefCell::new(Some(Arc::new(SvgGenerator::default()))),
                config: Rc::new(RefCell::new(Config::new())),
                main_window: RefCell::new(None),
            }
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for QuellcodeApplication {
        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "code-theme" => {
                    let theme_name = value.get::<String>().expect("Failed to get theme name");
                    let themes_set = &self.theme_set.borrow();

                    if let Some(theme) = themes_set.themes.get(&theme_name) {
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

            let style = gtk::CssProvider::new();
            style.load_from_string(include_str!("../../assets/style.css"));
            gtk::style_context_add_provider_for_display(
                &Display::default().expect("Failed to get display"),
                &style,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );

            let app_theme = gtk::CssProvider::new();
            app_theme.load_from_string(include_str!("../../assets/theme.css"));
            gtk::style_context_add_provider_for_display(
                &Display::default().expect("Failed to get display"),
                &app_theme,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );

            // TODO: Organize the following code.
            let window = Window::new(&self.obj());

            let theme_set = self.theme_set.borrow();
            let themes = create_theme_model(&theme_set);
            let theme_dropdown = window.theme_dropdown();

            let syntax_set = self.syntax_set.borrow();
            let syntaxes = create_syntax_model(&syntax_set);
            let syntax_dropdown = window.syntax_dropdown();

            theme_dropdown.set_model(Some(&themes));

            if let Some(position) = theme_set
                .themes
                .iter()
                .position(|t| *t.0 == *self.code_theme.borrow())
            {
                theme_dropdown.set_selected(position as u32);
            } else {
                warn!("Could not find theme {}", self.code_theme.borrow());
                theme_dropdown.set_selected(0);
            }

            syntax_dropdown.set_model(Some(&syntaxes));

            if let Some(position) = syntax_set
                .syntaxes()
                .iter()
                .position(|s| s.name == *self.code_syntax.borrow())
            {
                syntax_dropdown.set_selected(position as u32);
            } else {
                warn!("Could not find syntax {}", self.code_syntax.borrow());
                syntax_dropdown.set_selected(0);
            }

            let config = self.config.clone();
            let self_ = self.obj().clone();
            theme_dropdown.connect_selected_notify(move |dropdown| {
                let theme_name = themes.string(dropdown.selected()).unwrap();
                self_.set_property("code-theme", &theme_name);
                config.borrow_mut().core.theme = theme_name.to_string();
            });

            let self_ = self.obj().clone();
            let config = self.config.clone();
            syntax_dropdown.connect_selected_notify(move |dropdown| {
                let syntax_name = syntaxes.string(dropdown.selected()).unwrap();
                self_.set_property("code-syntax", &syntax_name);
                config.borrow_mut().core.syntax = syntax_name.to_string();
            });

            let viewer = window.viewer().clone();
            let editor = window.editor().clone();

            editor.set_theme(
                self.theme_set
                    .borrow()
                    .themes
                    .get(&self.code_theme.borrow().to_string())
                    .cloned(),
            );

            editor.set_syntax(
                self.syntax_set
                    .borrow()
                    .find_syntax_by_name(&self.code_syntax.borrow().to_string())
                    .cloned(),
            );

            viewer.set_theme(
                self.theme_set
                    .borrow()
                    .themes
                    .get(&self.code_theme.borrow().to_string())
                    .cloned(),
            );

            viewer.set_syntax(Some(
                viewer
                    .syntax_set()
                    .find_syntax_by_name("XML")
                    .unwrap()
                    .clone(),
            ));

            let self_ = self.obj().clone();
            self_.connect_closure(
                "theme-changed",
                false,
                closure_local!(move |app: &super::QuellcodeApplication,
                                     old_theme: &str,
                                     new_theme: &str| {
                    if old_theme == new_theme {
                        return;
                    }

                    let theme_set = app.theme_set();
                    if let Some(theme) = theme_set.themes.get(new_theme) {
                        editor.set_theme(Some(theme.clone()));
                        viewer.set_theme(Some(theme.clone()));
                    }
                }),
            );

            let editor = window.editor().clone();
            let editor_buffer = editor.buffer().clone();

            self_.connect_code_syntax_notify(move |app| {
                if let Some(syntax) = app.syntax_set().find_syntax_by_name(&app.code_syntax()) {
                    editor.set_syntax(Some(syntax.clone()));
                }
            });

            let editor = window.editor().clone();
            let (generator_sender, generator_receiver) = async_channel::bounded(1);
            let generator = self.generator.borrow().as_ref().unwrap().clone();
            editor_buffer.connect_changed(move |buffer| {
                let editor_syntax = editor.syntax().clone();
                let editor_syntax_set = editor.syntax_set().clone();
                let editor_theme = editor.theme().clone();

                let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), true);
                let generator_sender = generator_sender.clone();
                let generator = generator.clone();
                gio::spawn_blocking(move || {
                    let generated_svg = generator.generate(
                        &text,
                        &editor_theme.unwrap(),
                        &editor_syntax.unwrap(),
                        &editor_syntax_set,
                    );

                    generator_sender
                        .send_blocking(generated_svg)
                        .expect("Failed to send svg");
                });
            });
            let viewer = window.imp().viewer.clone();
            glib::spawn_future_local(async move {
                while let Ok(svg) = generator_receiver.recv().await {
                    if let Ok(RenderOutput::Text(svg)) = svg {
                        viewer.buffer().set_text(&svg);
                    }
                }
            });

            let config = self.config.clone();
            window.connect_close_request(move |_| {
                debug!("Saving config before closing:\n{:#?}", config.borrow());
                if let Err(err) = save_config(&config.borrow()) {
                    error!("Failed to save config, Error:\n{}", err);
                }

                debug!("Closing window");
                glib::Propagation::Proceed
            });

            window.present();
            self.main_window.replace(Some(window.clone()));
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
                        core: Core {
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

            let theme: (&String, &Theme) = theme_set
                .themes
                .get_key_value(&config.core.theme)
                .unwrap_or_else(|| {
                    log::warn!(
                        "Theme \"{}\" not found, using default theme",
                        config.core.theme
                    );
                    theme_set
                        .themes
                        .first_key_value()
                        .expect("Failed to get theme")
                });

            let syntax_sets = &mut self.syntax_set.borrow_mut();

            let syntax = syntax_sets
                .find_syntax_by_name(&config.core.syntax)
                .unwrap_or_else(|| {
                    log::warn!(
                        "Syntax \"{}\" not found, using default syntax",
                        config.core.syntax
                    );
                    syntax_sets
                        .syntaxes()
                        .first()
                        .expect("Failed to get syntax")
                });

            debug!("Loaded config:\n{:#?}", config);
            self.config.replace(config);

            self.code_theme.set(theme.0.clone());
            self.code_syntax.set(syntax.name.clone());
            self.code_theme_provider
                .borrow()
                .load_from_string(&theme_to_gtk_css(theme.1));
        }
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

fn create_theme_model(themes_set: &ThemeSet) -> gtk::StringList {
    gtk::StringList::new(
        &themes_set
            .themes
            .iter()
            .map(|t| t.0.as_str())
            .collect::<Vec<_>>(),
    )
}

fn create_syntax_model(syntax_set: &SyntaxSet) -> gtk::StringList {
    gtk::StringList::new(
        &syntax_set
            .syntaxes()
            .iter()
            .map(|t| t.name.as_str())
            .collect::<Vec<_>>(),
    )
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
