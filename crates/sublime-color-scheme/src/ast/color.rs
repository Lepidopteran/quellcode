use log::{debug, info};
use crate::{error::ParseError, named::Keyword};
use std::str::FromStr;

/// A color in a color scheme
#[derive(Debug, PartialEq)]
pub enum Color {
    Hex(String),
    HexAlpha(String),
    RGB(u8, u8, u8),
    RGBA(u8, u8, u8, u8),
    HSL(u8, u8, u8),
    HSLA(u8, u8, u8, u8),
    HWB(u8, u8, u8),
    Named(Keyword),
    Variable(Variable),
    Expression {
        color: Box<Color>,
        adjusters: Vec<Adjuster>,
    },
}

// TODO: Add min contrast
#[derive(Debug, PartialEq)]
pub enum Adjuster {
    /// Blend two colors `(Color, Percentage, ColorSpace)`
    Blend(Color, f32, Option<ColorSpace>),
    /// Blend two colors with alpha `(Color, Percentage, ColorSpace)`
    BlendAlpha(Color, f32, Option<ColorSpace>),
    /// Adjust the opacity of the color `(Percentage)`
    Alpha(f32),
    /// Adjust the hue of the color `(Percentage)`
    Saturation(f32),
    /// Adjust the saturation of the color `(Percentage)`
    Lightness(f32),
}

// TODO: Add min contrast
#[derive(Debug, PartialEq)]
pub enum AdjusterKind {
    Blend,
    BlendAlpha,
    Alpha,
    Saturation,
    Lightness,
} 

#[derive(Debug, PartialEq)]
pub enum ColorSpace {
    RGB,
    HSL,
    HWB,
}

#[derive(Debug, PartialEq)]
pub enum ColorFunction {
    RGB,
    RGBA,
    HSL,
    HSLA,
    HWB,
    Variable,
    Expression,
}

impl FromStr for ColorFunction {
    type Err = ParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        info!("Parsing color function: {}", value);
        let func = match value.to_lowercase().as_str() {
            "rgb" => Self::RGB,
            "rgba" => Self::RGBA,
            "hsl" => Self::HSL,
            "hsla" => Self::HSLA,
            "hwb" => Self::HWB,
            "var" => Self::Variable,
            "color" => Self::Expression,
            _ => return Err(ParseError::ParseFunction),
        };

        Ok(func)
    }
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Hash,
    OpenParen,
    AddOperator,
    SubtractOperator,
    Point,
    Percent,
    WhiteSpace,
    CloseParen,
    Comma,
    Number(String),
    Literal(String),
}


pub fn parse_color(s: &str) -> Result<Color, ParseError> {
    let mut chars = s.chars();
    let mut stream = get_tokens(&mut chars).peekable();

    while stream.peek() == Some(&Token::WhiteSpace) {
        stream.next();
    }

    if stream.peek() == Some(&Token::Hash) {
        return parse_hex(&mut stream);
    }

    let color_function: ColorFunction = {
        let mut name = String::new();

        for token in stream.by_ref() {
            match token {
                Token::Literal(literal) => name.push_str(&literal),
                Token::WhiteSpace => continue,
                Token::OpenParen => break,
                _ => break,
            }
        }

        ColorFunction::from_str(&name)?
    };

    match color_function {
        ColorFunction::Variable => parse_variable(&mut stream),
        _ => todo!(),
    }
}

pub fn parse_adjuster(s: &str) -> Result<Adjuster, ParseError> {
    let mut chars = s.chars();
    let mut stream = get_tokens(&mut chars).peekable();

    while stream.peek() == Some(&Token::WhiteSpace) {
        stream.next();
    }

    let kind: AdjusterKind = {
        let mut name = String::new();

        for token in stream.by_ref() {
            match token {
                Token::Literal(literal) => name.push_str(&literal),
                Token::WhiteSpace => continue,
                Token::OpenParen => break,
                _ => break,
            }
        }

        match name.to_lowercase().as_str() {
            "blend" => AdjusterKind::Blend,
            "blendalpha" => AdjusterKind::BlendAlpha,
            "alpha" | "a" => AdjusterKind::Alpha,
            "saturation" | "s" => AdjusterKind::Saturation,
            "lightness" | "l" => AdjusterKind::Lightness,
            _ => return Err(ParseError::ParseAdjuster),
        }
    };

    match kind {
        AdjusterKind::Alpha => {
            for token in stream.by_ref() {
                match token {
                    Token::Number(number) => return Ok(Adjuster::Alpha(number.parse()?)),
                    _ => return Err(ParseError::ParseAdjuster),
                }
            }
        }
        _ => todo!(),
    }
}

