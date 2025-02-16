//! Create HTML documentation for WDL workflows.

use std::path::Path;

use maud::Markup;
use maud::html;
use wdl_ast::v1::MetadataSection;

use super::Callable;
use crate::full_page;
use crate::parameter::Parameter;

/// A task in a WDL document.
#[derive(Debug)]
pub struct Workflow {
    /// The name of the task.
    name: String,
    /// The meta section of the task.
    meta_section: Option<MetadataSection>,
    /// The input parameters of the task.
    inputs: Vec<Parameter>,
    /// The output parameters of the task.
    outputs: Vec<Parameter>,
}

impl Workflow {
    /// Create a new task.
    pub fn new(
        name: String,
        meta_section: Option<MetadataSection>,
        inputs: Vec<Parameter>,
        outputs: Vec<Parameter>,
    ) -> Self {
        Self {
            name,
            meta_section,
            inputs,
            outputs,
        }
    }

    /// Render the workflow as HTML.
    pub fn render(&self, parent_dir: &Path) -> Markup {
        let body = html! {
            h1 { (self.name()) }
            (self.description())
            (self.render_meta())
            (self.render_inputs())
            (self.render_outputs())
        };

        full_page(self.name(), parent_dir, body)
    }
}

impl Callable for Workflow {
    /// Get the name of the callable.
    fn name(&self) -> &str {
        &self.name
    }

    /// Get the meta section of the callable.
    fn meta(&self) -> Option<&MetadataSection> {
        self.meta_section.as_ref()
    }

    /// Get the input parameters of the callable.
    fn inputs(&self) -> &[Parameter] {
        &self.inputs
    }

    /// Get the output parameters of the callable.
    fn outputs(&self) -> &[Parameter] {
        &self.outputs
    }
}
