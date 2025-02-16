//! HTML generation for WDL callables (workflows and tasks).

pub mod task;
pub mod workflow;

use std::collections::HashSet;

use maud::Markup;
use maud::html;
use wdl_ast::AstToken;
use wdl_ast::v1::MetadataSection;

use crate::meta::Meta;
use crate::meta::render_value;
use crate::parameter::Parameter;

/// A callable (workflow or task) in a WDL document.
pub trait Callable {
    /// Get the name of the callable.
    fn name(&self) -> &str;

    /// Get the meta section of the callable.
    fn meta(&self) -> Option<&MetadataSection>;

    /// Get the description of the callable.
    fn description(&self) -> Markup {
        if let Some(meta_section) = self.meta() {
            for entry in meta_section.items() {
                if entry.name().as_str() == "description" {
                    return render_value(&entry.value());
                }
            }
        }
        html! {}
    }

    /// Get the input parameters of the callable.
    fn inputs(&self) -> &[Parameter];

    /// Get the required input parameters of the callable.
    fn required_inputs(&self) -> impl Iterator<Item = &Parameter> {
        self.inputs().iter().filter(|param| {
            param
                .required()
                .expect("inputs should return Some(required)")
        })
    }

    /// Get the set of unique `group` values of the inputs.
    fn input_groups(&self) -> HashSet<String> {
        self.inputs()
            .iter()
            .filter_map(|param| param.group().as_ref().map(|s| s.to_string()))
            .collect()
    }

    /// Get the inputs of the callable that are part of `group`.
    fn inputs_in_group<'a>(&'a self, group: &'a str) -> impl Iterator<Item = &'a Parameter> {
        self.inputs().iter().filter(move |param| {
            if let Some(param_group) = param.group() {
                if param_group == group {
                    return true;
                }
            }
            false
        })
    }

    /// Get the inputs of the callable that are neither required nor part of a
    /// group.
    fn other_inputs(&self) -> impl Iterator<Item = &Parameter> {
        self.inputs().iter().filter(|param| {
            !param
                .required()
                .expect("inputs should return Some(required)")
                && param.group().is_none()
        })
    }

    /// Get the output parameters of the callable.
    fn outputs(&self) -> &[Parameter];

    /// Render the meta section of the callable.
    fn render_meta(&self) -> Markup {
        if let Some(meta_section) = self.meta() {
            Meta::new(meta_section.clone()).render()
        } else {
            html! {}
        }
    }

    /// Render the required inputs of the callable.
    fn render_required_inputs(&self) -> Markup {
        let mut iter = self.required_inputs().peekable();
        if iter.peek().is_some() {
            return html! {
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
            };
        };
        html! {}
    }

    /// Render the common inputs of the callable.
    fn render_common_inputs(&self) -> Markup {
        let mut iter = self.inputs_in_group("Common").peekable();
        if iter.peek().is_some() {
            return html! {
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
            };
        };
        html! {}
    }

    /// Render the inputs with a group of the callable.
    fn render_group_inputs(&self) -> Markup {
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
        html! {
            @for group_table in group_tables {
                (group_table)
            }
        }
    }

    /// Render the inputs that are neither required nor part of a group.
    fn render_other_inputs(&self) -> Markup {
        let mut iter = self.other_inputs().peekable();
        if iter.peek().is_some() {
            return html! {
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
            };
        };
        html! {}
    }

    /// Render the inputs of the callable.
    fn render_inputs(&self) -> Markup {
        html! {
            h2 { "Inputs" }
            (self.render_required_inputs())
            (self.render_common_inputs())
            (self.render_group_inputs())
            (self.render_other_inputs())
        }
    }

    /// Render the outputs of the callable.
    fn render_outputs(&self) -> Markup {
        html! {
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
    }
}
