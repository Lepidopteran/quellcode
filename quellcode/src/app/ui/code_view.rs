use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

use gtk::{
    gdk::RGBA,
    glib::{self, Object},
    pango,
    prelude::*,
    subclass::prelude::{ObjectSubclassExt, ObjectSubclassIsExt},
    TextTag,
};
use log::debug;
use syntect::{
    highlighting::{
        FontStyle, HighlightState, Highlighter, RangedHighlightIterator, ScopeSelectors, Theme,
        ThemeSet,
    },
    parsing::{ParseState, ScopeStack, SyntaxReference, SyntaxSet, SCOPE_REPO},
};

pub mod imp {
    use std::{cell::RefCell, rc::Rc};

    use super::*;
    use glib::Properties;
    use gtk::{
        glib::subclass::prelude::*,
        prelude::{TextViewExt, WidgetExt},
        subclass::{
            prelude::TextViewImpl,
            widget::{WidgetClassExt, WidgetImpl},
        },
    };

    #[derive(Default)]
    pub struct CodeView {
        pub syntax_set: RefCell<SyntaxSet>,
        pub syntax: RefCell<Option<SyntaxReference>>,
        pub theme: RefCell<Option<Theme>>,
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

    impl ObjectImpl for CodeView {
        fn constructed(&self) {
            self.parent_constructed();

            let buffer = self.obj().buffer();
            buffer.create_tag(Some("global"), &[]);

            self.syntax_set
                .replace(SyntaxSet::load_defaults_nonewlines());

            self.obj().add_css_class("code");
            self.obj().set_wrap_mode(gtk::WrapMode::WordChar);
            self.obj().set_monospace(true);

            let syntax_set = self.syntax_set.clone();
            let theme = self.theme.clone();
            let syntax = self.syntax.clone();

            buffer.connect_changed(move |buffer| {
                let start_iter = buffer.start_iter();
                let end_iter = buffer.end_iter();
                let syntax_set = syntax_set.borrow();
                let theme = theme.borrow();
                let syntax = syntax.borrow();

                if let (Some(theme), Some(syntax)) = (theme.as_ref(), syntax.as_ref()) {
                    highlight_code(buffer, &start_iter, &end_iter, theme, &syntax_set, syntax);
                }

                buffer.apply_tag_by_name("global", &start_iter, &end_iter);
            });
        }
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
            let start = buffer
                .iter_at_line_offset(index as i32, range.start as i32)
                .expect("Failed to get start iter");
            let end = buffer
                .iter_at_line_offset(index as i32, range.end as i32)
                .expect("Failed to get end iter");

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
        }
    }
}

fn create_tags_from_scopes(buffer: &gtk::TextBuffer, theme: &Theme) {
    for rule in &theme.scopes {
        let style = rule.style;
        let font_style = style.font_style;

        let underline_style = if font_style.unwrap_or_default() == FontStyle::UNDERLINE {
            pango::Underline::Single
        } else {
            pango::Underline::None
        };

        let italic = if font_style.unwrap_or_default() == FontStyle::ITALIC {
            pango::Style::Italic
        } else {
            pango::Style::Normal
        };

        let weight = if font_style.unwrap_or_default() == FontStyle::BOLD {
            700
        } else {
            400
        };

        buffer.create_tag(
            Some(&scope_to_tag_name(&rule.scope)),
            &[
                (
                    "background",
                    &style
                        .background
                        .map(|color| rgba_to_hex(color.r, color.g, color.b, color.a)),
                ),
                (
                    "foreground",
                    &style
                        .foreground
                        .map(|color| rgba_to_hex(color.r, color.g, color.b, color.a)),
                ),
                ("underline", &underline_style),
                ("style", &italic),
                ("weight", &weight.to_value()),
            ],
        );
    }
}

fn scope_to_tag_name(scope: &ScopeSelectors) -> String {
    let mut name = String::new();
    for selector in &scope.selectors {
        let scopes = selector.extract_scopes();

        for scope in &scopes {
            let repo = SCOPE_REPO.lock().expect("Failed to get scope repo");
            for i in 0..(scope.len()) {
                let atom = scope.atom_at(i as usize);
                let atom_s = repo.atom_str(atom);
                name.push('.');
                name.push_str(atom_s);
            }

            name.push(' ')
        }

        name.pop();
        name.push_str(", ");
    }

    let len = name.len();
    name.truncate(len - 2);

    println!("{}", name);
    name
}

fn rgba_to_hex(r: u8, g: u8, b: u8, a: u8) -> String {
    format!("#{:02x}{:02x}{:02x}{:02x}", r, g, b, a)
}
