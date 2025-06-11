//! Create HTML documentation for WDL structs.
// TODO: handle >=v1.2 structs

use maud::Markup;
use maud::html;
use wdl_ast::AstNode;
use wdl_ast::v1::StructDefinition;
use wdl_ast::AstToken;

use crate::docs_tree::PageHeaders;

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

    /// Render the struct as HTML.
    pub fn render(&self) -> (Markup, PageHeaders) {
        let name = self.def.name();
        let name = name.text();
        let markup = html! {
            div class="main__container" {
                div class="main__section" {
                    article class="main__prose" {
                        p class="text-pink-400 not-prose" { "Struct" }
                        h1 id="title" class="main__title" { code { (name) } }
                        sprocket-code language="wdl" { (self.def.inner().to_string()) }
                    }
                }
            }
        };
        (markup, PageHeaders::default())
    }
}
