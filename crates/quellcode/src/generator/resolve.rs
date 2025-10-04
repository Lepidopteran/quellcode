use std::fmt::Display;

use syntect::easy::HighlightLines;

use super::*;

pub const RED_CHANNEL: u16 = 2401;
pub const GREEN_CHANNEL: u16 = 2402;
pub const BLUE_CHANNEL: u16 = 2403;
pub const ALPHA_CHANNEL: u16 = 2600;

const DEFAULT_WIDTH: i32 = 1920;
const DEFAULT_HEIGHT: i32 = 1080;

#[derive(Debug, Clone, Default)]
pub struct FusionGenerator {}

impl FusionGenerator {
    pub fn new() -> FusionGenerator {
        FusionGenerator::default()
    }
}

#[derive(Debug, Clone)]
struct Input {
    key: String,
    value: InputValue,
}

impl Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} = Input {{ Value = {}, }}", self.key, self.value)
    }
}

impl Input {
    fn new(key: String, value: InputValue) -> Input {
        Input { key, value }
    }

    fn string(key: &str, value: &str) -> Input {
        Input::new(key.to_string(), InputValue::String(value.to_string()))
    }

    fn integer(key: &str, value: i32) -> Input {
        Input::new(key.to_string(), InputValue::Integer(value))
    }

    fn float(key: &str, value: f32) -> Input {
        Input::new(key.to_string(), InputValue::Float(value))
    }
}

#[derive(Debug, Clone)]
enum InputValue {
    String(String),
    Integer(i32),
    Float(f32),
}

impl Display for InputValue {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputValue::String(string) => write!(formatter, "\"{}\"", string),
            InputValue::Integer(int) => write!(formatter, "{}", int),
            InputValue::Float(float) => write!(formatter, "{}", float),
        }
    }
}

// NOTE: possibly use handlebars-rs to do this instead.
impl Generator for FusionGenerator {
    fn generate_code(
        &self,
        text: &str,
        theme: &Theme,
        syntax: &SyntaxReference,
        syntax_set: &SyntaxSet,
        options: &GeneratorOptions,
    ) -> Result<String> {
        let mut result = String::new();

        let text = if options
            .extra
            .get("preserve_tabs")
            .and_then(|value| value.clone().try_into().ok())
            .unwrap_or(false)
        {
            text.to_string()
        } else {
            text.replace(
                "\t",
                &" ".repeat(
                    options
                        .extra
                        .get("tab_size")
                        .and_then(|value| value.clone().try_into().ok())
                        .unwrap_or(4) as usize,
                ),
            )
        };

        result.push_str("{\n");
        result.push_str("\tTools = ordered() {\n");
        result.push_str("\t\tCodeText = TextPlus {\n");
        result.push_str("\t\t\tInputs = {\n");

        let width = options
            .extra
            .get("width")
            .and_then(|value| value.clone().try_into().ok())
            .unwrap_or(DEFAULT_WIDTH);

        let height = options
            .extra
            .get("height")
            .and_then(|value| value.clone().try_into().ok())
            .unwrap_or(DEFAULT_HEIGHT);

        let inputs = [
            Input::integer("GlobalOut", 119),
            Input::integer(
                "Width",
                width,
            ),
            Input::integer(
                "Height",
                height
            ),
            Input::string("Font", &options.font_family),
            Input::float("FontSize", options.font_size / width as f32),
            Input::integer("VerticalTopCenterBottom", -1),
            Input::integer("HorizontalLeftCenterRight", -1),
            Input::integer("VerticalJustificationNew", 3),
            Input::integer("HorizontalJustificationNew", 3),
        ];

        for input in inputs {
            result.push_str(&format!("\t\t\t\t{input},\n"));
        }

        result.push_str("\t\t\t\tStyledText = Input {\n");
        result.push_str("\t\t\t\t\tSourceOp = \"CodeTextStyling\",\n");
        result.push_str("\t\t\t\t\tSource = \"StyledText\",\n");
        result.push_str("\t\t\t\t},\n");

        result.push_str("\t\t\t},\n");
        result.push_str("\t\t},\n");

        result.push_str("\t\tCodeTextStyling = StyledTextCLS {\n");
        result.push_str("\t\t\tInputs = {\n");

        result.push_str(&format!(
            "\t\t\t\t{},\n",
            Input::string("Text", &text.escape_default().to_string())
        ));

        result.push_str("\t\t\t\tCharacterLevelStyling = Input {\n");
        result.push_str("\t\t\t\t\tValue = StyledText {\n");
        result.push_str("\t\t\t\t\t\tArray = {\n");

        let mut document_offset = 0;
        let mut highlight = HighlightLines::new(syntax, theme);

        for line in text.lines() {
            let ranges = highlight.highlight_line(line, syntax_set)?;

            let mut line_offset = 0;
            for &(ref style, text) in ranges.iter() {
                let start = document_offset + line_offset;
                let end = start + text.len();

                for (id, color) in [
                    (RED_CHANNEL, style.foreground.r),
                    (GREEN_CHANNEL, style.foreground.g),
                    (BLUE_CHANNEL, style.foreground.b),
                    (ALPHA_CHANNEL, style.foreground.a),
                ] {
                    result.push_str(&format!(
                        "\t\t\t\t\t\t\t{{{}, {}, {}, Value = {}, }},\n",
                        id,
                        start,
                        end,
                        color as f32 / 255.0
                    ));
                }

                line_offset += text.len();
            }

            document_offset += line.len() + 1;
        }

        result.push_str("\t\t\t\t\t\t},\n");
        result.push_str("\t\t\t\t\t},\n");
        result.push_str("\t\t\t\t},\n");
        result.push_str("\t\t\t},\n");
        result.push_str("\t\t},\n");
        result.push_str("\t},\n");
        result.push_str("}\n");

        Ok(result)
    }
}

impl GeneratorExt for FusionGenerator {
    fn information() -> GeneratorInfo {
        GeneratorInfo {
            name: "Fusion",
            description: "Generates code that can be used in Davinci Resolve's Fusion editor.",
            extensions: None,
            properties: Some(vec![
                PropertyInfo::Integer {
                    name: "width".to_string(),
                    description: "The width of the generated text node".to_string(),
                    default: Some(DEFAULT_HEIGHT),
                    min: Some(1),
                    max: None,
                    step: Some(1),
                    depends_on: None,
                    display_name: None,
                    disables: None,
                },
                PropertyInfo::Integer {
                    name: "height".to_string(),
                    description: "The height of the generated text node".to_string(),
                    default: Some(DEFAULT_WIDTH),
                    min: Some(1),
                    max: None,
                    step: Some(1),
                    depends_on: None,
                    display_name: None,
                    disables: None,
                },
                PropertyInfo::Boolean {
                    name: "preserve_tabs".to_string(),
                    description: "Use tabs instead of spaces".to_string(),
                    default: Some(false),
                    depends_on: None,
                    display_name: None,
                    disables: Some("spaces_per_tab".to_string()),
                },
                PropertyInfo::Integer {
                    name: "spaces_per_tab".to_string(),
                    description: "The number of spaces per tab".to_string(),
                    default: Some(4),
                    min: Some(1),
                    max: None,
                    step: Some(1),
                    depends_on: None,
                    display_name: None,
                    disables: None,
                },
            ]),
            syntax: Some("Lua"),
            saveable: false,
        }
    }
}
