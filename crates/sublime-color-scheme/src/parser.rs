use crate::error::ParseError;
use log::{debug, error, info};
use palette::{encoding::Srgb, named, rgb::Rgb};
use std::str::FromStr;

/// A color in a color scheme
#[derive(Debug, PartialEq)]
pub enum Color {
    Hex(String),
    HexAlpha(String),
    RGB(u8, u8, u8),
    RGBA(u8, u8, u8, f32),
    HSL(u32, f32, f32),
    HSLA(u32, f32, f32, f32),
    HWB(u32, f32, f32, Option<f32>),
    Named(Rgb<Srgb, u8>),
    Variable(String),
    Expression(Box<Color>, Vec<Adjuster>),
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
pub enum Adjuster {
    /// Blend two colors `(Color, Percentage, ColorSpace)`
    Blend(Color, f32, Option<ColorSpace>),

    /// Blend two colors with alpha `(Color, Percentage, ColorSpace)`
    BlendAlpha(Color, f32, Option<ColorSpace>),

    /// Adjust the opacity of the color `(Percentage)`
    Alpha(f32),

    /// Adjust the hue of the color `(Percentage, Relative)`.
    Saturation(f32, bool),

    /// Adjust the saturation of the color `(Percentage, Relative)`.
    Lightness(f32, bool),

