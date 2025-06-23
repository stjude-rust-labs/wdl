//! Create HTML documentation for WDL meta sections.

use std::collections::BTreeMap;
use std::path::Path;

use maud::Markup;
use maud::html;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::v1::MetadataValue;

use crate::Markdown;
use crate::Render;

/// Help key for custom rendering.
const HELP_KEY: &str = "help";
/// External help key for custom rendering.
const EXTERNAL_HELP_KEY: &str = "external_help";
/// Warning key for custom rendering.
const WARNING_KEY: &str = "warning";

/// A map of metadata key-value pairs, sorted by key.
pub(crate) type MetaMap = BTreeMap<String, MetadataValue>;

/// An extension trait for [`MetaMap`] to provide additional functionality
/// commonly used in WDL documentation generation.
pub(crate) trait MetaMapExt {
    /// Returns the rendered [`Markup`] of the `description` key, optionally
    /// summarizing it.
    fn render_description(&self, summarize: bool) -> Markup;
    /// Returns the rendered [`Markup`] of the remaining metadata keys,
    /// excluding the keys specified in `filter_keys`.
    fn render_remaining(&self, filter_keys: &[&str], assets: &Path) -> Option<Markup>;
}

impl MetaMapExt for MetaMap {
    fn render_description(&self, summarize: bool) -> Markup {
        let desc = self
            .get("description")
            .map(|v| match v {
                MetadataValue::String(s) => {
                    let t = s.text().expect("meta string should not be interpolated");
                    t.text().to_string()
                }
                _ => "ERROR: description not of type String".to_string(),
            })
            .unwrap_or_else(|| "No description provided".to_string());

        if !summarize {
            return Markdown(desc).render();
        }

        match summarize_if_needed(&desc) {
            MaybeSummarized::No(desc) => Markdown(desc).render(),
            MaybeSummarized::Yes(summary) => {
                html! {
                    (summary)
                    button type="button" class="main__button" x-on:click="description_expanded = !description_expanded" x-text="description_expanded ? 'Show less' : 'Show full description'" {}
                }
            }
        }
    }

    fn render_remaining(&self, filter_keys: &[&str], assets: &Path) -> Option<Markup> {
        let custom_keys = &[HELP_KEY, EXTERNAL_HELP_KEY, WARNING_KEY];
        let filtered_items = self
            .iter()
            .filter(|(k, _v)| {
                !filter_keys.contains(&k.as_str()) && !custom_keys.contains(&k.as_str())
            })
            .collect::<Vec<_>>();

        let help_item = self.get(HELP_KEY);
        let external_help_item = self.get(EXTERNAL_HELP_KEY);
        let warning_item = self.get(WARNING_KEY);

        let any_additional_items = !filtered_items.is_empty();
        let custom_key_present =
            help_item.is_some() || external_help_item.is_some() || warning_item.is_some();
        if !any_additional_items && !custom_key_present {
            return None;
        }

        let external_link = external_help_item.map(|v| match v {
            MetadataValue::String(s) => {
                let text = s.text().expect("meta string should not be interpolated");
                let mut buffer = String::new();
                text.unescape_to(&mut buffer);
                Some(buffer)
            }
            _ => None,
        });
        let external_link_on_click = if let Some(Some(link)) = external_link {
            Some(format!("window.open('{}', '_blank')", link))
        } else {
            None
        };

        Some(html! {
            @if let Some(help) = help_item {
                div class="markdown-body" {
                    (render_value(help))
                }
            }
            @if let Some(on_click) = external_link_on_click {
                button type="button" class="main__button" x-on:click=(on_click) {
                    b { "Go to External Documentation" }
                    img src=(assets.join("link.svg").to_string_lossy()) alt="External Documentation Icon" class="size-5";
                }
            }
            @if let Some(warning) = warning_item {
                div class="metadata__warning" {
                    img src=(assets.join("information-circle.svg").to_string_lossy()) alt="Warning Icon" class="size-5";
                    p { (render_value(warning)) }
                }
            }
            @if any_additional_items {
                div class="main__grid-container" {
                    div class="main__grid-addl-meta-container" {
                        // No header row, just the items
                        @for (key, value) in filtered_items {
                            div class="main__grid-row" {
                                div class="main__grid-cell" {
                                    code { (key) }
                                }
                                div class="main__grid-cell" {
                                    (render_value(value))
                                }
                            }
                        }
                    }
                }
            }
        })
    }
}

/// Recursively render a [`MetadataValue`] as HTML.
fn render_value_inner(value: &MetadataValue) -> Markup {
    match value {
        MetadataValue::String(s) => {
            let inner_text = s
                .text()
                .map(|t| t.text().to_string())
                .expect("meta string should not be interpolated");
            Markdown(inner_text).render()
        }
        MetadataValue::Boolean(b) => html! { code { (b.text()) } },
        MetadataValue::Integer(i) => html! { code { (i.text()) } },
        MetadataValue::Float(f) => html! { code { (f.text()) } },
        MetadataValue::Null(n) => html! { code { (n.text()) } },
        MetadataValue::Array(a) => {
            html! {
                @for item in a.elements() {
                    @match item {
                        MetadataValue::Array(_) | MetadataValue::Object(_) => {
                            // TODO better handling of recursive structures
                            (render_value_inner(&item))
                        }
                        _ => {
                            // TODO revisit this
                            code { (item.text()) }
                        }
                    }
                }
            }
        }
        MetadataValue::Object(o) => {
            html! {
                div class="main__grid-container" {
                    // TODO revisit this
                    div class="main__grid-addl-meta-container" {
                        @for item in o.items() {
                            div class="main__grid-row" {
                                div class="main__grid-cell" {
                                    code { (item.name().text()) }
                                }
                                div class="main__grid-cell" {
                                    (render_value(&item.value()))
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Render a [`MetadataValue`] as HTML.
pub(crate) fn render_value(value: &MetadataValue) -> Markup {
    render_value_inner(value)
}

/// The maximum length of a string before it is clipped.
const MAX_LENGTH: usize = 80;
/// The amount of characters to show in the summary.
const CLIP_LENGTH: usize = 60;

/// A string that may be summarized.
// TODO return reference to the original string?
#[derive(Debug)]
pub(crate) enum MaybeSummarized {
    /// The string was truncated, providing a summary.
    Yes(String),
    /// The string was not truncated, providing the full thing.
    No(String),
}

/// Summarize a string if it exceeds a maximum length.
pub(crate) fn summarize_if_needed(in_str: &str) -> MaybeSummarized {
    if in_str.len() > MAX_LENGTH {
        MaybeSummarized::Yes(format!("{}...", in_str[..CLIP_LENGTH].trim_end()))
    } else {
        MaybeSummarized::No(in_str.to_string())
    }
}
