use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt::Debug, sync::{atomic::AtomicBool, mpsc::Sender, Arc}};

use syntect::{
    highlighting::Theme,
    parsing::{SyntaxReference, SyntaxSet},
};
use ts_rs::TS;

use super::property::*;

pub mod svg;
pub use svg::SvgGenerator;

pub mod resolve;
pub use resolve::FusionGenerator;

type Properties = Vec<PropertyInfo>;
type Extensions = Vec<&'static str>;

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase", tag = "kind")]
#[ts(export)]
pub enum GeneratorEvent {
    Started,
    Cancelled,
    Progress { total: usize, current: usize },
}

impl GeneratorEvent {
    pub fn progress(total: usize, current: usize) -> GeneratorEvent {
        GeneratorEvent::Progress { total, current }
    }
}

#[derive(Debug, Clone)]
pub struct GeneratorContext {
    pub event_tx: Sender<GeneratorEvent>,
    pub cancel: Arc<AtomicBool>,
}

impl GeneratorContext {
    pub fn new(event_tx: Sender<GeneratorEvent>) -> GeneratorContext {
        GeneratorContext {
            event_tx: event_tx.clone(),
            cancel: Arc::new(AtomicBool::new(false)),
        }
    }
}

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
        text: &str,
        theme: &Theme,
        syntax: &SyntaxReference,
        syntax_set: &SyntaxSet,
        options: &GeneratorOptions,
        context: &GeneratorContext
    ) -> Result<String>;
}

pub trait GeneratorExt {
    fn information() -> GeneratorInfo;
}
