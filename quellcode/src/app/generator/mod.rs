use std::fmt::Debug;
use thiserror::Error;

use syntect::{
    highlighting::Theme,
    parsing::{SyntaxReference, SyntaxSet},
};

use super::property::*;

pub mod svg;

type Properties = Vec<Property>;
type Extensions = Vec<&'static str>;

#[derive(Debug, Error)]
pub enum GeneratorError {
    #[error("Property error: {0}")]
    PropertyError(#[from] PropertyError),
    #[error("Highlight error: {0}")]
    HighlightError(#[from] syntect::Error),
    #[error("Other error: {0}")]
    Other(String),
}

pub trait Generator: Send + Sync + Debug {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn properties(&self) -> &Properties;
    fn font_family(&self) -> &str;
    fn set_font_family(&mut self, family: &str);
    fn font_size(&self) -> f32;
    fn set_font_size(&mut self, size: f32);
    fn get_property(&self, name: &str) -> Result<PropertyValue, GeneratorError>;
    fn set_property(&mut self, name: &str, value: PropertyValue) -> Result<(), GeneratorError>;
    fn saveable(&self) -> bool;
    fn extensions(&self) -> Option<&Extensions> {
        None
    }

    fn generate_code(
        &self,
        _text: &str,
        _theme: &Theme,
        _syntax: &SyntaxReference,
        _syntax_set: &SyntaxSet,
    ) -> Result<String, GeneratorError>;
}
