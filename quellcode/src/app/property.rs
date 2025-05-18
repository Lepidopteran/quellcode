use std::fmt::Display;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PropertyError {
    #[error("Unknown property")]
    UnknownProperty,
    #[error("Cannot assign value to property")]
    InvalidValueType,
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
    Float(f64),
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

impl From<f64> for PropertyValue {
    fn from(value: f64) -> Self {
        PropertyValue::Float(value)
    }
}

impl From<f32> for PropertyValue {
    fn from(value: f32) -> Self {
        PropertyValue::Float(value as f64)
    }
}

impl From<bool> for PropertyValue {
    fn from(value: bool) -> Self {
        PropertyValue::Bool(value)
    }
}

impl Display for PropertyValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PropertyValue::String(string) => write!(f, "{}", string),
            PropertyValue::Int(int) => write!(f, "{}", int),
            PropertyValue::Float(float) => write!(f, "{}", float),
            PropertyValue::Bool(bool) => write!(f, "{}", bool),
        }
    }
}

impl TryInto<f64> for PropertyValue {
    type Error = PropertyError;
    fn try_into(self) -> Result<f64, Self::Error> {
        match self {
            PropertyValue::Float(float) => Ok(float),
            PropertyValue::Int(int) => Ok(int as f64),
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
