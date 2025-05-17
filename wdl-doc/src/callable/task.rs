//! Create HTML documentation for WDL tasks.

use maud::Markup;
use maud::html;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::v1::InputSection;
use wdl_ast::v1::MetadataSection;
use wdl_ast::v1::OutputSection;
use wdl_ast::v1::ParameterMetadataSection;
use wdl_ast::v1::RuntimeSection;

use super::*;
use crate::docs_tree::Header;
use crate::docs_tree::PageHeaders;
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
        let meta = match meta_section {
            Some(mds) => parse_meta(&mds),
            _ => MetaMap::default(),
        };
        let parameter_meta = match parameter_meta {
            Some(pmds) => parse_parameter_meta(&pmds),
            _ => MetaMap::default(),
        };
        let inputs = match input_section {
            Some(is) => parse_inputs(&is, &parameter_meta),
            _ => Vec::new(),
        };
        let outputs = match output_section {
            Some(os) => parse_outputs(&os, &meta, &parameter_meta),
            _ => Vec::new(),
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
    pub fn render_meta(&self) -> Option<Markup> {
        let kv = self
            .meta
            .iter()
            .filter(|(k, _)| !matches!(k.as_str(), "outputs" | "description"))
            .collect::<Vec<_>>();
        if kv.is_empty() {
            return None;
        }
        Some(html! {
            h2 id="meta" { "Meta" }
            @for (key, value) in kv {
                p {
                    b { (key) ":" } " " (render_value(value))
                }
            }
        })
    }

    /// Render the runtime section of the task as HTML.
    pub fn render_runtime_section(&self) -> Markup {
        match &self.runtime_section {
            Some(runtime_section) => {
                html! {
                    h2 id="runtime" { "Default Runtime Attributes" }
                    table class="border" {
                        thead class="border" { tr {
                            th { "Attribute" }
                            th { "Value" }
                        }}
                        tbody class="border" {
                            @for entry in runtime_section.items() {
                                tr class="border" {
                                    td class="border" { code { (entry.name().text()) } }
                                    td class="border" { code { ({let e = entry.expr(); e.text().to_string() }) } }
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                html! {}
            }
        }
    }

    /// Render the task as HTML.
    pub fn render(&self) -> (Markup, PageHeaders) {
        let mut headers = PageHeaders::default();
        let meta_markup = if let Some(meta) = self.render_meta() {
            headers.push(Header::Header("Meta".to_string(), "meta".to_string()));
            meta
        } else {
            html! {}
        };

        let (input_markup, inner_headers) = self.render_inputs();
        headers.extend(inner_headers);

        let markup = html! {
            div class="flex flex-col gap-y-6" {
                h1 id="title" { (self.name()) }
                (self.description())
                (meta_markup)
                (input_markup)
                (self.render_outputs())
                (self.render_runtime_section())
            }
        };
        headers.push(Header::Header("Outputs".to_string(), "outputs".to_string()));
        headers.push(Header::Header("Runtime".to_string(), "runtime".to_string()));

        (markup, headers)
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

#[cfg(test)]
mod tests {
    use wdl_ast::Document;

    use super::*;

    #[test]
    fn test_task() {
        let (doc, _) = Document::parse(
            r#"
            version 1.0

            task my_task {
                input {
                    String name
                }
                output {
                    String greeting = "Hello, ${name}!"
                }
                runtime {
                    docker: "ubuntu:latest"
                }
                meta {
                    description: "A simple task"
                }
            }
            "#,
        );

        let doc_item = doc.ast().into_v1().unwrap().items().next().unwrap();
        let ast_task = doc_item.into_task_definition().unwrap();

        let task = Task::new(
            ast_task.name().text().to_owned(),
            ast_task.metadata(),
            ast_task.parameter_metadata(),
            ast_task.input(),
            ast_task.output(),
            ast_task.runtime(),
        );

        assert_eq!(task.name(), "my_task");
        assert_eq!(
            task.meta()
                .get("description")
                .unwrap()
                .clone()
                .unwrap_string()
                .text()
                .unwrap()
                .text(),
            "A simple task"
        );
        assert_eq!(task.inputs().len(), 1);
        assert_eq!(task.outputs().len(), 1);
    }
}
