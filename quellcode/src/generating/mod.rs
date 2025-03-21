use std::collections::HashMap;
use syntect::{highlighting::Theme, parsing::{SyntaxReference, SyntaxSet}};
use thiserror::Error;

pub mod svg;

type Properties = Vec<(&'static str, PropertyType)>;

#[derive(Debug, Error)]
pub enum GeneratorError {
    #[error("Property error: {0}")]
    PropertyError(#[from] PropertyError),
    #[error("Highlight error: {0}")]
    HighlightError(#[from] syntect::Error),
    #[error("Other error: {0}")]
    Other(String),
}

#[derive(Debug, Error)]
pub enum PropertyError {
    #[error("Unknown property")]
    UnknownProperty,
    #[error("Cannot assign value to property")]
    InvalidValueType,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum RenderType {
    Text,
    Image,
    Both,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum RenderOutput {
    Text(String),
    Image(Vec<u8>),
    Both(String, Option<Vec<u8>>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PropertyType {
    String,
    Int,
    Float,
    Bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    String(String),
    Int(i32),
    Float(f32),
    Bool(bool),
}

impl PropertyValue {
    pub fn type_of(&self) -> PropertyType {
        match self {
            PropertyValue::String(_) => PropertyType::String,
            PropertyValue::Int(_) => PropertyType::Int,
            PropertyValue::Float(_) => PropertyType::Float,
            PropertyValue::Bool(_) => PropertyType::Bool,
        }
    }
}

impl From<&str> for PropertyValue {
    fn from(value: &str) -> Self {
        PropertyValue::String(value.to_string())
    }
}

impl From<i32> for PropertyValue {
    fn from(value: i32) -> Self {
        PropertyValue::Int(value)
    }
}

impl From<f32> for PropertyValue {
    fn from(value: f32) -> Self {
        PropertyValue::Float(value)
    }
}

impl From<bool> for PropertyValue {
    fn from(value: bool) -> Self {
        PropertyValue::Bool(value)
    }
}

pub trait Generator {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn saveable(&self) -> &bool;
    fn properties(&self) -> &Properties;
    fn get_property(&self, name: &str) -> Result<PropertyValue, GeneratorError>;
    fn set_property<T: Into<PropertyValue>>(
        &mut self,
        name: &str,
        value: T,
    ) -> Result<(), GeneratorError>;
    fn kind(&self) -> &RenderType;

    fn generate(
        &self,
        text: &str,
        theme: &Theme,
        syntax: &SyntaxReference,
        syntax_set: &SyntaxSet,
    ) -> Result<RenderOutput, GeneratorError>;
}
