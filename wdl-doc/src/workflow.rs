//! Create HTML documentation for WDL workflows.

use std::path::Path;

use maud::Markup;
use maud::html;
use wdl_ast::v1::MetadataSection;

use crate::full_page;
use crate::meta::Meta;
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

    /// Get the name of the task.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the meta section of the workflow as HTML.
    pub fn meta_section(&self) -> Markup {
        if let Some(meta_section) = &self.meta_section {
            let meta = Meta::new(meta_section.clone());
            meta.render()
        } else {
            html! {}
        }
    }

    /// Get the input parameters of the task.
    pub fn inputs(&self) -> &[Parameter] {
        &self.inputs
    }

    /// Get the output parameters of the task.
    pub fn outputs(&self) -> &[Parameter] {
        &self.outputs
    }

    /// Render the workflow as HTML.
    pub fn render(&self, parent_dir: &Path) -> Markup {
        let body = html! {
            h1 { (self.name()) }
            (self.meta_section())
            h2 { "Inputs" }
            ul {
                @for param in self.inputs() {
                    li {
                        (param.render())
                    }
                }
            }
            h2 { "Outputs" }
            ul {
                @for param in self.outputs() {
                    li {
                        (param.render())
                    }
                }
            }
        };

        full_page(self.name(), parent_dir, body)
    }
}
