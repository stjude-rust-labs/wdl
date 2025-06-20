//! Create HTML documentation for WDL parameters.

use std::path::Path;

use maud::Markup;
use maud::html;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::v1::Decl;
use wdl_ast::v1::MetadataValue;

use crate::meta::MetaMap;
use crate::meta::render_meta_map;
use crate::meta::render_value;

/// A group of inputs.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Group(pub String);

impl Group {
    /// Get the display name of the group.
    pub fn display_name(&self) -> String {
        self.0.clone()
    }

    /// Get the id of the group.
    pub fn id(&self) -> String {
        format!("inputs-{}", self.0.replace(" ", "-"))
    }
}

impl PartialOrd for Group {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Group {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.0 == "Common" {
            return std::cmp::Ordering::Less;
        }
        if other.0 == "Common" {
            return std::cmp::Ordering::Greater;
        }
        if self.0 == "Resources" {
            return std::cmp::Ordering::Greater;
        }
        if other.0 == "Resources" {
            return std::cmp::Ordering::Less;
        }
        self.0.cmp(&other.0)
    }
}

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
    pub fn _render_remaining_meta(&self, assets: &Path) -> Option<Markup> {
        render_meta_map(self.meta(), &["description", "group"], true, assets)
    }

    /// Render the parameter as HTML.
    pub fn render(&self, _assets: &Path) -> Markup {
        html! {
            div class="main__grid-row" {
                div class="main__grid-cell" {
                    code { (self.name()) }
                }
                div class="main__grid-cell" {
                    code { (self.ty()) }
                }
                @if self.required() != Some(true) {
                    div class="main__grid-cell" { (shorten_expr_if_needed(self.expr())) }
                }
                div class="main__grid-cell" {
                    (self.description(true))
                }
                // TODO collapsable row for additional metadata
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
                button type="button" class="main__button" x-on:click="expanded = true" {
                    b { "Show full expression" }
                }
            }
            div x-show="expanded" {
                p { code { (expr) } }
                button type="button" class="main__button" x-on:click="expanded = false" {
                    b { "Show less" }
                }
            }
        }
    }
}

/// Render a table for non-required parameters (both inputs and outputs
/// accepted).
///
/// A separate implementation is used for non-required parameters
/// because they require an extra column for the default value (when inputs)
/// or expression (when outputs). This may seem like a duplication on its
/// surface, but because of the way CSS/HTML grids work, this is the most
/// straightforward way to handle the different shape grids.
///
/// The distinction between inputs and outputs is made by checking if the
/// `required` field is `None` for all parameters. If any parameter has
/// `required` set to `Some(_)`, then all parameters are considered inputs.
pub(crate) fn render_non_required_parameters_table<'a, I>(params: I, assets: &Path) -> Markup
where
    I: Iterator<Item = &'a Parameter>,
{
    let params = params.collect::<Vec<_>>();

    let third_col = if params.iter().any(|p| p.required().is_none()) {
        // If any parameter is an output, we use "Expression" as the third column
        // header.
        "Expression"
    } else {
        // If all parameters are inputs, we use "Default" as the third column header.
        "Default"
    };

    html! {
        div class="main__grid-container" {
            div class="main__grid-non-req-param-container" {
                div class="main__grid-header-cell" { "Name" }
                div class="main__grid-header-cell" { "Type" }
                div class="main__grid-header-cell" { (third_col) }
                div class="main__grid-header-cell" { "Description" }
                div class="main__grid-header-separator" {}
                @for param in params {
                    (param.render(assets))
                }
            }
        }
    }
}
