use serde::{Deserialize, Serialize};
use std::fmt::Display;
use thiserror::Error;
use ts_rs::TS;

#[derive(Debug, Error)]
pub enum PropertyError {
    #[error("Unknown property")]
    UnknownProperty,
    #[error("Cannot assign value to property")]
    InvalidValueType,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum PropertyType {
    String,
    Int,
    Float,
    Bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(untagged)]
pub enum PropertyValue {
    String(String),
    Int(i32),
    Float(f64),
    Bool(bool),
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct Property {
    pub name: &'static str,
    pub kind: PropertyType,
    pub value: PropertyValue,
    pub description: &'static str,
    pub default: Option<PropertyValue>,
    pub min: Option<PropertyValue>,
    pub max: Option<PropertyValue>,
}

impl Property {
    /// Converts the property's name from snake_case to a prettified title case string.
    ///
    /// # Returns
    ///
    /// A `String` that represents the property's name converted to title case, with underscores replaced by spaces and each word capitalized.
    ///
    /// # Examples
    ///
    /// ```
    /// use quellcode_lib::property::{Property, PropertyType, PropertyValue};
    ///
    /// let property = Property {
    ///     name: "example_property_name",
    ///     kind: PropertyType::String,
    ///     value: PropertyValue::String("".to_string()),
    ///     description: "",
    ///     default: None,
    ///     min: None,
    ///     max: None,
    /// };
    ///
    /// assert_eq!(property.pretty_name(), "Example Property Name");
    /// ```
    pub fn pretty_name(&self) -> String {
        self.name
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
