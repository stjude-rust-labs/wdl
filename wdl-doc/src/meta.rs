//! Create HTML documentation for WDL meta sections.

use maud::Markup;
use maud::html;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::v1::MetadataSection;
use wdl_ast::v1::MetadataValue;

use crate::Markdown;
use crate::Render;

/// Render a [`MetadataValue`] as HTML.
pub(crate) fn render_value(value: &MetadataValue) -> Markup {
    match value {
        MetadataValue::String(s) => {
            html! { (Markdown(s.text().map(|t| t.as_str().to_string()).unwrap_or_default()).render()) }
        }
        MetadataValue::Boolean(b) => html! { code { (b.syntax().to_string()) } },
        MetadataValue::Integer(i) => html! { code { (i.syntax().to_string()) } },
        MetadataValue::Float(f) => html! { code { (f.syntax().to_string()) } },
        MetadataValue::Null(n) => html! { code { (n.syntax().to_string()) } },
        MetadataValue::Array(a) => {
            html! {
                "["
                ul {
                    @for item in a.elements() {
                        li {
                            (render_value(&item)) ","
                        }
                    }
                }
                "]"
            }
        }
        MetadataValue::Object(o) => {
            html! {
                "{"
                ul {
                    @for item in o.items() {
                        li {
                            (item.name().syntax().to_string()) ":" (render_value(&item.value())) ","
                        }
                    }
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
                        p { b { (name) ":" } " " (render_value(&value)) }
                    }
                }
            }
        }
    }
}
