//! Create HTML documentation for WDL tasks.

use maud::Markup;
use maud::html;
use wdl_ast::AstToken;
use wdl_ast::v1::InputSection;
use wdl_ast::v1::MetadataSection;
use wdl_ast::v1::OutputSection;
use wdl_ast::v1::ParameterMetadataSection;
use wdl_ast::v1::RuntimeSection;

use super::*;
use crate::parameter::Parameter;

/// A task in a WDL document.
#[derive(Debug)]
pub struct Task {
    /// The name of the task.
    name: String,
    /// The meta of the task.
    meta: MetaMap,
    /// The input parameters of the task.
    inputs: Vec<Parameter>,
    /// The output parameters of the task.
    outputs: Vec<Parameter>,
    /// The runtime section of the task.
    runtime_section: Option<RuntimeSection>,
}

impl Task {
    /// Create a new task.
    pub fn new(
        name: String,
        meta_section: Option<MetadataSection>,
        parameter_meta: Option<ParameterMetadataSection>,
        input_section: Option<InputSection>,
        output_section: Option<OutputSection>,
        runtime_section: Option<RuntimeSection>,
    ) -> Self {
        let meta = if let Some(mds) = meta_section {
            parse_meta(&mds)
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
            runtime_section,
        }
    }

    /// Render the meta section of the task as HTML.
    ///
    /// This will render all metadata key-value pairs except for `outputs` and
    /// `description`.
    pub fn render_meta(&self) -> Markup {
        let mut kv = self
            .meta
            .iter()
            .filter(|(k, _)| !matches!(k.as_str(), "outputs" | "description"))
            .peekable();
        html! {
            @if kv.peek().is_some() {
                h2 { "Meta" }
                @for (key, value) in kv {
                    p {
                        b { (key) ":" } " " (render_value(value))
                    }
                }
            }
        }
    }

    /// Render the rutime section of the task as HTML.
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
    pub fn render(&self) -> Markup {
        html! {
            div class="table-auto border-collapse" {
                h1 { (self.name()) }
                (self.description())
                (self.render_meta())
                (self.render_inputs())
                (self.render_outputs())
                (self.render_runtime_section())
            }
        }
    }
}

impl Callable for Task {
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
