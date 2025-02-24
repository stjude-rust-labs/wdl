//! Create HTML documentation for WDL workflows.

use std::collections::HashSet;
use std::path::Path;

use maud::Markup;
use maud::html;
use wdl_ast::AstToken;
use wdl_ast::v1::MetadataSection;
use wdl_ast::v1::MetadataValue;

use super::Callable;
use crate::DocsTree;
use crate::full_page;
use crate::meta::Meta;
use crate::meta::render_value;
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

    /// Returns the `name` entry from the meta section, if it exists.
    pub fn name_override(&self) -> Option<Markup> {
        if let Some(meta_section) = self.meta_section.as_ref() {
            for entry in meta_section.items() {
                if entry.name().as_str() == "name" {
                    return Some(render_value(&entry.value()));
                }
            }
        }
        None
    }

    /// Returns the `category` entry from the meta section, if it exists.
    pub fn category(&self) -> Option<String> {
        if let Some(meta_section) = self.meta_section.as_ref() {
            for entry in meta_section.items() {
                if entry.name().as_str() == "category" {
                    match entry.value() {
                        MetadataValue::String(s) => {
                            return Some(
                                s.text().map(|t| t.as_str().to_string()).unwrap_or_default(),
                            );
                        }
                        _ => return None,
                    }
                }
            }
        }
        None
    }

    /// Render the workflow as HTML.
    pub fn render(&self, docs_tree: &DocsTree, stylesheet: &Path) -> Markup {
        let body = html! {
            h1 { @if let Some(name_override) = self.name_override() { (name_override) } @else { (self.name()) } }
            @if let Some(category) = self.category() {
                h2 { "Category: " (category) }
            }
            (self.description())
            (self.render_meta())
            (self.render_inputs())
            (self.render_outputs())
        };

        full_page(self.name(), docs_tree, stylesheet, body)
    }
}

impl Callable for Workflow {
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

    fn render_meta(&self) -> Markup {
        if let Some(meta_section) = self.meta() {
            Meta::new(meta_section.clone()).render(&HashSet::from([
                "description".to_string(),
                "outputs".to_string(),
                "name".to_string(),
                "category".to_string(),
            ]))
        } else {
            html! {}
        }
    }
}
