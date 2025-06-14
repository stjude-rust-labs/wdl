//! Create HTML documentation for WDL workflows.

use maud::Markup;
use wdl_ast::AstToken;
use wdl_ast::SupportedVersion;
use wdl_ast::v1::MetadataValue;
use wdl_ast::v1::WorkflowDefinition;

use super::*;
use crate::docs_tree::Header;
use crate::docs_tree::PageHeaders;
use crate::meta::render_meta_map;
use crate::meta::render_value;
use crate::parameter::Parameter;

/// A workflow in a WDL document.
#[derive(Debug)]
pub(crate) struct Workflow {
    /// The name of the workflow.
    name: String,
    /// The version of WDL this workflow is defined in.
    version: VersionBadge,
    /// The meta of the workflow.
    meta: MetaMap,
    /// The inputs of the workflow.
    inputs: Vec<Parameter>,
    /// The outputs of the workflow.
    outputs: Vec<Parameter>,
}

impl Workflow {
    /// Create a new workflow.
    pub fn new(name: String, version: SupportedVersion, definition: WorkflowDefinition) -> Self {
        let meta = match definition.metadata() {
            Some(mds) => parse_meta(&mds),
            _ => MetaMap::default(),
        };
        let parameter_meta = match definition.parameter_metadata() {
            Some(pmds) => parse_parameter_meta(&pmds),
            _ => MetaMap::default(),
        };
        let inputs = match definition.input() {
            Some(is) => parse_inputs(&is, &parameter_meta),
            _ => Vec::new(),
        };
        let outputs = match definition.output() {
            Some(os) => parse_outputs(&os, &meta, &parameter_meta),
            _ => Vec::new(),
        };

        Self {
            name,
            version: VersionBadge::new(version),
            meta,
            inputs,
            outputs,
        }
    }

    /// Returns the `name` entry from the meta section, if it exists.
    pub fn name_override(&self) -> Option<Markup> {
        self.meta.get("name").map(|v| render_value(v, false))
    }

    /// Returns the "pretty" name of the workflow as HTML.
    pub fn pretty_name(&self) -> Markup {
        if let Some(name) = self.name_override() {
            name
        } else {
            html! { (self.name) }
        }
    }

    /// Returns the `category` entry from the meta section, if it exists.
    pub fn category(&self) -> Option<String> {
        self.meta.get("category").and_then(|v| match v {
            MetadataValue::String(s) => Some(s.text().unwrap().text().to_string()),
            _ => None,
        })
    }

    /// Renders the meta section of the workflow as HTML.
    ///
    /// This will render all metadata key-value pairs except for `name`,
    /// `category`, `description`, `allowNestedInputs`/`allow_nested_inputs`,
    /// and `outputs`.
    pub fn render_meta(&self, assets: &Path) -> Option<Markup> {
        render_meta_map(
            self.meta(),
            &[
                "name",
                "category",
                "outputs",
                "description",
                "allowNestedInputs",
                "allow_nested_inputs",
            ],
            false,
            assets,
        )
    }

    /// Render the `allowNestedInputs`/`allow_nested_inputs` meta entry as HTML.
    pub fn render_allow_nested_inputs(&self) -> Markup {
        if let Some(MetadataValue::Boolean(b)) = self
            .meta
            .get("allowNestedInputs")
            .or(self.meta.get("allow_nested_inputs"))
        {
            if b.value() {
                return html! {
                    div class="main__badge-green" {
                        span class="main__badge-green-text" {
                            "Nested Inputs Allowed"
                        }
                    }
                };
            }
        }
        html! {
            div class="main__badge" {
                span class="main__badge-text" {
                    "Nested Inputs Not Allowed"
                }
            }
        }
    }

    /// Render the workflow as HTML.
    pub fn render(&self, assets: &Path) -> (Markup, PageHeaders) {
        let mut headers = PageHeaders::default();
        let meta_markup = if let Some(meta) = self.render_meta(assets) {
            headers.push(Header::Header("Meta".to_string(), "meta".to_string()));
            meta
        } else {
            html! {}
        };

        let (input_markup, inner_headers) = self.render_inputs(assets);

        headers.extend(inner_headers);

        let markup = html! {
            div class="main__container" {
                div class="main__section" {
                    article class="main__prose" {
                        span class="text-emerald-400 not-prose" { "Workflow" }
                        h1 id="title" class="main__title" { (self.pretty_name()) }
                        div class="flex flex-row not-prose items-center gap-2" {
                            (self.version().render())
                            @if let Some(category) = self.category() {
                                div class="main__badge" {
                                    span class="main__badge-text" {
                                        "Category"
                                    }
                                    div class="main__badge-inner" {
                                        span class="main__badge-inner-text" {
                                            (category)
                                        }
                                    }
                                }
                            }
                            (self.render_allow_nested_inputs())
                        }
                        (self.description(false))
                        (meta_markup)
                        (input_markup)
                        (self.render_outputs(assets))
                    }
                }
            }
        };

        headers.push(Header::Header("Outputs".to_string(), "outputs".to_string()));

        (markup, headers)
    }
}

impl Callable for Workflow {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &VersionBadge {
        &self.version
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
    use wdl_ast::version::V1;

    use super::*;

    #[test]
    fn test_workflow() {
        let (doc, _) = Document::parse(
            r#"
            version 1.0
            workflow test {
                input {
                    String name
                }
                output {
                    String greeting = "Hello, ${name}!"
                }
            }
            "#,
        );

        let doc_item = doc.ast().into_v1().unwrap().items().next().unwrap();
        let ast_workflow = doc_item.into_workflow_definition().unwrap();

        let workflow = Workflow::new(
            ast_workflow.name().text().to_string(),
            SupportedVersion::V1(V1::Zero),
            ast_workflow,
        );

        assert_eq!(workflow.name(), "test");
        assert_eq!(workflow.inputs.len(), 1);
        assert_eq!(workflow.outputs.len(), 1);
    }
}
