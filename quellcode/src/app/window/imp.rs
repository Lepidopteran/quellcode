use evalexpr::*;
use log::{debug, error};
use std::{
    cell::{Ref, RefCell, RefMut},
    collections::BTreeMap,
    rc::Rc,
    sync::{Arc, Mutex},
    time::Duration,
};

use gtk::{
    gdk::Display,
    gio::{self, ListStore, SimpleAction},
    glib::{self, subclass::InitializingObject, Properties},
    prelude::*,
    subclass::prelude::*,
    CompositeTemplate, FileDialog, TemplateChild,
};

use crate::app::{
    state::{save_state, CodeState, State as AppState, WindowState},
    property::PropertyType,
    ui::code_view::CodeView,
};

use super::*;

type Generator = Arc<Mutex<dyn GeneratorTrait>>;

const UNITS: &[&str] = &["px", "pt", "pc", "in", "cm", "mm"];
const ROUND_DIGITS: i32 = 4;
const FONT_SCALE_DEBOUNCE_DELAY: Duration = Duration::from_millis(200);
const BUFFER_DEBOUNCE_DELAY: Duration = Duration::from_millis(300);

#[derive(Default, Debug)]
pub struct State {
    pub syntax_set: Option<SyntaxSet>,
    pub themes: BTreeMap<String, Theme>,
    generator: Option<Generator>,
    css_provider: gtk::CssProvider,
    font_scale_debounce_id: Option<glib::SourceId>,
    buffer_debounce_id: Option<glib::SourceId>,
}

#[derive(CompositeTemplate, Default, Properties)]
#[properties(wrapper_type = super::Window)]
#[template(resource = "/org/quellcode/quellcode/window.ui")]
pub struct Window {
    state: Rc<RefCell<State>>,

    // Children
    #[template_child]
    pub content: TemplateChild<gtk::Box>,

    #[template_child]
    pub inspector: TemplateChild<gtk::Box>,

    #[template_child]
    pub layout: TemplateChild<gtk::Grid>,

    #[template_child]
    editor_viewer_layout: TemplateChild<gtk::Grid>,

    #[template_child]
    pub editor_scroll: TemplateChild<gtk::ScrolledWindow>,

    #[template_child]
    pub editor: TemplateChild<CodeView>,

    #[template_child]
    editor_separator: TemplateChild<gtk::Separator>,

    #[template_child]
    viewer_overlay: TemplateChild<gtk::Overlay>,

    #[template_child]
    pub viewer_stack: TemplateChild<gtk::Stack>,

    #[template_child]
    pub viewer: TemplateChild<CodeView>,

    #[template_child]
    pub viewer_loading_spinner: TemplateChild<gtk::Spinner>,

    #[template_child]
    pub viewer_loading_box: TemplateChild<gtk::Box>,

    #[template_child]
    pub generator_dropdown: TemplateChild<gtk::DropDown>,

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
    pub fn state(&self) -> Ref<State> {
        self.state.borrow()
    }

    pub fn state_mut(&self) -> RefMut<State> {
        self.state.borrow_mut()
    }

    /// Returns the current generator
    pub fn generator(&self) -> Option<Generator> {
        self.state().generator.clone()
    }

    /// Sets the generator and updates the UI
    pub fn set_generator(&self, generator: Option<Generator>) {
        self.state_mut().generator = generator;
        self.display_generator_properties();
    }