fn get_tokens(chars: &mut impl Iterator<Item = char>) -> impl Iterator<Item = Token> {
    let mut tokens = Vec::new();
    let current_time = std::time::Instant::now();

    debug!("Parsing tokens...");
    for char in chars.by_ref() {
        match char {
            '#' => tokens.push(Token::Hash),
            '.' => tokens.push(Token::Point),
            '(' => tokens.push(Token::OpenParen),
            ')' => tokens.push(Token::CloseParen),
            '+' => tokens.push(Token::AddOperator),
            '-' => tokens.push(Token::SubtractOperator),
            '%' => tokens.push(Token::Percent),
            ',' => tokens.push(Token::Comma),
            _ => (),
        }

        if char.is_whitespace() {
            tokens.push(Token::WhiteSpace);
        }

        if char.is_alphabetic() || char == '_' {
            tokens.push(Token::Literal(char.to_string()));
        }

        if char.is_numeric() {
            tokens.push(Token::Number(char.to_string()));
        }
    }

    info!(
        "Took {}μs to parse tokens...",
        current_time.elapsed().as_micros()
    );

    tokens.into_iter()
}

fn parse_number(stream: &mut impl Iterator<Item = Token>) -> Result<f32, ParseError> {
    let mut chars = String::new();
    for token in stream.by_ref() {
        match token {
            Token::Number(number) => chars.push_str(&number),
            Token::SubtractOperator => chars.push('-'),
            Token::AddOperator => chars.push('+'),
            Token::Point => chars.push('.'),
            Token::Percent => chars.push('%'),
            Token::WhiteSpace => continue,
            _ => break,
        }
    }

    Ok(if let Some(stripped) = chars.strip_suffix('%') {
        stripped.parse::<f32>().map(|n| n / 100.0)?
    } else {
        chars.parse::<f32>()?
    })
}

fn parse_variable(stream: &mut impl Iterator<Item = Token>) -> Result<Color, ParseError> {
    let mut name = String::new();
    let mut open_paren_count = 0;

    for token in stream.by_ref() {
        if open_paren_count > 1 {
            return Err(ParseError::InvalidVariable);
        }

        match token {
            Token::OpenParen => open_paren_count += 1,
            Token::Literal(literal) => name.push_str(&literal),
            Token::CloseParen => break,
            _ => return Err(ParseError::InvalidVariable),
        }
    }

    Ok(Color::Variable(Variable(name)))
}

fn parse_hex(stream: &mut impl Iterator<Item = Token>) -> Result<Color, ParseError> {
    let mut skipped_hash = false;
    let mut code = String::new();

    debug!("Parsing hex color");
    for token in stream.by_ref() {
        if skipped_hash && token == Token::Hash {
            return Err(ParseError::InvalidHexColor);
        }

        if token == Token::Hash {
            skipped_hash = true;
            continue;
        }

        match token {
            Token::Literal(literal) => code.push_str(&literal),
            Token::Number(number) => code.push_str(&number.to_string()),
            _ => return Err(ParseError::InvalidHexColor),
        }
    }

    info!("Successfully parsed hex color: {}", code);
    match code.len() {
        3 => Ok(Color::Hex(code)),
        4 => Ok(Color::HexAlpha(code)),
        6 => Ok(Color::Hex(code)),
        8 => Ok(Color::HexAlpha(code)),
        _ => Err(ParseError::InvalidHexColor),
    }
}

#[derive(Debug, PartialEq)]
pub struct Variable(String);

impl Variable {
    pub fn name(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn start_log() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn parse_variable() {
        start_log();

        let variables = [
            (
                "var(lightPurple)",
                Color::Variable(Variable("lightPurple".to_string())),
            ),
            (
                "var(light_Purple)",
                Color::Variable(Variable("light_Purple".to_string())),
            ),
            (
                "var(light_purple)",
                Color::Variable(Variable("light_purple".to_string())),
            ),
        ];

        for (string, expected) in variables {
            assert_eq!(parse_color(string).unwrap(), expected);
        }
    }

    #[test]
    fn parse_hex() {
        start_log();

        let colors = [
            ("#fff", Color::Hex("fff".to_string())),
            ("#9900ee", Color::Hex("9900ee".to_string())),
            ("#eee7", Color::HexAlpha("eee7".to_string())),
            ("#8800ffff", Color::HexAlpha("8800ffff".to_string())),
        ];

        for (string, expected) in colors {
            assert_eq!(parse_color(string).unwrap(), expected);
        }
    }

    #[test]
    fn invalid_hex() {
        assert!(parse_color("#8800-0").is_err());
        assert!(parse_color("#88000").is_err());
    }

    #[test]
    fn parse_numbers() {
        let numbers = [
            ("1", 1.0),
            ("-1", -1.0),
            ("1.0", 1.0),
            ("-1.0", -1.0),
            ("1%", 0.01),
            ("-1%", -0.01),
            ("+1", 1.0),
            ("1.0%", 0.01),
            ("100%", 1.0),
            ("100.0%", 1.0),
            ("100.0", 100.0),
        ];

        for (string, expected) in numbers {
            let mut stream = get_tokens(&mut string.chars());
            assert_eq!(parse_number(&mut stream).unwrap(), expected);
        }
    }
}
