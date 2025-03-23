use quellcode::generating::svg::{generate_svg, SvgOptions};

use super::{Generator, Properties, Property, PropertyType, PropertyValue, RenderType};

#[derive(Default, Clone)]
pub struct SvgGenerator {
    options: SvgOptions,
    properties: Properties,
}

impl SvgGenerator {
    pub fn new() -> SvgGenerator {
        let options = SvgOptions::default();

        SvgGenerator {
            properties: vec![
                Property {
                    name: "include_background",
                    description: "Include the background",
                    kind: PropertyType::Bool,
                    default: Some(PropertyValue::Bool(options.include_background)),
                    ..Default::default()
                },
                Property {
                    name: "font_size",
                    description: "Font size",
                    kind: PropertyType::Int,
                    default: Some(PropertyValue::Int(options.font_size as i32)),
                    ..Default::default()
                },
                Property {
                    name: "bake_font",
                    description: "Whether to convert a font to points",
                    kind: PropertyType::Bool,
                    default: Some(PropertyValue::Bool(!options.write_options.preserve_text)),
                    ..Default::default()
                },
            ],
            options,
        }
    }
}

impl Generator for SvgGenerator {
    fn name(&self) -> &str {
        "SVG"
    }

    fn description(&self) -> &str {
        "Generate SVG"
    }

    fn kind(&self) -> &RenderType {
        &RenderType::Text
    }

    fn saveable(&self) -> &bool {
        &true
    }

    fn properties(&self) -> &Properties {
        &self.properties
    }

    fn get_property(&self, name: &str) -> Result<PropertyValue, super::GeneratorError> {
        match name {
            "include_background" => Ok(self.options.include_background.into()),
            "font_size" => Ok(self.options.font_size.into()),
            "bake_font" => Ok(PropertyValue::from(
                !self.options.write_options.preserve_text,
            )),
            _ => Err(super::GeneratorError::PropertyError(
                super::PropertyError::UnknownProperty,
            )),
        }
    }

    fn set_property(
        &mut self,
        name: &str,
        value: PropertyValue,
    ) -> Result<(), super::GeneratorError> {
        match name {
            "include_background" => {
                self.options.include_background = value.try_into()?;
            }
            "font_size" => {
                self.options.font_size = value.try_into()?;
            }
            "bake_font" => {
                self.options.write_options.preserve_text = !value.try_into()?;
            }
            _ => {
                return Err(super::GeneratorError::PropertyError(
                    super::PropertyError::UnknownProperty,
                ))
            }
        }

        Ok(())
    }

    fn generate(
            &self,
            text: &str,
            theme: &syntect::highlighting::Theme,
            syntax: &syntect::parsing::SyntaxReference,
            syntax_set: &syntect::parsing::SyntaxSet,
        ) -> Result<super::RenderOutput, super::GeneratorError> {
        if let Ok(svg) = generate_svg(text, theme, syntax, syntax_set, &self.options) {
            return Ok(super::RenderOutput::Text(svg));
        }

        Err(super::GeneratorError::Other("Failed to generate svg".to_string()))
    }
}
