//! HTML generation for WDL callables (workflows and tasks).

pub mod task;
pub mod workflow;

use std::collections::HashMap;
use std::collections::HashSet;

use maud::Markup;
use maud::html;
use wdl_ast::AstToken;
use wdl_ast::v1::InputSection;
use wdl_ast::v1::MetadataSection;
use wdl_ast::v1::MetadataValue;
use wdl_ast::v1::OutputSection;
use wdl_ast::v1::ParameterMetadataSection;

use crate::meta::Meta;
use crate::meta::render_value;
use crate::parameter::InputOutput;
use crate::parameter::Parameter;

/// A callable (workflow or task) in a WDL document.
pub trait Callable {
    /// Get the name of the callable.
    fn name(&self) -> &str;

    /// Get the metadata section of the callable.
    fn metadata_section(&self) -> Option<&MetadataSection>;

    /// Get the parameter metadata section of the callable.
    fn parameter_metadata_section(&self) -> Option<&ParameterMetadataSection>;

    /// Get the input section of the callable.
    fn input_section(&self) -> Option<&InputSection>;

    /// Get the output section of the callable.
    fn output_section(&self) -> Option<&OutputSection>;

    fn parse_meta(&self) -> HashMap<String, MetadataValue> {
        self.metadata_section()
            .into_iter()
            .flat_map(|m| m.items())
            .map(|m| {
                let name = m.name().as_str().to_owned();
                let item = m.value();
                (name, item)
            })
            .collect()
    }

    fn parse_parameter_meta(&self) -> HashMap<String, MetadataValue> {
        self.parameter_metadata_section()
            .into_iter()
            .flat_map(|m| m.items())
            .map(|m| {
                let name = m.name().as_str().to_owned();
                let item = m.value();
                (name, item)
            })
            .collect()
    }

    fn parse_inputs(&self) -> Vec<Parameter> {
        let parameter_meta = self.parse_parameter_meta();

        self.input_section()
            .into_iter()
            .flat_map(|i| i.declarations())
            .map(|decl| {
                let name = decl.name().as_str().to_owned();
                let meta = parameter_meta.get(&name);
                Parameter::new(decl.clone(), meta.cloned(), InputOutput::Input)
            })
            .collect()
    }

    fn parse_outputs(&self) -> Vec<Parameter> {
        let meta = self.parse_meta();
        let parameter_meta = self.parse_parameter_meta();
        let output_meta: HashMap<String, MetadataValue> = meta
            .get("outputs")
            .and_then(|v| match v {
                MetadataValue::Object(o) => Some(o),
                _ => None,
            })
            .and_then(|o| Some(o.items().map(|i| (i.name().as_str().to_owned(), i.value().clone())).collect()))
            .unwrap_or_default();

        self.output_section()
            .into_iter()
            .flat_map(|o| o.declarations())
            .map(|decl| {
                let name = decl.name().as_str().to_owned();
                let meta = parameter_meta.get(&name).or_else(|| output_meta.get(&name));
                Parameter::new(wdl_ast::v1::Decl::Bound(decl.clone()), meta.cloned(), InputOutput::Output)
            })
            .collect()
    }

    /// Get the description of the callable.
    fn description(&self) -> Markup {
        if let Some(meta_section) = self.metadata_section() {
            for entry in meta_section.items() {
                if entry.name().as_str() == "description" {
                    return render_value(&entry.value());
                }
            }
        }
        html! {}
    }

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

    /// Render the meta section of the callable.
    fn render_meta(&self) -> Markup {
        if let Some(meta_section) = self.metadata_section() {
            Meta::new(meta_section.clone()).render(&HashSet::from([
                "description".to_string(),
                "outputs".to_string(),
            ]))
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