    fn display_generator_properties(&self) {
        let container = self.generator_box.clone();
        let state = self.state.clone();

        if let Some(generator) = &state.borrow().generator {
            for property in generator.lock().unwrap().properties().clone() {
                debug!("Property: {}", property.name);
                let generator = generator.clone();
                match property.kind {
                    PropertyType::Bool => {
                        let check = gtk::CheckButton::builder()
                            .label(create_property_display_name(property.name))
                            .active(
                                property
                                    .default
                                    .clone()
                                    .is_some_and(|v| v.try_into().unwrap()),
                            )
                            .build();

                        check.connect_toggled(move |check| {
                            let _ = generator
                                .lock()
                                .unwrap()
                                .set_property(property.name, check.is_active().into());
                            let _ = check.activate_action("win.generate-code", None);
                        });

                        container.append(&check);
                    }
                    PropertyType::String => {
                        let wrapper = gtk::Box::builder()
                            .orientation(gtk::Orientation::Vertical)
                            .build();

                        let label = gtk::Label::builder()
                            .label(create_property_display_name(property.name))
                            .build();

                        let entry = gtk::Entry::builder()
                            .text(
                                property
                                    .default
                                    .clone()
                                    .unwrap_or_else(|| "".into())
                                    .to_string(),
                            )
                            .build();

                        entry.connect_activate(move |entry| {
                            let _ = generator
                                .lock()
                                .unwrap()
                                .set_property(property.name, entry.text().to_string().into());
                            let _ = entry.activate_action("win.generate-code", None);
                        });

                        wrapper.append(&label);
                        wrapper.append(&entry);

                        container.append(&wrapper);
                    }
                    PropertyType::Float => {
                        let wrapper = gtk::Box::builder()
                            .orientation(gtk::Orientation::Vertical)
                            .build();

                        let label = gtk::Label::builder()
                            .label(create_property_display_name(property.name))
                            .build();

                        let adjustment = gtk::Adjustment::new(
                            property
                                .default
                                .clone()
                                .unwrap_or_else(|| 0.0.into())
                                .try_into()
                                .unwrap(),
                            property
                                .min
                                .unwrap_or_else(|| 0.0.into())
                                .try_into()
                                .unwrap(),
                            property
                                .max
                                .unwrap_or_else(|| 0.0.into())
                                .try_into()
                                .unwrap(),
                            0.0,
                            0.0,
                            0.0,
                        );

                        let spinner = gtk::SpinButton::builder()
                            .adjustment(&adjustment)
                            .digits(2)
                            .climb_rate(0.1)
                            .build();

                        spinner.connect_value_changed(move |spinner| {
                            let _ = generator
                                .lock()
                                .unwrap()
                                .set_property(property.name, spinner.value().to_string().into());
                            let _ = spinner.activate_action("win.generate-code", None);
                        });

                        wrapper.append(&label);
                        wrapper.append(&spinner);

                        container.append(&wrapper);
                    }
                    PropertyType::Int => {
                        let wrapper = gtk::Box::builder()
                            .orientation(gtk::Orientation::Vertical)
                            .build();

                        let label = gtk::Label::builder()
                            .label(create_property_display_name(property.name))
                            .build();

                        let adjustment = gtk::Adjustment::new(
                            property
                                .default
                                .clone()
                                .unwrap_or_else(|| 0.0.into())
                                .try_into()
                                .unwrap(),
                            property
                                .min
                                .unwrap_or_else(|| 0.0.into())
                                .try_into()
                                .unwrap(),
                            property
                                .max
                                .unwrap_or_else(|| 0.0.into())
                                .try_into()
                                .unwrap(),
                            1.0,
                            0.0,
                            0.0,
                        );

                        let spinner = gtk::SpinButton::builder()
                            .adjustment(&adjustment)
                            .digits(0)
                            .climb_rate(1.0)
                            .build();

                        spinner.connect_value_changed(move |spinner| {
                            let _ = generator
                                .lock()
                                .unwrap()
                                .set_property(property.name, spinner.value().to_string().into());
                            let _ = spinner.activate_action("win.generate-code", None);
                        });

                        wrapper.append(&label);
                        wrapper.append(&spinner);

                        container.append(&wrapper);
                    }
                }
            }
        };
    }

    #[template_callback] 
    fn generator_changed(&self) {

    }

    #[template_callback]
    fn theme_changed(&self) {
        let state = self.state.borrow();
        let theme_dropdown = self.theme_dropdown.clone();
        let name = theme_dropdown
            .model()
            .expect("Could not get model")
            .downcast_ref::<gtk::StringList>()
            .expect("Could not downcast model")
            .string(theme_dropdown.selected())
            .map(|s| s.to_string())
            .expect("Could not get syntax name");

        let (_, theme) = state
            .themes
            .iter()
            .find(|t| *t.0 == name)
            .expect("Could not find theme");

        state
            .css_provider
            .load_from_string(&theme_to_gtk_css(theme));

        self.editor.set_theme(Some(theme.clone()));

        let _ = self
            .theme_dropdown
            .activate_action("win.generate-code", None);
    }

    #[template_callback]
    fn syntax_changed(&self) {
        let syntax_dropdown = self.syntax_dropdown.clone();
        if let Some(syntax_set) = self.state.borrow().syntax_set.clone() {
            let name = syntax_dropdown
                .model()
                .expect("Could not get model")
                .downcast_ref::<gtk::StringList>()
                .expect("Could not downcast model")
                .string(syntax_dropdown.selected())
                .map(|s| s.to_string())
                .expect("Could not get syntax name");

            let syntax = syntax_set
                .find_syntax_by_name(&name)
                .expect("Could not find syntax");

            self.editor.set_syntax(Some(syntax.clone()));
        }

        let _ = syntax_dropdown.activate_action("win.generate-code", None);
    }

