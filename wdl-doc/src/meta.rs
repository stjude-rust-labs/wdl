//! Create HTML documentation for WDL meta sections.

use maud::Markup;
use maud::html;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::v1::MetadataValue;

use crate::Markdown;
use crate::Render;

/// Render a [`MetadataValue`] as HTML.
pub(crate) fn render_value(value: &MetadataValue, summarize_if_needed: bool) -> Markup {
    fn render_value_inner(value: &MetadataValue, summarize_if_needed: bool) -> Markup {
        match value {
            MetadataValue::String(s) => {
                let inner_text = s.text().map(|t| t.text().to_string()).unwrap_or_default();
                if summarize_if_needed {
                    return html! { (summarize_markdown_if_needed(inner_text)) };
                }
                return Markdown(inner_text).render();
            }
            MetadataValue::Boolean(b) => html! { code { (b.text().to_string()) } },
            MetadataValue::Integer(i) => html! { code { (i.text().to_string()) } },
            MetadataValue::Float(f) => html! { code { (f.text().to_string()) } },
            MetadataValue::Null(n) => html! { code { (n.text().to_string()) } },
            MetadataValue::Array(a) => {
                html! {
                    div x-data="{ expanded: false }" {
                        div x-show="!expanded" {
                            p { (format!("Array with {} elements... ", a.elements().collect::<Vec<_>>().len())) }
                            button class="hover:cursor-pointer" x-on:click="expanded = true" {
                                b { "Expand" }
                            }
                        }
                        div x-show="expanded" {
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
                            br;
                            button class="hover:cursor-pointer" x-on:click="expanded = false" {
                                b { "Collapse" }
                            }
                        }
                    }
                }
            }
            MetadataValue::Object(o) => {
                html! {
                    div x-data="{ expanded: false }" {
                        div x-show="!expanded" {
                            p { (format!("Object with {} items... ", o.items().collect::<Vec<_>>().len())) }
                            button class="hover:cursor-pointer" x-on:click="expanded = true" {
                                b { "Expand" }
                            }
                        }
                        div x-show="expanded" {
                            code { "{" }
                            ul {
                                @for item in o.items() {
                                    li {
                                        b { (item.name().text()) ":" } " " (render_value_inner(&item.value(), false)) ","
                                    }
                                }
                            }
                            code { "}" }
                            br;
                            button class="hover:cursor-pointer" x-on:click="expanded = false" {
                                b { "Collapse" }
                            }
                        }
                    }
                }
            }
        }
    }

    render_value_inner(value, summarize_if_needed)
}

/// The maximum length of a markdown snippet before it is clipped.
const MAX_MD_LENGTH: usize = 140;
/// The amount of characters to show in the clipped markdown.
const MD_CLIP_LENGTH: usize = 120;

/// Summarize a long string if it exceeds the threshold.
fn summarize_markdown_if_needed(content: String) -> Markup {
    if content.len() <= MAX_MD_LENGTH {
        return Markdown(content).render();
    }

    let markup = Markdown(content.clone()).render();

    let summary_text = format!("{}... ", &content[..MD_CLIP_LENGTH].trim());

    html! {
        div x-data="{ expanded: false }" {
            div x-show="!expanded" {
                p { (summary_text) }
                button class="hover:cursor-pointer" x-on:click="expanded = true" {
                    b { "Read more" }
                }
            }
            div x-show="expanded" {
                (markup)
                br;
                button class="hover:cursor-pointer" x-on:click="expanded = false" {
                    b { "Read less" }
                }
            }
        }
    }
}
