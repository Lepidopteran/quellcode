use std::cell::Ref;

use gtk::{
    gdk::RGBA,
    glib::{self, Object},
    prelude::*,
    subclass::prelude::{ObjectSubclassExt, ObjectSubclassIsExt},
    TextTag,
};

use log::warn;
use syntect::{
    highlighting::{HighlightState, Highlighter, RangedHighlightIterator, Theme},
    parsing::{ParseState, ScopeStack, SyntaxReference, SyntaxSet},
};

pub mod imp {
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
    };

    use super::*;
    use gtk::{
        glib::{subclass::prelude::*, Properties},
        pango,
        prelude::{TextViewExt, WidgetExt},
        subclass::{
            prelude::TextViewImpl,
            widget::{WidgetClassExt, WidgetImpl},
        },
    };

    #[derive(Properties)]
    #[properties(wrapper_type = super::CodeView)]
    pub struct CodeView {
        pub syntax_set: Rc<RefCell<SyntaxSet>>,
        pub syntax: Rc<RefCell<Option<SyntaxReference>>>,
        pub theme: Rc<RefCell<Option<Theme>>>,
        #[property(get, set = set_font_family)]
        pub font_family: RefCell<String>,
        #[property(get, set = set_font_size)]
        pub font_size: Cell<f64>,
        #[property(get, set)]
        pub tab_width: Cell<u32>,
    }

    impl Default for CodeView {
        fn default() -> Self {
            Self {
                syntax_set: Rc::new(RefCell::new(SyntaxSet::load_defaults_nonewlines())),
                syntax: Rc::new(RefCell::new(None)),
                theme: Rc::new(RefCell::new(None)),
                font_family: RefCell::new("Monospace".to_string()),
                tab_width: Cell::new(4),
                font_size: Cell::new(12.0),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CodeView {
        const NAME: &'static str = "QuellcodeCodeView";
        type Type = super::CodeView;
        type ParentType = gtk::TextView;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("codeview");
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for CodeView {
        fn constructed(&self) {
            self.parent_constructed();

            let self_obj = self.obj().clone();
            set_tab_stops(&self_obj);

            self_obj.add_css_class("code");
            self_obj.set_monospace(true);

            let syntax_set = self.syntax_set.clone();
            let theme = self.theme.clone();
            let syntax = self.syntax.clone();
            let buffer = self_obj.buffer();
            let tag_table = buffer.tag_table();

            tag_table.connect_tag_changed(move |_, _, size_changed| {
                if size_changed {
                    set_tab_stops(&self_obj);
                }
            });

            buffer.create_tag(Some("global"), &[]);
            buffer.connect_changed(move |buffer| {
                let start_iter = buffer.start_iter();
                let end_iter = buffer.end_iter();
                let syntax_set = syntax_set.borrow();
                let theme = theme.borrow();
                let syntax = syntax.borrow();

                if let (Some(theme), Some(syntax)) = (theme.as_ref(), syntax.as_ref()) {
                    highlight_code(buffer, &start_iter, &end_iter, theme, &syntax_set, syntax);
                }
            });
        }
    }

    fn set_font_size(view: &CodeView, value: f64) {
        view.obj().global_tag().set_size((value as f32 * 0.75 * pango::SCALE as f32) as i32);
        view.font_size.set(value);
    }

    fn set_font_family(view: &CodeView, value: String) {
        view.obj().global_tag().set_family(Some(&value));
        view.font_family.replace(value);
    }

    fn set_tab_stops(view: &super::CodeView) {
        let width = calculate_tab_width(view, ' ');
        let mut tab_array = gtk::pango::TabArray::new(1, true);

        tab_array.set_tab(0, gtk::pango::TabAlign::Left, width);

        view.set_tabs(&tab_array);
    }

    fn calculate_tab_width(view: &super::CodeView, character: char) -> i32 {
        let text = character.to_string().repeat(view.tab_width() as usize);
        let layout = view.create_pango_layout(Some(&text));
        let font_desc = pango::FontDescription::from_string(
            format!("{} {}px", view.font_family(), view.font_size()).as_str(),
        );

        layout.set_font_description(Some(&font_desc));
        layout.pixel_size().0
    }

    impl WidgetImpl for CodeView {}

    impl TextViewImpl for CodeView {}
}

glib::wrapper! {
    pub struct CodeView(ObjectSubclass<imp::CodeView>)
        @extends gtk::TextView, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl CodeView {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn global_tag(&self) -> TextTag {
        self.buffer()
            .tag_table()
            .lookup("global")
            .expect("Failed to get global tag")
    }

    pub fn set_theme(&self, theme: Option<Theme>) {
        log::info!("Setting theme");
        self.imp().theme.replace(theme);
        if let (Some(theme), Some(syntax)) = (self.theme().as_ref(), self.syntax().as_ref()) {
            let buffer = self.buffer();
            highlight_code(
                &buffer,
                &buffer.start_iter(),
                &buffer.end_iter(),
                theme,
                &self.syntax_set(),
                syntax,
            )
        }
    }

    pub fn theme(&self) -> Ref<Option<Theme>> {
        self.imp().theme.borrow()
    }

    pub fn syntax(&self) -> Ref<Option<SyntaxReference>> {
        self.imp().syntax.borrow()
    }

    pub fn set_syntax(&self, syntax: Option<SyntaxReference>) {
        log::info!("Setting syntax");
        self.imp().syntax.replace(syntax);
        if let (Some(theme), Some(syntax)) = (self.theme().as_ref(), self.syntax().as_ref()) {
            let buffer = self.buffer();
            highlight_code(
                &buffer,
                &buffer.start_iter(),
                &buffer.end_iter(),
                theme,
                &self.syntax_set(),
                syntax,
            )
        }
    }

    pub fn set_syntax_set(&self, syntax_set: SyntaxSet) {
        self.imp().syntax_set.replace(syntax_set);
        if let (Some(theme), Some(syntax)) = (self.theme().as_ref(), self.syntax().as_ref()) {
            let buffer = self.buffer();
            highlight_code(
                &buffer,
                &buffer.start_iter(),
                &buffer.end_iter(),
                theme,
                &self.syntax_set(),
                syntax,
            )
        }
    }

    pub fn syntax_set(&self) -> Ref<SyntaxSet> {
        self.imp().syntax_set.borrow()
    }
}

impl Default for CodeView {
    fn default() -> Self {
        Self::new()
    }
}

fn highlight_code(
    buffer: &gtk::TextBuffer,
    start: &gtk::TextIter,
    end: &gtk::TextIter,
    theme: &Theme,
    syntax_set: &SyntaxSet,
    syntax: &SyntaxReference,
) {
    buffer.remove_all_tags(start, end);

    for (index, line) in buffer.text(start, end, true).as_str().lines().enumerate() {
        let highlighter = Highlighter::new(theme);
        let mut highlight_state = HighlightState::new(&highlighter, ScopeStack::new());
        let mut parse_state = ParseState::new(syntax);
        let operations = parse_state.parse_line(line, syntax_set).unwrap();

        let iter =
            RangedHighlightIterator::new(&mut highlight_state, &operations[..], line, &highlighter);

        for (style, _, range) in iter {
            let start = buffer.iter_at_line_offset(index as i32, range.start as i32);
            let end = buffer.iter_at_line_offset(index as i32, range.end as i32);
            if let (Some(start), Some(end)) = (start, end) {
                let tag = TextTag::builder()
                    .foreground_rgba(&RGBA::new(
                        style.foreground.r as f32 / 255.0,
                        style.foreground.g as f32 / 255.0,
                        style.foreground.b as f32 / 255.0,
                        style.foreground.a as f32 / 255.0,
                    ))
                    .background_rgba(&RGBA::new(
                        style.background.r as f32 / 255.0,
                        style.background.g as f32 / 255.0,
                        style.background.b as f32 / 255.0,
                        style.background.a as f32 / 255.0,
                    ))
                    .build();

                buffer.tag_table().add(&tag);

                buffer.apply_tag(&tag, &start, &end);
            } else {
                if start.is_none() {
                    warn!(
                        "Failed to get start iter for line {}, start: {}, end: {}",
                        index, range.start, range.end
                    )
                }
                if end.is_none() {
                    warn!(
                        "Failed to get end iter for line {}, start: {}, end: {}",
                        index, range.start, range.end
                    )
                }
            }
        }

        buffer.apply_tag_by_name("global", start, end);
    }
}
