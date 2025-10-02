use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt::Debug};

use syntect::{
    highlighting::Theme,
    parsing::{SyntaxReference, SyntaxSet},
};
use ts_rs::TS;

use super::property::*;

pub mod svg;
pub use svg::SvgGenerator;

type Properties = Vec<PropertyInfo>;
type Extensions = Vec<&'static str>;

#[derive(Debug, Default, Clone, Serialize, TS)]
#[ts(export)]
pub struct GeneratorInfo {
    /// The name of the generator
    name: &'static str,
    /// The description of a generator
    description: &'static str,
    /// The output language syntax of the generated code if any
    syntax: Option<&'static str>,
    /// The extensions supported by the generator if any
    extensions: Option<Extensions>,
    /// Defines extra properties defined by the generator if any
    properties: Option<Properties>,
    /// Whether the generator result should/can be saved
    saveable: bool,
}

impl GeneratorInfo {
    pub fn name(&self) -> &str {
        self.name
    }
    pub fn description(&self) -> &str {
        self.description
    }
    pub fn extensions(&self) -> Option<Extensions> {
        self.extensions.clone()
    }
    pub fn saveable(&self) -> bool {
        self.saveable
    }
}

#[derive(Debug, Default, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GeneratorOptions {
    pub font_size: f32,
    pub font_family: String,
    pub extra: BTreeMap<String, PropertyValue>,
}

pub trait Generator: Send + Sync + Debug {
    fn generate_code(
        &self,
        _text: &str,
        _theme: &Theme,
        _syntax: &SyntaxReference,
        _syntax_set: &SyntaxSet,
        _options: &GeneratorOptions,
    ) -> Result<String>;
}

pub trait GeneratorExt {
    fn information() -> GeneratorInfo;
}
