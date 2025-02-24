//! Create HTML documentation for WDL workflows.

use maud::Markup;
use wdl_ast::AstToken;
use wdl_ast::v1::InputSection;
use wdl_ast::v1::MetadataSection;
use wdl_ast::v1::MetadataValue;
use wdl_ast::v1::OutputSection;
use wdl_ast::v1::ParameterMetadataSection;

use super::*;
use crate::meta::render_value;
use crate::parameter::Parameter;

/// A task in a WDL document.
#[derive(Debug)]
pub struct Workflow {
    /// The name of the workflow.
    name: String,
    /// The meta of the workflow.
    meta: MetaMap,
    /// The inputs of the workflow.
    inputs: Vec<Parameter>,
    /// The outputs of the workflow.
    outputs: Vec<Parameter>,
}

impl Workflow {
    /// Create a new workflow.
    pub fn new(
        name: String,
        meta_section: Option<MetadataSection>,
        parameter_meta: Option<ParameterMetadataSection>,
        input_section: Option<InputSection>,
        output_section: Option<OutputSection>,
    ) -> Self {
        let meta = if let Some(ref mds) = meta_section {
            parse_meta(mds)
        } else {
            MetaMap::default()
        };
        let parameter_meta = if let Some(pmds) = parameter_meta {
            parse_parameter_meta(&pmds)
        } else {
            MetaMap::default()
        };
        let inputs = if let Some(is) = input_section {
            parse_inputs(&is, &parameter_meta)
        } else {
            Vec::new()
        };
        let outputs = if let Some(os) = output_section {
            parse_outputs(&os, &meta, &parameter_meta)
        } else {
            Vec::new()
        };

        Self {
            name,
            meta,
            inputs,
            outputs,
        }
    }

    /// Returns the `name` entry from the meta section, if it exists.
    pub fn name_override(&self) -> Option<Markup> {
        self.meta.get("name").map(|v| render_value(v))
    }

    /// Returns the `category` entry from the meta section, if it exists.
    pub fn category(&self) -> Option<String> {
        self.meta.get("category").and_then(|v| match v {
            MetadataValue::String(s) => Some(s.text().unwrap().as_str().to_string()),
            _ => None,
        })
    }

    // /// Render the workflow as HTML.
    // pub fn render(&self, docs_tree: &DocsTree, stylesheet: &Path) -> Markup {
    //     let body = html! {
    //         h1 { @if let Some(name_override) = self.name_override() {
    // (name_override) } @else { (self.name()) } }         @if let
    // Some(category) = self.category() {             h2 { "Category: "
    // (category) }         }
    //         (self.description())
    //         (self.render_meta())
    //         (self.render_inputs())
    //         (self.render_outputs())
    //     };

    //     // TODO
    //     body
    // }
}

impl Callable for Workflow {
    fn name(&self) -> &str {
        &self.name
    }

    fn meta(&self) -> &MetaMap {
        &self.meta
    }

    fn inputs(&self) -> &[Parameter] {
        &self.inputs
    }

    fn outputs(&self) -> &[Parameter] {
        &self.outputs
    }
}
