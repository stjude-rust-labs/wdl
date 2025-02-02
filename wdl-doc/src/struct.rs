//! Create HTML documentation for WDL structs.

use std::path::Path;

use maud::Markup;
use maud::html;
use wdl_ast::AstToken;
use wdl_ast::v1::StructDefinition;

use crate::full_page;

/// A struct in a WDL document.
#[derive(Debug)]
pub struct Struct {
    /// The AST definition of the struct.
    def: StructDefinition,
}

impl Struct {
    /// Create a new struct.
    pub fn new(def: StructDefinition) -> Self {
        Self { def }
    }

    /// Get the name of the struct.
    pub fn name(&self) -> String {
        self.def.name().as_str().to_string()
    }

    /// Get the members of the struct.
    pub fn members(&self) -> impl Iterator<Item = (String, String)> + '_ {
        self.def.members().map(|decl| {
            let name = decl.name().as_str().to_owned();
            let ty = decl.ty().to_string();
            (name, ty)
        })
    }

    /// Render the struct as HTML.
    pub fn render(&self, stylesheet: &Path) -> Markup {
        let body = html! {
            h1 { (self.name()) }
            h3 { "Members" }
            ul {
                @for (name, ty) in self.members() {
                    li {
                        b {
                            (name)
                            ":"
                        }
                        " " (ty)
                    }
                }
            }
        };

        full_page(&self.name(), stylesheet, body)
    }
}
