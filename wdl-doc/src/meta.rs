//! Create HTML documentation for WDL meta sections.

use maud::Markup;
use maud::html;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::v1::MetadataSection;
use wdl_ast::v1::MetadataValue;

/// Render a [`MetadataValue`] as HTML.
fn render_value(value: &MetadataValue) -> Markup {
    match value {
        MetadataValue::String(s) => html! { (s.syntax().to_string()) },
        MetadataValue::Boolean(b) => html! { (b.syntax().to_string()) },
        MetadataValue::Integer(i) => html! { (i.syntax().to_string()) },
        MetadataValue::Float(f) => html! { (f.syntax().to_string()) },
        MetadataValue::Null(n) => html! { (n.syntax().to_string()) },
        MetadataValue::Array(a) => {
            html! {
                "["
                @for item in a.elements() {
                    (render_value(&item))
                    ","
                }
                "]"
            }
        }
        MetadataValue::Object(o) => {
            html! {
                "{"
                @for item in o.items() {
                    (item.name().syntax().to_string())
                    ":"
                    (render_value(&item.value()))
                    ","
                }
                "}"
            }
        }
    }
}

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
        let entries = self.0.items().filter_map(|entry| {
            let name = entry.name();
            if name.as_str() == "outputs" {
                return None;
            }
            let value = entry.value();
            Some((name.as_str().to_string(), value))
        });
        html! {
            h3 { "Meta" }
            ul {
                @for (name, value) in entries {
                    li {
                        b {
                            (name)
                            ":"
                        }
                        " "
                        (render_value(&value))
                    }
                }
            }
        }
    }
}
