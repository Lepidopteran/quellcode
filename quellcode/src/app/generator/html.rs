use super::{
    Generator, GeneratorError, GeneratorInfo, Info, Properties, Property, PropertyError,
    PropertyType, PropertyValue,
};

#[derive(Clone, Debug)]
pub struct HtmlGenerator {
    properties: Properties,
}

impl HtmlGenerator {
    pub fn new() -> HtmlGenerator {
        HtmlGenerator::default()
    }
}

impl GeneratorInfo for HtmlGenerator {
    fn information() -> Info {
        Info {
            name: "HTML".to_string(),
            description: "Generates html snippet".to_string(),
            extensions: Some(vec!["html"]),
            saveable: true,
        }
    }
}
