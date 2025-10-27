use log::debug;
use serde::Deserialize;
use std::{collections::HashMap, path::Path, str::FromStr};
use syntect::highlighting::{
    Color, FontStyle, ScopeSelectors, StyleModifier, Theme, ThemeItem, ThemeSettings,
};

pub mod error;
pub mod named_color;

use crate::error::ParseError;

/// A token color
#[derive(Debug, Deserialize)]
pub struct TokenColor {
    pub scope: Option<Scope>,
    pub settings: TokenSettings,
}

/// A scope of a token
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Scope {
    Single(String),
    Multiple(Vec<String>),
}

/// The settings of a token
#[derive(Debug, Deserialize)]
pub struct TokenSettings {
    pub foreground: Option<String>,
    pub background: Option<String>,
    pub font_style: Option<String>,
}

/// A Visual Studio Code theme
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VscodeTheme {
    pub name: Option<String>,
    pub author: Option<String>,
    pub maintainers: Option<Vec<String>>,
    pub type_: Option<String>,
    pub colors: HashMap<String, Option<String>>,
    pub token_colors: Vec<TokenColor>,
}

impl FromStr for VscodeTheme {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // NOTE: Need to remove comments and remove leading commas since JSON doesn't support them.
        // TODO: Possibly do this manually to get rid of jsonc_parser.
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

fn get_color(s: &str) -> Result<Color, ParseError> {
    debug!("get_color: {}", s);
    if let Some(color) = named_color::from_name(s) {
        Ok(color)
    } else {
        Ok(Color::from_str(s)?)
    }
}

impl TryFrom<VscodeTheme> for Theme {
    type Error = ParseError;
    fn try_from(value: VscodeTheme) -> Result<Self, Self::Error> {
        let mut settings = ThemeSettings::default();

        for (key, value) in &value.colors {
            if value.is_none() {
                continue;
            }

            let value = value.as_ref().unwrap();
            match &key[..] {
                "editor.background" => settings.background = get_color(value).ok(),
                "editor.foreground" => {
                    settings.foreground = get_color(value).ok();
                    settings.caret = settings.foreground;
                }
                "foreground" => settings.foreground = get_color(value).ok(),
                "editorCursor.background" => settings.caret = get_color(value).ok(),
                "editor.lineHighlightBackground" => settings.line_highlight = get_color(value).ok(),
                "editorEditor.foreground" => settings.misspelling = get_color(value).ok(),
                "list.highlightForeground" => {
                    settings.find_highlight_foreground = get_color(value).ok();
                    settings.accent = get_color(value).ok()
                }
                "editorGutter.background" => settings.gutter = get_color(value).ok(),
                "editorLineNumber.foreground" => settings.gutter_foreground = get_color(value).ok(),
                "editor.selectionBackground" => settings.selection = get_color(value).ok(),
                "list.inactiveSelectionBackground" => {
                    settings.inactive_selection = get_color(value).ok()
                }
                "list.inactiveSelectionForeground" => {
                    settings.inactive_selection_foreground = get_color(value).ok()
                }
                "editor.findMatchBackground" | "peekViewEditor.matchHighlightBorder" => {
                    settings.highlight = get_color(value).ok();
                    settings.find_highlight = get_color(value).ok();
                }
                "editorIndentGuide.background" => settings.guide = get_color(value).ok(),
                "breadcrumb.activeSelectionForeground" => {
                    settings.active_guide = get_color(value).ok()
                }
                "breadcrumb.foreground" => settings.stack_guide = get_color(value).ok(),
                "selection.background" => {
                    settings.tags_foreground = get_color(value).ok();
                    settings.brackets_foreground = get_color(value).ok();
                }
                "widget.shadow" | "scrollbar.shadow" => settings.shadow = get_color(value).ok(),
                _ => (),
            }
        }

        Ok(Self {
            name: value.name,
            author: value.author,
            scopes: value
                .token_colors
                .iter()
                .map(|color| {
                    Ok(ThemeItem {
                        scope: if let Some(scope) = &color.scope {
                            match scope {
                                Scope::Single(s) => ScopeSelectors::from_str(s)?,
                                Scope::Multiple(s) => ScopeSelectors::from_str(&s.join(","))?,
                            }
                        } else {
                            ScopeSelectors::from_str("*")?
                        },
                        style: StyleModifier {
                            foreground: color
                                .settings
                                .foreground
                                .clone()
                                .and_then(|s| get_color(&s).ok()),
                            background: color
                                .settings
                                .background
                                .clone()
                                .and_then(|s| get_color(&s).ok()),
                            font_style: color
                                .settings
                                .font_style
                                .clone()
                                .map(|s| FontStyle::from_str(&s))
                                .transpose()?,
                        },
                    })
                })
                .collect::<Result<Vec<_>, ParseError>>()?,
            settings,
        })
    }
}

/// Parse a Visual Studio Code theme from a string.
///
/// Equivalent to calling [VscodeTheme::from_str]
pub fn parse_vscode_theme(scheme: &str) -> Result<VscodeTheme, ParseError> {
    VscodeTheme::from_str(scheme)
}

/// Parse a Visual Studio Code theme from a file.
///
/// Equivalent to calling [parse_vscode_theme]
pub fn parse_vscode_theme_file(path: &Path) -> Result<VscodeTheme, ParseError> {
    let scheme = std::fs::read_to_string(path)?;
    parse_vscode_theme(&scheme)
}

#[cfg(test)]
mod tests {
    use log::debug;

    use super::*;

    fn start_log() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn convert_theme() {
        start_log();
        let schemes = vec![
            (
                "Synthwave",
                include_str!("../assets/synthwave-color-theme.json"),
            ),
            (
                "Tokyo Night",
                include_str!("../assets/tokyo-night-color-theme.json"),
            ),
            ("Pale Night", include_str!("../assets/palenight.json")),
            ("One Dark", include_str!("../assets/OneDark.json")),
        ];

        for (name, scheme) in schemes {
            let now = std::time::Instant::now();
            let scheme = VscodeTheme::from_str(scheme).expect("Failed to parse theme");

            debug!("Parsed {} in {} ms", name, now.elapsed().as_millis());
            Theme::try_from(scheme).expect("Failed to convert to theme");

            debug!(
                "Converted {} to SytectTheme in {} ms",
                name,
                now.elapsed().as_millis()
            );
        }
    }
}
