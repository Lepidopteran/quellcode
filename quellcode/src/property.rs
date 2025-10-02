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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(untagged)]
pub enum PropertyValue {
    String(String),
    Int(i32),
    Float(f64),
    Bool(bool),
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum StringPropertySubtype {
    Path,
    Template,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase", tag = "kind")]
#[ts(export)]
pub enum PropertyInfo {
    String {
        name: String,
        description: String,
        default: Option<String>,
        sub_type: Option<StringPropertySubtype>,
        display_name: Option<String>,
        depends_on: Option<String>,
        disables: Option<String>,
    },

    Integer {
        name: String,
        description: String,
        default: Option<i32>,
        min: Option<i32>,
        max: Option<i32>,
        step: Option<i32>,
        depends_on: Option<String>,
        display_name: Option<String>,
        disables: Option<String>,
    },

    Float {
        name: String,
        description: String,
        default: Option<f64>,
        min: Option<f64>,
        max: Option<f64>,
        step: Option<f64>,
        depends_on: Option<String>,
        display_name: Option<String>,
        disables: Option<String>,
    },

    Boolean {
        name: String,
        description: String,
        default: Option<bool>,
        depends_on: Option<String>,
        display_name: Option<String>,
        disables: Option<String>,
    },
}

impl PropertyInfo {
    pub fn name(&self) -> &str {
        match self {
            PropertyInfo::String { name, .. } => name,
            PropertyInfo::Integer { name, .. } => name,
            PropertyInfo::Float { name, .. } => name,
            PropertyInfo::Boolean { name, .. } => name,
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
