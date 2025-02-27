use syntect::{highlighting::ParseThemeError, parsing::ParseScopeError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Failed to parse JSON")]
    Json(#[from] serde_json::Error),
    #[error("Failed to convert color")]
    ConvertColor(String),
    #[error("Failed to parse hex color")]
    InvalidHexColor,
    #[error("Failed to convert keyword to color")]
    InvalidKeyword,
    #[error("Failed to parse variable function")]
    InvalidVariable,
    #[error("Not a valid color function")]
    InvalidColorFunction,
    #[error("Failed to parse adjuster")]
    ParseAdjuster,
    #[error("Failed to parse function")]
    ParseFunction,
    #[error("Failed to parse color space")]
    ParseColorSpace,
    #[error("Failed to parse scope")]
    ParseScope(#[from] ParseScopeError),
    #[error("Failed to parse expression")]
    ParseExpression,
    #[error("Failed to parse theme")]
    ParseTheme(#[from] ParseThemeError),
    #[error("Failed to parse number")]
    ParseNumber(#[from] std::num::ParseFloatError),
    #[error("Failed to parse JSONC")]
    ParseNumberString(#[from] jsonc_parser::errors::ParseError),
}
