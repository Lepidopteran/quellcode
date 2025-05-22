use glib::Object;
use gtk::gio::ActionEntry;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use log::{debug, warn};
use std::sync::{Arc, Mutex};
use syntect::{highlighting::Theme, parsing::SyntaxSet};

use super::state::WindowState;
use super::{
    application::{QuellcodeApplication, FALLBACK_FONT_FAMILY},
    state::CodeState,
    generator::{svg::SvgGenerator, Generator as GeneratorTrait},
    ui::FontFamilyChooser,
};

mod imp;

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
        window.load_state(app);

        let inner = window.imp();
        inner.set_generator(Some(Arc::new(Mutex::new(SvgGenerator::new()))));

        window
    }

    fn load_state(&self, app: &QuellcodeApplication) {
        self.load_syntaxes(app);
        self.load_themes(app);

        let state = app.state();
        let inner = self.imp();

        debug!("Loading config...");

        let CodeState {
            theme,
            syntax,
            font_family,
            font_size,
        } = &state.code;

        if let Some(theme) = theme {
            let themes = app.theme_set().themes.clone();
            let theme: (&String, &Theme) = themes.get_key_value(theme).unwrap_or_else(|| {
                log::warn!("Theme \"{}\" not found, using default theme", theme);
                themes.first_key_value().expect("Failed to get theme")
            });

            debug!(
                "Selected theme: {}, position {}",
                theme.0,
                themes.iter().position(|t| t.0 == theme.0).unwrap() as u32
            );

            inner
                .theme_dropdown
                .set_selected(themes.iter().position(|t| t.0 == theme.0).unwrap() as u32);
        }

        if let Some(syntax) = syntax {
            let syntax_sets = app.syntax_set();
            let syntax = syntax_sets.find_syntax_by_name(syntax).unwrap_or_else(|| {
                log::warn!("Syntax \"{}\" not found, using default syntax", syntax);
                syntax_sets
                    .syntaxes()
                    .first()
                    .expect("Failed to get syntax")
            });

            debug!(
                "Selected syntax: {}, position {}",
                syntax.name,
                syntax_sets
                    .syntaxes()
                    .iter()
                    .position(|s| s.name == syntax.name)
                    .unwrap() as u32
            );

            inner.syntax_dropdown.set_selected(
                syntax_sets
                    .syntaxes()
                    .iter()
                    .position(|s| s.name == syntax.name)
                    .unwrap() as u32,
            );
        }

        if let Some(font_family) = font_family {
            let pango_context = self.pango_context();
            let font_families = pango_context.list_families();
            if let Some(font) = font_families.iter().find(|f| f.name() == *font_family) {
                debug!("Found font {} in list of available fonts", font_family);
                inner.font_family_chooser.set_selected_family(font);
            } else {
                warn!(
                    "Could not find font {}, using fallback \"{}\"",
                    font_family, FALLBACK_FONT_FAMILY
                );
                inner.font_family_chooser.set_selected_family(
                    font_families
                        .iter()
                        .find(|f| f.name() == FALLBACK_FONT_FAMILY)
                        .unwrap(),
                );
            }
        }

        if let Some(font_size) = font_size {
            debug!("Setting font size to {}", font_size);
            inner.font_size_scale.set_value(*font_size);
        }

        let WindowState {
            width,
            height,
            maximized,
        } = &state.window;

        if let Some(width) = width {
            debug!("Setting window width to {}", width);
            self.set_default_width(*width);
        }

        if let Some(height) = height {
            debug!("Setting window height to {}", height);
            self.set_default_height(*height);
        }

        if let Some(maximized) = maximized {
            debug!("Setting window maximized to {}", maximized);
            self.set_maximized(*maximized);
        }

        debug!("Finished loading config");
    }

    fn load_themes(&self, app: &QuellcodeApplication) {
        let inner = self.imp();
        inner.state_mut().themes = app.theme_set().themes.clone();

        let theme_list = gtk::StringList::new(
            &inner
                .state()
                .themes
                .iter()
                .map(|t| t.0.as_str())
                .collect::<Vec<_>>(),
        );

        inner.theme_dropdown.set_model(Some(&theme_list));
    }

    fn load_syntaxes(&self, app: &QuellcodeApplication) {
        let inner = self.imp();
        let syntax_set = app.syntax_set().clone();
        let syntax_list = gtk::StringList::new(
            &syntax_set
                .syntaxes()
                .iter()
                .map(|s| s.name.as_str())
                .collect::<Vec<_>>(),
        );

        inner.state_mut().syntax_set = Some(syntax_set);
        inner.syntax_dropdown.set_model(Some(&syntax_list));
    }

    fn setup_actions(&self, code_tx: async_channel::Sender<String>) {
        let generate_code = ActionEntry::builder("generate-code")
            .activate(move |window: &Self, _, _| {
                let inner = window.imp();
                if let Some(generator) = inner.generator() {
                    let editor = &inner.editor.clone();
                    let editor_syntax = editor.syntax().clone();
                    let editor_syntax_set = editor.syntax_set().clone();
                    let editor_theme = editor.theme().clone();
                    let font_size = editor.font_size();
                    let font_family = editor.font_family();
                    let text = editor.buffer().text(
                        &editor.buffer().start_iter(),
                        &editor.buffer().end_iter(),
                        true,
                    );

                    let code_tx = code_tx.clone();
                    gio::spawn_blocking(move || {
                        let mut generator = generator.lock().unwrap();
                        generator.set_font_size(font_size as f32);
                        generator.set_font_family(&font_family);

                        if let (Some(editor_syntax), Some(editor_theme)) =
                            (editor_syntax, editor_theme)
                        {
                            let generated_svg = generator.generate_code(
                                &text,
                                &editor_theme,
                                &editor_syntax,
                                &editor_syntax_set,
                            );

                            match generated_svg {
                                Ok(svg) => {
                                    code_tx.send_blocking(svg).expect("Failed to send svg");
                                }
                                Err(err) => {
                                    warn!("Failed to generate svg: {}", err);
                                }
                            }
                        }
                    });
                }
            })
            .build();

        self.add_action_entries([generate_code]);
    }
}
