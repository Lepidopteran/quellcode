use syntect::{
    highlighting::Theme,
    parsing::{SyntaxReference, SyntaxSet},
};
use thiserror::Error;

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

#[derive(Debug, Clone)]
pub struct Property {
    pub name: &'static str,
    pub kind: PropertyType,
    pub value: PropertyValue,
    pub description: &'static str,
    pub default: Option<PropertyValue>,
    pub min: Option<PropertyValue>,
    pub max: Option<PropertyValue>,
}

impl Default for Property {
    fn default() -> Self {
        Property {
            name: "",
            kind: PropertyType::String,
            value: PropertyValue::String("".to_string()),
            description: "",
            default: None,
            min: None,
            max: None,
        }
    }
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

impl From<String> for PropertyValue {
    fn from(value: String) -> Self {
        PropertyValue::String(value)
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

impl std::string::ToString for PropertyValue {
    fn to_string(&self) -> String {
        match self {
            PropertyValue::String(string) => string.clone(),
            PropertyValue::Int(int) => int.to_string(),
            PropertyValue::Float(float) => float.to_string(),
            PropertyValue::Bool(bool) => bool.to_string(),
        }
    }
}

impl TryInto<f32> for PropertyValue {
    type Error = PropertyError;
    fn try_into(self) -> Result<f32, Self::Error> {
        match self {
            PropertyValue::Float(float) => Ok(float),
            PropertyValue::Int(int) => Ok(int as f32),
            _ => Err(PropertyError::InvalidValueType),
        }
    }
}

impl TryInto<i32> for PropertyValue {
    type Error = PropertyError;
    fn try_into(self) -> Result<i32, Self::Error> {
        match self {
            PropertyValue::Int(int) => Ok(int),
            PropertyValue::Float(float) => Ok(float as i32),
            _ => Err(PropertyError::InvalidValueType),
        }
    }
}

impl TryInto<bool> for PropertyValue {
    type Error = PropertyError;
    fn try_into(self) -> Result<bool, Self::Error> {
        match self {
            PropertyValue::Bool(bool) => Ok(bool),
            PropertyValue::Int(int) => Ok(int != 0),
            _ => Err(PropertyError::InvalidValueType),
        }
    }
}

pub trait Generator: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn saveable(&self) -> &bool;
    fn extensions(&self) -> Option<&Extensions> {
        None
    }
    fn properties(&self) -> &Properties;
    fn font_family(&self) -> &str;
    fn set_font_family(&mut self, family: &str);
    fn font_size(&self) -> f32;
    fn set_font_size(&mut self, size: f32);
    fn get_property(&self, name: &str) -> Result<PropertyValue, GeneratorError>;
    fn set_property(&mut self, name: &str, value: PropertyValue) -> Result<(), GeneratorError>;
    fn kind(&self) -> &RenderType;

    fn generate(
        &self,
        text: &str,
        theme: &Theme,
        syntax: &SyntaxReference,
        syntax_set: &SyntaxSet,
    ) -> Result<RenderOutput, GeneratorError>;
}
