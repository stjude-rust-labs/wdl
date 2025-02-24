//! Create HTML documentation for WDL tasks.

use std::path::Path;

use maud::Markup;
use maud::html;
use wdl_ast::AstToken;
use wdl_ast::v1::MetadataSection;
use wdl_ast::v1::RuntimeSection;

use super::Callable;
use crate::DocsTree;
use crate::full_page;
use crate::parameter::Parameter;

/// A task in a WDL document.
#[derive(Debug)]
pub struct Task {
    /// The name of the task.
    name: String,
    /// The meta section of the task.
    meta_section: Option<MetadataSection>,
    /// The runtime section of the task.
    runtime_section: Option<RuntimeSection>,
    /// The input parameters of the task.
    inputs: Vec<Parameter>,
    /// The output parameters of the task.
    outputs: Vec<Parameter>,
}

impl Task {
    /// Create a new task.
    pub fn new(
        name: String,
        meta_section: Option<MetadataSection>,
        runtime_section: Option<RuntimeSection>,
        inputs: Vec<Parameter>,
        outputs: Vec<Parameter>,
    ) -> Self {
        Self {
            name,
            meta_section,
            runtime_section,
            inputs,
            outputs,
        }
    }

    /// Get the rutime section of the task as HTML.
    pub fn render_runtime_section(&self) -> Markup {
        if let Some(runtime_section) = &self.runtime_section {
            html! {
                h2 { "Default Runtime Attributes" }
                table class="border" {
                    thead class="border" { tr {
                        th { "Attribute" }
                        th { "Value" }
                    }}
                    tbody class="border" {
                        @for entry in runtime_section.items() {
                            tr class="border" {
                                td class="border" { code { (entry.name().as_str()) } }
                                td class="border" { code { (entry.expr().syntax().to_string()) } }
                            }
                        }
                    }
                }
            }
        } else {
            html! {}
        }
    }

    /// Render the task as HTML.
    pub fn render(&self, docs_tree: &DocsTree, stylesheet: &Path) -> Markup {
        let body = html! {
            div class="table-auto border-collapse" {
                h1 { (self.name()) }
                (self.description())
                (self.render_meta())
                (self.render_inputs())
                (self.render_outputs())
                (self.render_runtime_section())
            }
        };

        full_page(self.name(), docs_tree, stylesheet, body)
    }
}

impl Callable for Task {
    fn name(&self) -> &str {
        &self.name
    }

    fn meta(&self) -> Option<&MetadataSection> {
        self.meta_section.as_ref()
    }

    fn inputs(&self) -> &[Parameter] {
        &self.inputs
    }

    fn outputs(&self) -> &[Parameter] {
        &self.outputs
    }
}
