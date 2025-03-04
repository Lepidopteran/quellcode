use color::get_color;
use log::debug;
use serde::Deserialize;
use std::{collections::HashMap, path::Path, str::FromStr};
use syntect::highlighting::{
    FontStyle, ScopeSelectors, StyleModifier, Theme, ThemeItem, ThemeSettings, UnderlineOption,
};

pub mod color;
pub mod error;
pub mod parser;

use crate::error::ParseError;

#[derive(Deserialize, Debug)]
pub struct Rule {
    pub name: Option<String>,
    pub scope: String,
    pub font_style: Option<String>,
    pub foreground: Option<String>,
    pub background: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ColorScheme {
    pub name: Option<String>,
    pub author: Option<String>,
    pub variables: Option<HashMap<String, String>>,
    pub globals: HashMap<String, String>,
    pub rules: Vec<Rule>,
}

impl FromStr for ColorScheme {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // NOTE: Need to remove comments and remove leading commas since JSON doesn't support them.
        let value: serde_json::Value = jsonc_parser::parse_to_ast(
            s,
            &jsonc_parser::CollectOptions {
                comments: jsonc_parser::CommentCollectionStrategy::Off,
                tokens: false,
            },
            &jsonc_parser::ParseOptions::default(),
        )?
        .value
        .into();

        serde_json::from_value(value).map_err(ParseError::Json)
    }
}

impl TryFrom<ColorScheme> for Theme {
    type Error = ParseError;
    fn try_from(value: ColorScheme) -> Result<Self, Self::Error> {
        let mut settings = ThemeSettings::default();
        let variables = value.variables.unwrap_or_default();

        for (key, value) in &value.globals {
            debug!("Got global: {} = {}", key, value);
            match &key[..] {
                "foreground" => settings.foreground = get_color(value, &variables).ok(),
                "background" => settings.background = get_color(value, &variables).ok(),
                "caret" => settings.caret = get_color(value, &variables).ok(),
                "line_highlight" => settings.line_highlight = get_color(value, &variables).ok(),
                "misspelling" => settings.misspelling = get_color(value, &variables).ok(),
                "minimap_border" => settings.minimap_border = get_color(value, &variables).ok(),
                "accent" => settings.accent = get_color(value, &variables).ok(),
                "popup_css" => settings.popup_css = Some(value.clone()),
                "phantom_css" => settings.phantom_css = Some(value.clone()),
                "bracket_contents_foreground" => {
                    settings.bracket_contents_foreground = get_color(value, &variables).ok()
                }
                "bracket_contents_options" => {
                    settings.bracket_contents_options = UnderlineOption::from_str(value).ok()
                }
                "brackets_foreground" => {
                    settings.brackets_foreground = get_color(value, &variables).ok()
                }
                "brackets_background" => {
                    settings.brackets_background = get_color(value, &variables).ok()
                }
                "brackets_options" => {
                    settings.brackets_options = UnderlineOption::from_str(value).ok()
                }
                "tags_foreground" => settings.tags_foreground = get_color(value, &variables).ok(),
                "tags_options" => settings.tags_options = UnderlineOption::from_str(value).ok(),
                "highlight" => settings.highlight = get_color(value, &variables).ok(),
                "find_highlight" => settings.find_highlight = get_color(value, &variables).ok(),
                "find_highlight_foreground" => {
                    settings.find_highlight_foreground = get_color(value, &variables).ok()
                }
                "gutter" => settings.gutter = get_color(value, &variables).ok(),
                "gutter_foreground" => {
                    settings.gutter_foreground = get_color(value, &variables).ok()
                }
                "selection" => settings.selection = get_color(value, &variables).ok(),
                "selection_foreground" => {
                    settings.selection_foreground = get_color(value, &variables).ok()
                }
                "selection_border" => settings.selection_border = get_color(value, &variables).ok(),
                "inactive_selection" => {
                    settings.inactive_selection = get_color(value, &variables).ok()
                }
                "inactive_selection_foreground" => {
                    settings.inactive_selection_foreground = get_color(value, &variables).ok()
                }
                "guide" => settings.guide = get_color(value, &variables).ok(),
                "active_guide" => settings.active_guide = get_color(value, &variables).ok(),
                "stack_guide" => settings.stack_guide = get_color(value, &variables).ok(),
                "shadow" => settings.shadow = get_color(value, &variables).ok(),
                _ => (), // E.g. "shadowWidth" and "invisibles" are ignored
            }
        }

        Ok(Self {
            name: value.name,
            author: value.author,
            settings,
            scopes: value
                .rules
                .into_iter()
                .map(|rule| {
                    Ok(ThemeItem {
                        scope: ScopeSelectors::from_str(&rule.scope)?,
                        style: StyleModifier {
                            foreground: rule
                                .foreground
                                .map(|s| get_color(&s, &variables))
                                .transpose()?,
                            background: rule
                                .background
                                .map(|s| get_color(&s, &variables))
                                .transpose()?,
                            font_style: rule
                                .font_style
                                .map(|s| FontStyle::from_str(&s))
                                .transpose()?,
                        },
                    })
                })
                .collect::<Result<Vec<_>, ParseError>>()?,
        })
    }
}

/// Parse a color scheme from a string.
///
/// Equivalent to calling [ColorScheme::from_str]
pub fn parse_color_scheme(scheme: &str) -> Result<ColorScheme, ParseError> {
    ColorScheme::from_str(scheme)
}

/// Parse a color scheme from a file.
///
/// Equivalent to calling [parse_color_scheme]
pub fn parse_color_scheme_file(path: &Path) -> Result<ColorScheme, ParseError> {
    let scheme = std::fs::read_to_string(path)?;
    parse_color_scheme(&scheme)
}

#[cfg(test)]
mod tests {
    use log::debug;

    use super::*;

    fn start_log() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn convert_theme_without_variables() {
        start_log();
        let schemes = vec![
            include_str!("../assets/schemes/ayu-dark.sublime-color-scheme"),
            include_str!("../assets/schemes/ayu-mirage.sublime-color-scheme"),
            include_str!("../assets/schemes/ayu-light.sublime-color-scheme"),
        ];

        for scheme in schemes {
            let scheme = ColorScheme::from_str(scheme).expect("Failed to parse theme");

            Theme::try_from(scheme).expect("Failed to convert to theme");
        }
    }

    #[test]
    fn convert_theme_with_variables() {
        start_log();

        let mut success = 0;
        let schemes = vec![
            include_str!("../assets/schemes/Catppuccin Latte.sublime-color-scheme"),
            include_str!("../assets/schemes/Catppuccin Mocha.sublime-color-scheme"),
            include_str!("../assets/schemes/Catppuccin Frappe.sublime-color-scheme"),
            include_str!("../assets/schemes/Catppuccin Macchiato.sublime-color-scheme"),
            include_str!("../assets/schemes/Gruvbox Material Dark.sublime-color-scheme"),
            include_str!("../assets/schemes/Kanagawa.sublime-color-scheme"),
            include_str!("../assets/schemes/Nord.sublime-color-scheme"),
        ];

        for scheme in schemes {
            let scheme = ColorScheme::from_str(scheme).expect("Failed to parse theme");

            Theme::try_from(scheme).expect("Failed to convert to theme");
            success += 1;

            debug!("Successfully converted {} themes", success);
        }
    }
}
