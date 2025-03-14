use std::{cell::RefCell, rc::Rc};

use gtk::{
    glib::{self, Object},
    prelude::*,
    subclass::prelude::ObjectSubclassIsExt,
};
use syntect::{
    highlighting::Theme,
    parsing::{SyntaxReference, SyntaxSet},
};

pub mod imp {
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
    };

    use super::*;
    use glib::Properties;
    use gtk::{
        gdk::RGBA,
        glib::subclass::prelude::*,
        pango,
        prelude::{TextViewExt, WidgetExt},
        subclass::{
            prelude::{TextBufferImpl, TextBufferImplExt, TextViewImpl, TextViewImplExt},
            widget::WidgetImpl,
        },
        TextTag,
    };
    use log::debug;
    use syntect::{
        easy::HighlightLines,
        highlighting::{
            FontStyle, HighlightIterator, HighlightState, Highlighter, RangedHighlightIterator,
            ScopeSelectors, Theme, ThemeSet,
        },
        parsing::{ParseState, ScopeStack, SyntaxReference, SyntaxSet, SCOPE_REPO},
        util::LinesWithEndings,
    };

    #[derive(Properties, Default)]
    #[properties(wrapper_type = super::CodeView)]
    pub struct CodeView {
        pub theme: Rc<RefCell<Theme>>,
        pub theme_set: Rc<RefCell<ThemeSet>>,
        pub syntax_set: Rc<RefCell<SyntaxSet>>,
        pub syntax: Rc<RefCell<Option<SyntaxReference>>>,

        #[property(name = "theme-name", get, set)]
        pub theme_name: RefCell<String>,

        #[property(name = "language-syntax-name", get, set)]
        pub syntax_name: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CodeView {
        const NAME: &'static str = "QuellcodeCodeView";
        type Type = super::CodeView;
        type ParentType = gtk::TextView;
    }

    #[glib::derived_properties]
    impl ObjectImpl for CodeView {
        fn constructed(&self) {
            self.parent_constructed();

            self.obj().add_css_class("code");
            self.obj().set_wrap_mode(gtk::WrapMode::WordChar);
            self.obj().set_monospace(true);
            self.obj().buffer().connect_changed(highlight_code);
        }
    }

    impl WidgetImpl for CodeView {}

    impl TextViewImpl for CodeView {}

    fn highlight_code(buffer: &gtk::TextBuffer) {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        let syntax = syntax_set.find_syntax_by_name("Rust").unwrap();
        let theme = &theme_set.themes["base16-eighties.dark"];

        buffer.remove_all_tags(&buffer.start_iter(), &buffer.end_iter());

        for (index, line) in buffer
            .text(&buffer.start_iter(), &buffer.end_iter(), true)
            .as_str()
            .lines()
            .enumerate()
        {
            let highlighter = Highlighter::new(theme);
            let mut highlight_state = HighlightState::new(&highlighter, ScopeStack::new());
            let mut parse_state = ParseState::new(syntax);
            let operations = parse_state.parse_line(line, &syntax_set).unwrap();

            let iter = RangedHighlightIterator::new(
                &mut highlight_state,
                &operations[..],
                line,
                &highlighter,
            );

            for (style, _, range) in iter {
                let start = buffer.iter_at_line_offset(index as i32, range.start as i32).unwrap();
                let end = buffer.iter_at_line_offset(index as i32, range.end as i32).unwrap();

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
                    )).build();

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

    pub fn set_theme(&self, theme: Theme) {
        self.imp().theme.replace(theme);
    }

    pub fn set_syntax(&self, syntax: SyntaxReference) {
        self.imp().syntax.replace(Some(syntax));
    }

    pub fn set_syntax_set(&self, syntax_set: SyntaxSet) {
        self.imp().syntax_set.replace(syntax_set);
    }
}

impl Default for CodeView {
    fn default() -> Self {
        Self::new()
    }
}