    #[template_callback]
    fn font_size_scale_changed(&self) {
        let font_size_scale = self.font_size_scale.clone();
        let value = font_size_scale.value();

        self.font_size_entry.set_text(&value.to_string());
        self.editor.set_font_size(value);

        let state = self.state.clone();
        let id = glib::timeout_add_local(FONT_SCALE_DEBOUNCE_DELAY, move || {
            let _ = font_size_scale.activate_action("win.generate-code", None);
            state.borrow_mut().font_scale_debounce_id = None;
            glib::ControlFlow::Break
        });

        let mut state = self.state.borrow_mut();
        if let Some(debounce_id) = state.font_scale_debounce_id.take() {
            debounce_id.remove();
        }

        state.font_scale_debounce_id = Some(id);
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

    #[template_callback]
    fn font_changed(&self) {
        let font_family_chooser = self.font_family_chooser.clone();
        let family = font_family_chooser.selected_family().clone();

        if let Some(family) = family {
            self.editor.set_font_family(family.name());
        }

        let _ = self
            .font_family_chooser
            .activate_action("win.generate-code", None);
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

#[glib::derived_properties]
impl ObjectImpl for Window {
    fn constructed(&self) {
        self.parent_constructed();
        let provider: &gtk::CssProvider = &self.state.borrow().css_provider;
        gtk::style_context_add_provider_for_display(
            &Display::default().expect("Failed to get display"),
            provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

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

        let import_action = SimpleAction::new("import-file", None);
        let outer_self = self.obj().clone();
        import_action.connect_activate(move |_, _| import_file(&outer_self));

        let export_action = SimpleAction::new("export-generated-code", None);
        let outer_self = self.obj().clone();
        export_action.connect_activate(move |_, _| export_generated_code(&outer_self));

        let layout_action = SimpleAction::new_stateful(
            "change_layout",
            Some(&String::static_variant_type()),
            &"split_horizontal".to_variant(),
        );

        let grid = self.editor_viewer_layout.clone();
        let editor_container = self.editor_scroll.clone();
        let separator = self.editor_separator.clone();
        let viewer_container = self.viewer_overlay.clone();

        layout_action.connect_activate(move |action, property| {
            if let Some(layout) = property {
                let layout_id = layout.get::<String>().expect("Failed to get layout");
                if action.state().is_some_and(|state| state == *layout) {
                    return;
                }

                action.set_state(layout);
                match layout_id.as_str() {
                    "split_horizontal" => {
                        position_grid_child(&grid, &editor_container, 0, 0, 1, 1);
                        position_grid_child(&grid, &separator, 1, 0, 1, 1);
                        position_grid_child(&grid, &viewer_container, 2, 0, 1, 1);
                    }
                    "split_vertical" => {
                        position_grid_child(&grid, &editor_container, 1, 0, 1, 1);
                        position_grid_child(&grid, &separator, 1, 1, 1, 1);
                        position_grid_child(&grid, &viewer_container, 1, 2, 1, 1);
                    }
                    "editor" => {
                        position_grid_child(&grid, &editor_container, 0, 0, 1, 1);
                        grid.remove(&viewer_container);
                        if separator.parent().is_some_and(|parent| parent == grid) {
                            grid.remove(&separator);
                        }
                    }
                    "viewer" => {
                        position_grid_child(&grid, &viewer_container, 0, 0, 1, 1);
                        grid.remove(&editor_container);
                        if separator.parent().is_some_and(|parent| parent == grid) {
                            grid.remove(&separator);
                        }
                    }
                    _ => {}
                }
            }
        });

        let editor = self.editor.clone();
        let state = self.state.clone();
        self.editor.buffer().connect_changed(move |_| {
            if let Some(debounce_id) = state.borrow_mut().buffer_debounce_id.take() {
                debounce_id.remove();
            }

            let editor = editor.clone();
            let state_clone = state.clone();
            let id = glib::timeout_add_local(BUFFER_DEBOUNCE_DELAY, move || {
                let _ = editor.activate_action("win.generate-code", None);
                state_clone.borrow_mut().buffer_debounce_id = None;
                glib::ControlFlow::Break
            });

            let state = state.clone();
            state.borrow_mut().buffer_debounce_id = Some(id);
        });

        let (code_tx, code_rx) = async_channel::unbounded::<String>();

        let viewer = self.viewer.clone();
        let viewer_loading_box = self.viewer_loading_box.clone();

        glib::spawn_future_local(async move {
            while let Ok(svg) = code_rx.recv().await {
                viewer.buffer().set_text(svg.as_str());
                viewer_loading_box.set_visible(false);
            }
        });

        let self_obj = self.obj().clone();

        self_obj.connect_close_request(|window| {
            let editor = window.imp().editor.clone();
            let state = AppState {
                code: CodeState {
                    theme: Some(
                        editor
                            .theme()
                            .clone()
                            .map(|t| t.name.unwrap_or_default())
                            .unwrap_or_default(),
                    ),
                    syntax: Some(editor.syntax().clone().map(|s| s.name).unwrap_or_default()),
                    font_family: Some(editor.font_family()),
                    font_size: Some(editor.font_size()),
                },
                window: WindowState {
                    width: Some(window.width()),
                    height: Some(window.height()),
                    maximized: Some(window.is_maximized()),
                }
            };

            if let Err(err) = save_state(&state) {
                error!("Failed to save state: {}", err);
            } else {
                log::info!("Saved state: {:#?}, exiting...", state);
            }

            glib::Propagation::Proceed
        });

        self_obj.add_action(&layout_action);
        self_obj.add_action(&import_action);
        self_obj.add_action(&export_action);
        self_obj.setup_actions(code_tx);

        self.inspector.set_size_request(300, -1);
    }
}
impl WidgetImpl for Window {}
impl WindowImpl for Window {}
impl ApplicationWindowImpl for Window {}

fn create_property_display_name(input: &str) -> String {
    input
        .replace("_", " ")
        .split_whitespace()
        .map(|word| {
            let mut c = word.chars();
            match c.next() {
                Some(first) => first.to_uppercase().chain(c).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

fn import_file(window: &super::Window) {
    let default_filter = gtk::FileFilter::new();
    default_filter.add_mime_type("text/plain");
    default_filter.set_name(Some("Plain Text"));

    let any_filter = gtk::FileFilter::new();
    any_filter.add_pattern("*");
    any_filter.set_name(Some("Any"));

    let list = ListStore::new::<gtk::FileFilter>();
    list.append(&default_filter);
    list.append(&any_filter);

    let dialog = FileDialog::builder()
        .title("Import File")
        .filters(&list)
        .default_filter(&default_filter)
        .build();

    let buffer = window.imp().editor.buffer().clone();
    dialog.open(Some(window), None::<&gio::Cancellable>, move |result| {
        if let Ok(file) = result {
            let path = file.path();
            let text = std::fs::read_to_string(path.unwrap()).unwrap();

            buffer.set_text(&text);
        }
    });
}

fn export_generated_code(window: &super::Window) {
    let svg_filter = gtk::FileFilter::new();
    svg_filter.add_mime_type("text/plain");
    svg_filter.set_name(Some("SVG"));
    svg_filter.add_pattern("*.svg");

    let any_filter = gtk::FileFilter::new();
    any_filter.add_pattern("*");
    any_filter.set_name(Some("Any"));

    let list = ListStore::new::<gtk::FileFilter>();
    list.append(&svg_filter);
    list.append(&any_filter);

    let viewer = window.imp().viewer.clone();
    let text = viewer
        .buffer()
        .text(
            &viewer.buffer().start_iter(),
            &viewer.buffer().end_iter(),
            true,
        )
        .to_string();

    let dialog = gtk::FileDialog::builder()
        .filters(&list)
        .title("Save Generated Code")
        .build();

    dialog.save(
        Some(window),
        None::<&gtk::gio::Cancellable>,
        move |result| {
            if let Ok(file) = result {
                if let Some(path) = file.path() {
                    let mut path = path.to_path_buf();
                    if path.extension().is_none() {
                        path.set_extension("svg");
                    }

                    if let Err(err) = std::fs::write(path, text.as_bytes()) {
                        error!("Failed to write to file, Error:\n{}", err);
                    }
                }
            }
        },
    );
}

fn position_grid_child(
    grid: &gtk::Grid,
    child: &impl IsA<gtk::Widget>,
    row: i32,
    column: i32,
    row_span: i32,
    column_span: i32,
) {
    if child.parent().is_some_and(|parent| parent == *grid) {
        grid.remove(child);
    }
    grid.attach(child, column, row, column_span, row_span);
}

/// Generate gtk css theme for a [Theme]
///
/// > Does not generate scopes as [gtk::TextView] uses [gtk::TextTag] for syntax highlighting.
fn theme_to_gtk_css(theme: &Theme) -> String {
    let mut css = String::new();

    css.push_str("/*\n");
    let name = theme
        .name
        .clone()
        .unwrap_or_else(|| "unknown theme".to_string());

    css.push_str(&format!(" * theme \"{}\" generated by Quellcode\n", name));
    css.push_str(" */\n\n");
    css.push_str(".code {\n");

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
        css.push_str(".code text selection {\n");

        css.push_str(&format!(
            " background-color: rgb({} {} {} / 0.5);\n",
            selection.r, selection.g, selection.b
        ));

        css.push_str("}\n\n");
    }

    css
}

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
