//! Create HTML documentation for WDL structs.
// TODO: handle >=v1.2 structs

use maud::Markup;
use maud::html;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::SupportedVersion;
use wdl_ast::v1::StructDefinition;

use crate::VersionBadge;
use crate::docs_tree::PageHeaders;

/// A struct in a WDL document.
#[derive(Debug)]
pub struct Struct {
    /// The AST definition of the struct.
    definition: StructDefinition,
    /// The version of WDL this struct is defined in.
    version: VersionBadge,
}

impl Struct {
    /// Create a new struct.
    pub fn new(definition: StructDefinition, version: SupportedVersion) -> Self {
        Self {
            definition,
            version: VersionBadge::new(version),
        }
    }

    /// Render the struct as HTML.
    pub fn render(&self) -> (Markup, PageHeaders) {
        let name = self.definition.name();
        let name = name.text();
        let markup = html! {
            div class="main__container" {
                div class="main__section" {
                    article class="main__prose" {
                        p class="text-pink-400 not-prose" { "Struct" }
                        h1 id="title" class="main__title" { code { (name) } }
                        (self.version.render())
                        sprocket-code language="wdl" { (self.definition.inner().to_string()) }
                    }
                }
            }
        };
        (markup, PageHeaders::default())
    }
}
