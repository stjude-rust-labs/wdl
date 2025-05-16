//! Create HTML documentation for WDL meta sections.

use maud::Markup;
use maud::html;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::v1::MetadataValue;

use crate::Markdown;
use crate::Render;

/// Default threshold for collapsing long strings.
const DEFAULT_THRESHOLD: usize = 80;

/// Render a [`MetadataValue`] as HTML.
pub(crate) fn render_value(value: &MetadataValue) -> Markup {
    fn render_value_inner(value: &MetadataValue, top_level: bool) -> Markup {
        match value {
            MetadataValue::String(s) => {
                let inner_text = s.text().map(|t| t.text().to_string()).unwrap_or_default();
                if top_level {
                    return html! { (wrap_markdown_in_details_if_needed(inner_text, DEFAULT_THRESHOLD)) };
                }
                html! { (s.text().map(|t| t.text().to_string()).unwrap_or_default()) }
            }
            MetadataValue::Boolean(b) => html! { code { (b.text().to_string()) } },
            MetadataValue::Integer(i) => html! { code { (i.text().to_string()) } },
            MetadataValue::Float(f) => html! { code { (f.text().to_string()) } },
            MetadataValue::Null(n) => html! { code { (n.text().to_string()) } },
            MetadataValue::Array(a) => {
                let full = html! {
                    div {
                        code { "[" }
                        ul {
                            @for item in a.elements() {
                                li {
                                    @match item {
                                        MetadataValue::Array(_) | MetadataValue::Object(_) => {
                                            (render_value_inner(&item, false)) ","
                                        }
                                        _ => {
                                            code { (item.text().to_string()) } ","
                                        }
                                    }
                                }
                            }
                        }
                        code { "]" }
                    }
                };
                html! {
                    details {
                        summary {
                            (format!("Array with {} elements... ", a.elements().collect::<Vec<_>>().len()))
                            b { "Expand" }
                        }
                        (full)
                    }
                }
            }
            MetadataValue::Object(o) => {
                let full = html! {
                    div {
                        code { "{" }
                        ul {
                            @for item in o.items() {
                                li {
                                    b { (item.name().text()) ":" } " " (render_value_inner(&item.value(), false)) ","
                                }
                            }
                        }
                        code { "}" }
                    }
                };
                html! {
                    details {
                        summary {
                            (format!("Object with {} items... ", o.items().collect::<Vec<_>>().len()))
                            b { "Expand" }
                        }
                        (full)
                    }
                }
            }
        }
    }

    render_value_inner(value, true)
}

/// Helper function to wrap markdown strings in details element if it exceeds a
/// length threshold.
fn wrap_markdown_in_details_if_needed(content: String, threshold: usize) -> Markup {
    if content.len() <= threshold {
        return Markdown(content).render();
    }

    let markup = Markdown(content.clone()).render();

    let summary_text = format!("{}... ", &content[..threshold].trim());

    html! {
        details {
            summary {
                (summary_text)
                b { "Read more" }
            }
            div {
                (markup)
            }
        }
    }
}
