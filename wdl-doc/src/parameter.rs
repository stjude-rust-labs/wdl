//! Create HTML documentation for WDL parameters.

use maud::Markup;
use maud::html;
use wdl_ast::AstToken;
use wdl_ast::v1::Decl;
use wdl_ast::v1::MetadataValue;
use crate::meta::render_value;

/// A parameter (input or output) in a workflow or task.
#[derive(Debug)]
pub struct Parameter {
    /// The declaration of the parameter.
    decl: Decl,
    /// Any meta entries associated with the parameter.
    meta: Option<MetadataValue>,
}

impl Parameter {
    /// Create a new parameter.
    pub fn new(decl: Decl, meta: Option<MetadataValue>) -> Self {
        Self { decl, meta }
    }

    /// Get the name of the parameter.
    pub fn name(&self) -> String {
        self.decl.name().as_str().to_owned()
    }

    /// Get the type of the parameter.
    pub fn ty(&self) -> String {
        self.decl.ty().to_string()
    }

    /// Get the Expr value of the parameter.
    pub fn expr(&self) -> Option<String> {
        self.decl.expr().map(|expr| expr.syntax().to_string())
    }

    /// Get the meta entries associated with the parameter.
    pub fn meta(&self) -> Option<&MetadataValue> {
        self.meta.as_ref()
    }

    /// Render the parameter as HTML.
    pub fn render(&self) -> Markup {
        html! {
            h3 { (self.name()) }
            p { "Type: " (self.ty()) }
            @if let Some(expr) = self.expr() {
                p { "Expr: " (expr) }
            } @else {
                p { "Expr: None" }
            }
            p { "Meta: " (self.meta().map(render_value).unwrap_or_else(|| html! { "None" })) }
        }
    }
}
