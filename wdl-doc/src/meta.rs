//! Create HTML documentation for WDL meta sections.

use std::fmt::Display;

use maud::Markup;
use maud::html;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::v1::MetadataSection;

/// A meta section in a WDL document.
#[derive(Debug)]
pub struct Meta(MetadataSection);

impl Meta {
    /// Create a new meta section.
    pub fn new(meta: MetadataSection) -> Self {
        Self(meta)
    }

    /// Render the meta section as HTML.
    pub fn render(&self) -> Markup {
        html! {
            h3 { "Meta" }
            ul {
                @for entry in self.0.items() {
                    li {
                        b {
                            (entry.name().as_str())
                            ":"
                        }
                        " "
                        (entry.value().syntax().to_string())
                    }
                }
            }
        }
    }
}

impl Display for Meta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let markup = self.render();

        write!(f, "{}", markup.into_string())
    }
}
