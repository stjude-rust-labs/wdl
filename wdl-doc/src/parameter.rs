//! Create HTML documentation for WDL parameters.

use maud::Markup;
use maud::html;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::v1::Decl;
use wdl_ast::v1::MetadataValue;

use crate::DEFAULT_THRESHOLD;
use crate::callable::Group;
use crate::meta::render_value;

/// Whether a parameter is an input or output.
#[derive(Debug, Clone, Copy)]
pub enum InputOutput {
    /// An input parameter.
    Input,
    /// An output parameter.
    Output,
}

/// A parameter (input or output) in a workflow or task.
#[derive(Debug)]
pub struct Parameter {
    /// The declaration of the parameter.
    decl: Decl,
    /// Any meta entries associated with the parameter.
    meta: Option<MetadataValue>,
    /// Whether the parameter is an input or output.
    io: InputOutput,
}

impl Parameter {
    /// Create a new parameter.
    pub fn new(decl: Decl, meta: Option<MetadataValue>, io: InputOutput) -> Self {
        Self { decl, meta, io }
    }

    /// Get the name of the parameter.
    pub fn name(&self) -> String {
        self.decl.name().text().to_owned()
    }

    /// Get the type of the parameter.
    pub fn ty(&self) -> String {
        self.decl.ty().to_string()
    }

    /// Get whether the parameter is an input or output.
    pub fn io(&self) -> InputOutput {
        self.io
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
        if let Some(MetadataValue::Object(o)) = &self.meta {
            for item in o.items() {
                if item.name().text() == "group" {
                    if let MetadataValue::String(s) = item.value() {
                        return s.text().map(|t| t.text().to_string()).map(Group);
                    }
                }
            }
        }
        None
    }

    /// Get the description of the parameter.
    pub fn description(&self) -> Markup {
        if let Some(meta) = &self.meta {
            if let MetadataValue::String(_) = meta {
                return render_value(meta);
            } else if let MetadataValue::Object(o) = meta {
                for item in o.items() {
                    if item.name().text() == "description" {
                        if let MetadataValue::String(_) = item.value() {
                            return render_value(&item.value());
                        }
                    }
                }
            }
        }
        html! {}
    }

    /// Render any remaining metadata as HTML.
    ///
    /// This will render any metadata that is not rendered elsewhere if present.
    pub fn render_remaining_meta(&self) -> Option<Markup> {
        if let Some(MetadataValue::Object(o)) = &self.meta {
            let filtered_items = o
                .items()
                .filter(|item| item.name().text() != "description" && item.name().text() != "group")
                .collect::<Vec<_>>();
            if filtered_items.is_empty() {
                return None;
            }
            return Some(html! {
                ul {
                    @for item in filtered_items {
                        li {
                            b { (item.name().text()) ":" } " " (render_value(&item.value()))
                        }
                    }
                }
            });
        }
        None
    }

    /// Render the parameter as HTML.
    pub fn render(&self, addl_meta: bool) -> Markup {
        html! {
            tr {
                td { (self.name()) }
                td { code { (self.ty()) } }
                @if self.required() != Some(true) {
                    td { (shorten_expr_if_needed(self.expr(), DEFAULT_THRESHOLD)) }
                }
                td { (self.description()) }
                @if addl_meta {
                    @if let Some(markup) = self.render_remaining_meta() {
                        td { (markup) }
                    } @else {
                        td { }
                    }
                }
            }
        }
    }
}

/// Render a WDL expression as HTML, with a "Read more" button if it exceeds a
/// certain length.
fn shorten_expr_if_needed(expr: String, threshold: usize) -> Markup {
    if expr.len() <= threshold {
        return html! { code { (expr) } };
    }

    let clipped_expr = expr[..threshold].trim();

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
pub(crate) fn render_parameter_table<'a, I>(headers: &[&str], params: I) -> Markup
where
    I: Iterator<Item = &'a Parameter>,
{
    let params = params.collect::<Vec<_>>();
    let addl_meta = params
        .iter()
        .any(|param| param.render_remaining_meta().is_some());

    html! {
        div class="workflow__table-outer-container" {
            div class="workflow__table-inner-container" {
                table class="workflow__table" {
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
                            (param.render(addl_meta))
                        }
                    }
                }
            }
        }
    }
}