    /// Modifies a color to ensure a minimum contrast ratio against a “background” color.
    /// [Source](https://www.sublimetext.com/docs/minihtml.html#min-contrast-adjuster)
    /// `(Color, Ratio)`
    MinContrast(Color, f32),
}

#[derive(Debug, PartialEq)]
pub enum AdjusterKind {
    Blend,
    BlendAlpha,
    Alpha,
    Saturation,
    Lightness,
    MinContrast,
}

#[derive(Debug, PartialEq)]
pub enum ColorSpace {
    RGB,
    HSL,
    HWB,
}

impl FromStr for ColorSpace {
    type Err = ParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_lowercase().as_str() {
            "rgb" => Ok(ColorSpace::RGB),
            "hsl" => Ok(ColorSpace::HSL),
            "hwb" => Ok(ColorSpace::HWB),
            _ => Err(ParseError::ParseColorSpace),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Number {
    Integer(i32),
    Float(f32),
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

    let mut name = String::new();

    let mut has_open_paren = false;
    for token in stream.by_ref() {
        match token {
            Token::Literal(literal) => name.push_str(&literal),

            Token::WhiteSpace => continue,
            Token::OpenParen => {
                has_open_paren = true;
                break;
            }
            _ => break,
        }
    }

    if !has_open_paren {
        return Ok(Color::Named(
            named::from_str(&name).ok_or(ParseError::InvalidKeyword)?,
        ));
    }

    let color_function = ColorFunction::from_str(&name)?;

    match color_function {
        ColorFunction::Variable => parse_variable(&mut stream),
        ColorFunction::Expression => {
            let arguments = split_expression(&mut stream)?;

            if arguments.len() < 2 {
                return Err(ParseError::ParseExpression);
            }

            let color = parse_color(&arguments[0])?;
            let adjusters: Vec<Adjuster> = arguments
                .iter()
                .skip(1)
                .map(|argument| parse_adjuster(argument))
                .collect::<Result<_, _>>()?;

            Ok(Color::Expression(Box::new(color), adjusters))
        }
        ColorFunction::HSL
        | ColorFunction::HSLA
        | ColorFunction::HWB
        | ColorFunction::RGB
        | ColorFunction::RGBA => parse_color_function(&mut stream, color_function),
    }
}

pub fn parse_adjuster(s: &str) -> Result<Adjuster, ParseError> {
    let mut chars = s.chars();
    let mut stream = get_tokens(&mut chars).peekable();

    info!("Parsing adjuster: {}", s);

    while stream.peek() == Some(&Token::WhiteSpace) {
        stream.next();
    }

    let kind: AdjusterKind = {
        let mut name = String::new();

        for token in stream.by_ref() {
            match token {
                // NOTE: min-contrast contains a dash in the name
                Token::SubtractOperator => name.push('-'),
                Token::Literal(literal) => name.push_str(&literal),
                Token::WhiteSpace => continue,
                Token::OpenParen => break,
                _ => break,
            }
        }

        match name.to_lowercase().as_str() {
            "blend" => AdjusterKind::Blend,
            "blenda" => AdjusterKind::BlendAlpha,
            "alpha" | "a" => AdjusterKind::Alpha,
            "saturation" | "s" => AdjusterKind::Saturation,
            "lightness" | "l" => AdjusterKind::Lightness,
            "min-contrast" => AdjusterKind::MinContrast,
            _ => return Err(ParseError::ParseAdjuster),
        }
    };

    match kind {
        AdjusterKind::Alpha => {
            let number = parse_number(&mut stream)?;

            if let Number::Float(float) = number {
                if !(0.0..=1.0).contains(&float) {
                    return Err(ParseError::ParseAdjuster);
                }

                Ok(Adjuster::Alpha(float))
            } else {
                Err(ParseError::ParseAdjuster)
            }
        }
        AdjusterKind::Saturation => {
            let mut number_string = String::new();
            let mut relative = false;

            for token in stream.by_ref() {
                match token {
                    Token::Number(n) => number_string.push_str(&n),
                    Token::SubtractOperator => {
                        number_string.push('-');
                        relative = true;
                    }
                    Token::AddOperator => {
                        number_string.push('+');
                        relative = true;
                    }
                    Token::Percent => number_string.push('%'),
                    Token::Point => number_string.push('.'),
                    Token::WhiteSpace => continue,
                    Token::CloseParen => break,
                    _ => return Err(ParseError::ParseAdjuster),
                }
            }

            let number = parse_number_string(&number_string)?;

            if let Number::Float(float) = number {
                if !(-1.0..=1.0).contains(&float) {
                    return Err(ParseError::ParseAdjuster);
                }

                Ok(Adjuster::Saturation(float, relative))
            } else {
                Err(ParseError::ParseAdjuster)
            }
        }
        AdjusterKind::Lightness => {
            let mut number_string = String::new();
            let mut relative = false;

            for token in stream.by_ref() {
                match token {
                    Token::Number(n) => number_string.push_str(&n),
                    Token::SubtractOperator => {
                        number_string.push('-');
                        relative = true;
                    }
                    Token::AddOperator => {
                        number_string.push('+');
                        relative = true;
                    }
                    Token::Percent => number_string.push('%'),
                    Token::Point => number_string.push('.'),
                    Token::WhiteSpace => continue,
                    Token::CloseParen => break,
                    _ => return Err(ParseError::ParseAdjuster),
                }
            }

            let number = parse_number_string(&number_string)?;

            if let Number::Float(float) = number {
                if !(-1.0..=1.0).contains(&float) {
                    return Err(ParseError::ParseAdjuster);
                }

                Ok(Adjuster::Lightness(float, relative))
            } else {
                Err(ParseError::ParseAdjuster)
            }
        }
        AdjusterKind::MinContrast => {
            let arguments = split_expression(&mut stream)?;

            if arguments.len() != 2 {
                return Err(ParseError::ParseAdjuster);
            }

            let color = parse_color(&arguments[0])?;
            let percentage = parse_number_string(&arguments[1])?;

            if let Number::Float(float) = percentage {
                Ok(Adjuster::MinContrast(color, float))
            } else {
                Err(ParseError::ParseAdjuster)
            }
        }
        AdjusterKind::Blend => {
            let arguments = split_expression(&mut stream)?;

            if arguments.len() < 2 || arguments.len() > 3 {
                return Err(ParseError::ParseAdjuster);
            }

            let color = parse_color(&arguments[0])?;
            let percentage = parse_number_string(&arguments[1])?;
            let color_space = arguments
                .get(2)
                .map(|arg| ColorSpace::from_str(arg))
                .transpose()?;

            if let Number::Float(float) = percentage {
                Ok(Adjuster::Blend(color, float, color_space))
            } else {
                Err(ParseError::ParseAdjuster)
            }
        }
        AdjusterKind::BlendAlpha => {
            let arguments = split_expression(&mut stream)?;

            if arguments.len() < 2 || arguments.len() > 3 {
                return Err(ParseError::ParseAdjuster);
            }

            let color = parse_color(&arguments[0])?;
            let percentage = parse_number_string(&arguments[1])?;
            let color_space = arguments
                .get(2)
                .map(|arg| ColorSpace::from_str(arg))
                .transpose()?;

            if let Number::Float(float) = percentage {
                Ok(Adjuster::BlendAlpha(color, float, color_space))
            } else {
                Err(ParseError::ParseAdjuster)
            }
        }
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

fn split_expression(stream: &mut impl Iterator<Item = Token>) -> Result<Vec<String>, ParseError> {
    let mut arguments = Vec::new();

    info!("Splitting expression...");

    let mut current_argument = String::new();
    let mut level = 1;
    for token in stream.by_ref() {
        match token {
            Token::Literal(literal) => current_argument.push_str(&literal),
            Token::Number(number) => current_argument.push_str(&number),
            Token::OpenParen => {
                level += 1;
                current_argument.push('(');
            }
            Token::CloseParen => {
                level -= 1;

                if level == 0 {
                    arguments.push(current_argument.clone());
                    debug!("Pushed argument: {}", current_argument);
                    current_argument.clear();
                    debug!("Cleared argument");

                    break;
                }

                current_argument.push(')');
            }
            Token::Comma => current_argument.push(','),
            Token::Point => current_argument.push('.'),
            Token::Percent => current_argument.push('%'),
            Token::AddOperator => current_argument.push('+'),
            Token::SubtractOperator => current_argument.push('-'),
            Token::WhiteSpace => {
                if level > 1 {
                    current_argument.push(' ');
                    continue;
                }

                if current_argument.is_empty() {
                    continue;
                }

                arguments.push(current_argument.clone());
                debug!("Pushed argument: {}", current_argument);
                current_argument.clear();
                debug!("Cleared argument");
            }
            _ => continue,
        }
    }

    if arguments.is_empty() {
        return Err(ParseError::ParseExpression);
    }

    info!("Successfully split expression: {:?}", arguments);

    Ok(arguments)
}

fn parse_number(stream: &mut impl Iterator<Item = Token>) -> Result<Number, ParseError> {
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

    parse_number_string(&chars)
}

fn parse_number_string(input: &str) -> Result<Number, ParseError> {
    if input.is_empty() {
        return Err(ParseError::InvalidNumber("Empty string".to_string()));
    }

    if input.chars().filter(|c| *c == '.').count() > 1 {
        return Err(ParseError::InvalidNumber(
            "Too many decimal points".to_string(),
        ));
    }

    let float = input.contains('.');

    Ok(if let Some(stripped) = input.strip_suffix('%') {
        Number::Float(stripped.parse::<f32>().map(|n| n / 100.0)?)
    } else if float {
        Number::Float(input.parse::<f32>()?)
    } else {
        Number::Integer(input.parse::<i32>()?)
    })
}

fn parse_variable(stream: &mut impl Iterator<Item = Token>) -> Result<Color, ParseError> {
    let mut name = String::new();
    let mut open_paren_count = 0;

    for token in stream.by_ref() {
        if open_paren_count > 1 {
            error!("Too many open parens: {}", name);
            return Err(ParseError::InvalidVariable);
        }

        match token {
            Token::OpenParen => open_paren_count += 1,
            Token::Literal(literal) => name.push_str(&literal),
            Token::Number(number) => name.push_str(&number),
            Token::CloseParen => break,
            _ => {
                error!("Invalid variable: {}", name);
                return Err(ParseError::InvalidVariable);
            }
        }
    }

    Ok(Color::Variable(name))
}

fn parse_color_function(
    stream: &mut impl Iterator<Item = Token>,
    color_function: ColorFunction,
) -> Result<Color, ParseError> {
    if color_function == ColorFunction::Variable || color_function == ColorFunction::Expression {
        return Err(ParseError::InvalidColorFunction);
    }

    let mut stream = stream.peekable();
    let mut numbers: Vec<Number> = Vec::new();
    let mut current_number = String::new();

    for token in stream.by_ref() {
        match token {
            Token::Number(number) => current_number.push_str(&number),
            Token::SubtractOperator => current_number.push('-'),
            Token::AddOperator => current_number.push('+'),
            Token::Point => current_number.push('.'),
            Token::Percent => current_number.push('%'),

            // NOTE: I'm assuming color-scheme files use the legacy color function format
            Token::WhiteSpace => continue,
            Token::Comma | Token::CloseParen => {
                if current_number.is_empty() {
                    return Err(ParseError::InvalidColorFunction);
                }

                numbers.push(parse_number_string(&current_number)?);
                current_number.clear();
            }
            Token::OpenParen => break,
            _ => break,
        }
    }

    debug!("Found {} numbers: {:?}", numbers.len(), numbers);

    match color_function {
        ColorFunction::RGB => {
            if numbers.len() != 3 {
                return Err(ParseError::InvalidColorFunction);
            }

            let rgb: Vec<u8> = numbers
                .iter()
                .take(3)
                .map(|num| {
                    if let Number::Integer(value) = num {
                        Ok(*value as u8)
                    } else {
                        Err(ParseError::InvalidColorFunction)
                    }
                })
                .collect::<Result<Vec<u8>, _>>()?;

            Ok(Color::RGB(rgb[0], rgb[1], rgb[2]))
        }
        ColorFunction::HSL => {
            if numbers.len() != 3 {
                return Err(ParseError::InvalidColorFunction);
            }

            let (first, rest) = numbers
                .split_first()
                .ok_or(ParseError::InvalidColorFunction)?;

            let hue = if let Number::Integer(value) = first {
                if *value < 0 || *value > 360 {
                    return Err(ParseError::InvalidColorFunction);
                }

                *value as u32
            } else {
                return Err(ParseError::InvalidColorFunction);
            };

            let numbers: Vec<f32> = rest
                .iter()
                .map(|num| {
                    if let Number::Float(value) = num {
                        Ok(*value)
                    } else {
                        Err(ParseError::InvalidColorFunction)
                    }
                })
                .collect::<Result<Vec<f32>, _>>()?;

            Ok(Color::HSL(hue, numbers[0], numbers[1]))
        }

        ColorFunction::HWB => {
            if numbers.len() != 3 || numbers.len() != 4 {
                return Err(ParseError::InvalidColorFunction);
            }

            let (hue, arguments) = numbers
                .split_first()
                .ok_or(ParseError::InvalidColorFunction)?;

            let hue = if let Number::Integer(value) = hue {
                if *value < 0 || *value > 360 {
                    return Err(ParseError::InvalidColorFunction);
                }

                *value as u32
            } else {
                return Err(ParseError::InvalidColorFunction);
            };

            let arguments: Vec<f32> = arguments
                .iter()
                .map(|num| {
                    if let Number::Float(value) = num {
                        Ok(*value)
                    } else {
                        Err(ParseError::InvalidColorFunction)
                    }
                })
                .collect::<Result<Vec<f32>, _>>()?;

            Ok(Color::HWB(
                hue,
                arguments[0],
                arguments[1],
                Some(arguments[2]),
            ))
        }

        ColorFunction::RGBA => {
            if numbers.len() != 4 {
                return Err(ParseError::InvalidColorFunction);
            }

            let (alpha, rest) = numbers
                .split_last()
                .ok_or(ParseError::InvalidColorFunction)?;

            let rgb: Vec<u8> = rest
                .iter()
                .map(|num| {
                    if let Number::Integer(value) = num {
                        Ok(*value as u8)
                    } else {
                        Err(ParseError::InvalidColorFunction)
                    }
                })
                .collect::<Result<Vec<u8>, _>>()?;

            let alpha = if let Number::Float(value) = alpha {
                if *value < 0.0 || *value > 1.0 {
                    return Err(ParseError::InvalidColorFunction);
                }
                *value
            } else {
                return Err(ParseError::InvalidColorFunction);
            };

            Ok(Color::RGBA(rgb[0], rgb[1], rgb[2], alpha))
        }

        ColorFunction::HSLA => {
            if numbers.len() != 4 {
                return Err(ParseError::InvalidColorFunction);
            }

            let (hue, arguments) = numbers
                .split_first()
                .ok_or(ParseError::InvalidColorFunction)?;

            let hue = if let Number::Integer(value) = hue {
                if *value < 0 || *value > 360 {
                    return Err(ParseError::InvalidColorFunction);
                }

                *value as u32
            } else {
                return Err(ParseError::InvalidColorFunction);
            };

            let arguments: Vec<f32> = arguments
                .iter()
                .map(|num| {
                    if let Number::Float(value) = num {
                        Ok(*value)
                    } else {
                        Err(ParseError::InvalidColorFunction)
                    }
                })
                .collect::<Result<Vec<f32>, _>>()?;

            Ok(Color::HSLA(hue, arguments[0], arguments[1], arguments[2]))
        }
        _ => Err(ParseError::InvalidColorFunction),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn start_log() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn parse_adjusters() {
        start_log();

        let adjusters = [
            ("alpha(50%)", Adjuster::Alpha(0.5)),
            ("saturation(50%)", Adjuster::Saturation(0.5, false)),
            ("lightness(50%)", Adjuster::Lightness(0.5, false)),
            (
                "blend(rgb(100, 0, 200) 50%)",
                Adjuster::Blend(Color::RGB(100, 0, 200), 0.5, None),
            ),
            (
                "blend(rgb(100, 0, 200) 50% hsl)",
                Adjuster::Blend(Color::RGB(100, 0, 200), 0.5, Some(ColorSpace::HSL)),
            ),
            (
                "min-contrast(rgb(100, 0, 200) 50%)",
                Adjuster::MinContrast(Color::RGB(100, 0, 200), 0.5),
            ),
        ];

        for (string, expected) in adjusters {
            assert_eq!(parse_adjuster(string).unwrap(), expected);
        }
    }

    #[test]
    fn parse_variable() {
        start_log();

        let variables = [
            (
                "var(lightPurple)",
                Color::Variable("lightPurple".to_string()),
            ),
            (
                "var(light_Purple)",
                Color::Variable("light_Purple".to_string()),
            ),
            (
                "var(light_purple)",
                Color::Variable("light_purple".to_string()),
            ),
        ];

        for (string, expected) in variables {
            assert_eq!(parse_color(string).unwrap(), expected);
        }
    }

    #[test]
    fn parse_color_functions() {
        start_log();

        let functions = [
            ("rgb(100, 0, 200)", Color::RGB(100, 0, 200)),
            ("rgba(100, 0, 200, 0.5)", Color::RGBA(100, 0, 200, 0.5)),
            ("hsl(100, 50%, 100%)", Color::HSL(100, 0.5, 1.0)),
        ];

        for (string, expected) in functions {
            assert_eq!(parse_color(string).unwrap(), expected);
        }
    }

    #[test]
    fn parse_color_mod() {
        start_log();

        assert_eq!(
            parse_color("color(rgb(100, 0, 200) alpha(50%))").unwrap(),
            Color::Expression(
                Box::new(Color::RGB(100, 0, 200)),
                vec![Adjuster::Alpha(0.5)]
            )
        );

        assert_eq!(
            parse_color(
                "color(rgb(100, 0, 200) alpha(50%) blend(color(rgb(100, 0, 200) alpha(50%)) 50%))"
            )
            .unwrap(),
            Color::Expression(
                Box::new(Color::RGB(100, 0, 200)),
                vec![
                    Adjuster::Alpha(0.5),
                    Adjuster::Blend(
                        Color::Expression(
                            Box::new(Color::RGB(100, 0, 200)),
                            vec![Adjuster::Alpha(0.5)]
                        ),
                        0.5,
                        None
                    )
                ]
            )
        );
    }

    #[test]
    fn parse_hex() {
        start_log();

        let colors = [
            ("#fff", Color::Hex("fff".to_string())),
            ("#9900ee", Color::Hex("9900ee".to_string())),
            ("#eee7", Color::HexAlpha("eee7".to_string())),
            ("#8800ffff", Color::HexAlpha("8800ffff".to_string())),
            ("red", Color::Named(named::RED)),
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
            ("1", Number::Integer(1)),
            ("-1", Number::Integer(-1)),
            ("1.0", Number::Float(1.0)),
            ("-1.0", Number::Float(-1.0)),
            ("1%", Number::Float(0.01)),
            ("-1%", Number::Float(-0.01)),
            ("+1", Number::Integer(1)),
            ("1.0%", Number::Float(0.01)),
            ("100%", Number::Float(1.0)),
            ("100.0%", Number::Float(1.0)),
            ("100.0", Number::Float(100.0)),
        ];

        for (string, expected) in numbers {
            assert_eq!(parse_number_string(string).unwrap(), expected);
        }
    }
}
