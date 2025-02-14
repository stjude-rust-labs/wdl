//! Create HTML documentation for WDL tasks.

use std::collections::HashSet;
use std::path::Path;

use maud::Markup;
use maud::html;
use wdl_ast::v1::MetadataSection;
use wdl_ast::AstToken;

use crate::full_page;
use crate::meta::Meta;
use crate::meta::render_value;
use crate::parameter::Parameter;

/// A task in a WDL document.
#[derive(Debug)]
pub struct Task {
    /// The name of the task.
    name: String,
    /// The meta section of the task.
    meta_section: Option<MetadataSection>,
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

    /// Get the desciption of the task as HTML.
    pub fn description(&self) -> Markup {
        if let Some(meta_section) = &self.meta_section {
            for entry in meta_section.items() {
                if entry.name().as_str() == "description" {
                    return render_value(&entry.value());
                }
            }
        }
        html! {}
    }

    /// Get the meta section of the task as HTML.
    pub fn meta_section(&self) -> Markup {
        if let Some(meta_section) = &self.meta_section {
            Meta::new(meta_section.clone()).render()
        } else {
            html! {}
        }
    }

    /// Get the input parameters of the task.
    pub fn inputs(&self) -> &[Parameter] {
        &self.inputs
    }

    /// Get the required inputs of the task.
    pub fn required_inputs(&self) -> impl Iterator<Item = &Parameter> {
        self.inputs.iter().filter(|param| {
            param
                .required()
                .expect("inputs should return Some(required)")
        })
    }

    /// Get the set of unique `group` values of the inputs.
    pub fn input_groups(&self) -> HashSet<String> {
        self.inputs
            .iter()
            .filter_map(|param| param.group().as_ref().map(|s| s.to_string()))
            .collect()
    }

    /// Get the inputs of the task that are part of `group`.
    pub fn inputs_in_group<'a>(&'a self, group: &'a str) -> impl Iterator<Item = &'a Parameter> {
        self.inputs.iter().filter(move |param| {
            if let Some(param_group) = param.group() {
                if param_group == group {
                    return true;
                }
            }
            false
        })
    }

    /// Get the inputs of the task that are neither required nor part of a
    /// group.
    pub fn other_inputs(&self) -> impl Iterator<Item = &Parameter> {
        self.inputs.iter().filter(|param| {
            !param
                .required()
                .expect("inputs should return Some(required)")
                && param.group().is_none()
        })
    }

    /// Get the output parameters of the task.
    pub fn outputs(&self) -> &[Parameter] {
        &self.outputs
    }

    /// Render the task as HTML.
    pub fn render(&self, stylesheet: &Path) -> Markup {
        let mut iter = self.required_inputs().peekable();
        let required_table = if iter.peek().is_some() {
            Some(html! {
                h3 { "Required Inputs" }
                table class="border" {
                    thead class="border" { tr {
                        th { "Name" }
                        th { "Type" }
                        th { "Description" }
                        th { "Additional Meta" }
                    }}
                    tbody class="border" {
                        @for param in iter {
                            (param.render())
                        }
                    }
                }
            })
        } else {
            None
        };

        let mut iter = self.inputs_in_group("Common").peekable();
        let common_table = if iter.peek().is_some() {
            Some(html! {
                h3 { "Common" }
                table class="border" {
                    thead class="border" { tr {
                        th { "Name" }
                        th { "Type" }
                        th { "Default" }
                        th { "Description" }
                        th { "Additional Meta" }
                    }}
                    tbody class="border" {
                        @for param in iter {
                            (param.render())
                        }
                    }
                }
            })
        } else {
            None
        };

        let group_tables = self
            .input_groups()
            .into_iter()
            .filter(|group| *group != "Common")
            .map(|group| {
                html! {
                    h3 { (group) }
                    table class="border" {
                        thead class="border" { tr {
                            th { "Name" }
                            th { "Type" }
                            th { "Default" }
                            th { "Description" }
                            th { "Additional Meta" }
                        }}
                        tbody class="border" {
                            @for param in self.inputs_in_group(&group) {
                                (param.render())
                            }
                        }
                    }
                }
            });

        let mut iter = self.other_inputs().peekable();
        let other_table = if iter.peek().is_some() {
            Some(html! {
                h3 { "Other Inputs" }
                table class="border" {
                    thead class="border" { tr {
                        th { "Name" }
                        th { "Type" }
                        th { "Default" }
                        th { "Description" }
                        th { "Additional Meta" }
                    }}
                    tbody class="border" {
                        @for param in iter {
                            (param.render())
                        }
                    }
                }
            })
        } else {
            None
        };

        let body = html! {
            div class="table-auto border-collapse" {
                h1 { (self.name()) }
                (self.description())
                (self.meta_section())
                h2 { "Inputs" }
                @if let Some(required_table) = required_table {
                    (required_table)
                }
                @if let Some(common_table) = common_table {
                    (common_table)
                }
                @for group_table in group_tables {
                    (group_table)
                }
                @if let Some(other_table) = other_table {
                    (other_table)
                }
                h2 { "Outputs" }
                table  {
                    thead class="border" { tr {
                        th { "Name" }
                        th { "Type" }
                        th { "Expression" }
                        th { "Description" }
                        th { "Additional Meta" }
                    }}
                    tbody class="border" {
                        @for param in self.outputs() {
                            (param.render())
                        }
                    }
                }
            }
        };

        full_page(self.name(), stylesheet, body)
    }
}
