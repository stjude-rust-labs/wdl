//! Create HTML documentation for WDL parameters.

use std::path::Path;

use maud::Markup;
use maud::html;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::v1::Decl;
use wdl_ast::v1::MetadataValue;

use crate::callable::Group;
use crate::meta::MetaMap;
use crate::meta::render_meta_map;
use crate::meta::render_value;

/// Whether a parameter is an input or output.
#[derive(Debug, Clone, Copy)]
pub(crate) enum InputOutput {
    /// An input parameter.
    Input,
    /// An output parameter.
    Output,
}

/// A parameter (input or output) in a workflow or task.
#[derive(Debug)]
pub(crate) struct Parameter {
    /// The declaration of the parameter.
    decl: Decl,
    /// Any meta entries associated with the parameter.
    meta: MetaMap,
    /// Whether the parameter is an input or output.
    io: InputOutput,
}

impl Parameter {
    /// Create a new parameter.
    pub fn new(decl: Decl, meta: Option<MetadataValue>, io: InputOutput) -> Self {
        let meta = match meta {
            Some(ref m) => {
                match m {
                    MetadataValue::Object(o) => o
                        .items()
                        .map(|item| (item.name().text().to_string(), item.value().clone()))
                        .collect(),
                    MetadataValue::String(_s) => {
                        MetaMap::from([("description".to_string(), m.clone())])
                    }
                    _ => {
                        // If it's not an object or string, we don't know how to handle it.
                        MetaMap::default()
                    }
                }
            }
            None => MetaMap::default(),
        };
        Self { decl, meta, io }
    }

    /// Get the name of the parameter.
    pub fn name(&self) -> String {
        self.decl.name().text().to_owned()
    }

    /// Get the meta of the parameter.
    pub fn meta(&self) -> &MetaMap {
        &self.meta
    }

    /// Get the type of the parameter.
    pub fn ty(&self) -> String {
        self.decl.ty().to_string()
    }

    /// Get the Expr value of the parameter as a String.
    pub fn expr(&self) -> String {
        self.decl
            .expr()
            .map(|expr| expr.text().to_string())
            .unwrap_or("None".to_string())
    }

    /// Get whether the input parameter is required.
    ///
    /// Returns `None` for outputs.
    pub fn required(&self) -> Option<bool> {
        match self.io {
            InputOutput::Input => match self.decl.as_unbound_decl() {
                Some(d) => Some(!d.ty().is_optional()),
                _ => Some(false),
            },
            InputOutput::Output => None,
        }
    }

    /// Get the "group" of the parameter.
    pub fn group(&self) -> Option<Group> {
        self.meta().get("group").and_then(|value| {
            if let MetadataValue::String(s) = value {
                Some(Group(
                    s.text().map(|t| t.text().to_string()).unwrap_or_default(),
                ))
            } else {
                None
            }
        })
    }

    /// Get the description of the parameter.
    pub fn description(&self, summarize_if_needed: bool) -> Markup {
        self.meta()
            .get("description")
            .map(|v| render_value(v, summarize_if_needed))
            .unwrap_or_else(|| html! { "No description provided." })
    }

    /// Render any remaining metadata as HTML.
    ///
    /// This will render any metadata that is not rendered elsewhere if present.
    pub fn render_remaining_meta(&self, assets: &Path) -> Option<Markup> {
        render_meta_map(self.meta(), &["description", "group"], true, assets)
    }

    /// Render the parameter as HTML.
    pub fn render(&self, addl_meta: bool, assets: &Path) -> Markup {
        html! {
            tr {
                td { (self.name()) }
                td { code { (self.ty()) } }
                @if self.required() != Some(true) {
                    td { (shorten_expr_if_needed(self.expr())) }
                }
                td { (self.description(true)) }
                @if addl_meta {
                    @if let Some(markup) = self.render_remaining_meta(assets) {
                        td { (markup) }
                    } @else {
                        td { }
                    }
                }
            }
        }
    }
}

/// The maximum length of an expression before it is clipped.
const MAX_EXPR_LENGTH: usize = 80;
/// The amount of characters to show in the clipped expression.
const EXPR_CLIP_LENGTH: usize = 60;

/// Render a WDL expression as HTML, with a show more button if it exceeds a
/// certain length.
pub(crate) fn shorten_expr_if_needed(expr: String) -> Markup {
    if expr.len() <= MAX_EXPR_LENGTH {
        return html! { code { (expr) } };
    }

    let clipped_expr = expr[..EXPR_CLIP_LENGTH].trim();

    html! {
        div x-data="{ expanded: false }" {
            div x-show="!expanded" {
                p { code { (clipped_expr) } "..." }
                button class="hover:cursor-pointer" x-on:click="expanded = true" {
                    b { "Show full expression" }
                }
            }
            div x-show="expanded" {
                p { code { (expr) } }
                button class="hover:cursor-pointer" x-on:click="expanded = false" {
                    b { "Show less" }
                }
            }
        }
    }
}

/// Render a table with the given headers and parameters
///
/// If any of the parameters return `Some(_)` for `render_remaining_meta()`, an
/// "Additional Meta" column will be added to the table.
pub(crate) fn render_parameter_table<'a, I>(headers: &[&str], params: I, assets: &Path) -> Markup
where
    I: Iterator<Item = &'a Parameter>,
{
    let params = params.collect::<Vec<_>>();
    let addl_meta = params
        .iter()
        .any(|param| param.render_remaining_meta(assets).is_some());

    html! {
        div class="main__table-outer-container" {
            div class="main__table-inner-container" {
                table class="main__table" {
                    thead { tr {
                        @for header in headers {
                            th { (header) }
                        }
                        @if addl_meta {
                            th { "Additional Meta" }
                        }
                    }}
                    tbody {
                        @for param in params {
                            (param.render(addl_meta, assets))
                        }
                    }
                }
            }
        }
    }
}
