use super::{
    Generator, GeneratorError, GeneratorInfo, Info, Properties, Property,
    PropertyError, PropertyType, PropertyValue,
};
use quellcode::generating::svg::{generate_svg, SvgOptions};

#[derive(Clone, Debug)]
pub struct SvgGenerator {
    options: SvgOptions,
    properties: Properties,
}

impl SvgGenerator {
    pub fn new() -> SvgGenerator {
        SvgGenerator::default()
    }
}

impl Default for SvgGenerator {
    fn default() -> SvgGenerator {
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
    fn font_family(&self) -> &str {
        &self.options.font_family
    }

    fn set_font_family(&mut self, family: &str) {
        self.options.font_family = family.to_string();
    }

    fn font_size(&self) -> f32 {
        self.options.font_size
    }

    fn set_font_size(&mut self, size: f32) {
        self.options.font_size = size;
    }

    fn generate_code(
        &self,
        text: &str,
        theme: &syntect::highlighting::Theme,
        syntax: &syntect::parsing::SyntaxReference,
        syntax_set: &syntect::parsing::SyntaxSet,
    ) -> Result<String, GeneratorError> {
        if let Ok(svg) = generate_svg(text, theme, syntax, syntax_set, &self.options) {
            return Ok(svg);
        }

        Err(super::GeneratorError::Other(
            "Failed to generate svg".to_string(),
        ))
    }

    fn properties(&self) -> &Properties {
        &self.properties
    }

    fn get_property(&self, name: &str) -> Result<PropertyValue, super::GeneratorError> {
        match name {
            "include_background" => Ok(self.options.include_background.into()),
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
            "bake_font" => {
                self.options.write_options.preserve_text = !value.try_into()?;
            }
            _ => return Err(PropertyError::UnknownProperty)?,
        }

        Ok(())
    }
}

impl GeneratorInfo for SvgGenerator {
    fn information() -> Info {
        Info {
            name: "SVG".to_string(),
            description: "Generates svg".to_string(),
            extensions: Some(vec!["svg"]),
            saveable: true,
        }
    }
}
