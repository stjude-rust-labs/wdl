//! Create HTML documentation for WDL structs.
// TODO: handle >=v1.2 structs

use maud::Markup;
use maud::html;
use wdl_ast::AstNode;
use wdl_ast::v1::StructDefinition;

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

    // /// Get the name of the struct.
    // pub fn name(&self) -> String {
    //     self.def.name().text().to_string()
    // }

    // /// Get the members of the struct.
    // pub fn members(&self) -> impl Iterator<Item = (String, String)> + '_ {
    //     self.def.members().map(|decl| {
    //         let name = decl.name().text().to_owned();
    //         let ty = decl.ty().to_string();
    //         (name, ty)
    //     })
    // }

    /// Render the struct as HTML.
    pub fn render(&self) -> (Markup, PageHeaders) {
        let markup = html! {
            sprocket-code language="wdl" { (self.def.inner().to_string()) }
        };
        (markup, PageHeaders::default())
    }
}
