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

/// A map of metadata key-value pairs, sorted by key.
pub(crate) type MetaMap = BTreeMap<String, MetadataValue>;

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
        MetadataValue::Boolean(b) => html! { code { (b.text().to_string()) } },
        MetadataValue::Integer(i) => html! { code { (i.text().to_string()) } },
        MetadataValue::Float(f) => html! { code { (f.text().to_string()) } },
        MetadataValue::Null(n) => html! { code { (n.text().to_string()) } },
        MetadataValue::Array(a) => {
            html! {
                @for item in a.elements() {
                    @match item {
                        MetadataValue::Array(_) | MetadataValue::Object(_) => {
                            (render_value_inner(&item))
                        }
                        _ => {
                            // TODO
                            code { (item.text().to_string()) }
                        }
                    }
                }
            }
        }
        MetadataValue::Object(o) => {
            html! {
                @for item in o.items() {
                    // TODO
                    b { (item.name().text()) ":" } " " (render_value_inner(&item.value())) ","
                }
            }
        }
    }
}

/// Render a [`MetadataValue`] as HTML.
pub(crate) fn render_value(value: &MetadataValue) -> Markup {
    render_value_inner(value)
}

/// Help key for custom rendering.
const HELP_KEY: &str = "help";
/// External help key for custom rendering.
const EXTERNAL_HELP_KEY: &str = "external_help";
/// Warning key for custom rendering.
const WARNING_KEY: &str = "warning";

/// Render a metadata map as HTML, filtering out certain keys and summarizing
/// values if needed.
pub(crate) fn render_meta_map(
    map: &MetaMap,
    filter_keys: &[&str],
    assets: &Path,
) -> Option<Markup> {
    let custom_keys = &[HELP_KEY, EXTERNAL_HELP_KEY, WARNING_KEY];
    let filtered_items = map
        .iter()
        .filter(|(k, _v)| !filter_keys.contains(&k.as_str()) && !custom_keys.contains(&k.as_str()))
        .collect::<Vec<_>>();

    let help_item = map.get(HELP_KEY);
    let external_help_item = map.get(EXTERNAL_HELP_KEY);
    let warning_item = map.get(WARNING_KEY);

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
            // TODO revisit this layout
            div class="main__table-outer-container main__metadata-table" {
                div class="main__table-inner-container" {
                    table class="main__table" {
                        tbody {
                            @for (k, v) in filtered_items {
                                tr {
                                    td { code {
                                        (k)
                                    } }
                                    td {
                                        (render_value(v))
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}

/// The maximum length of a markdown snippet before it is clipped.
const MAX_MD_LENGTH: usize = 140;
/// The amount of characters to show in the clipped markdown.
const MD_CLIP_LENGTH: usize = 120;

/// A description that may be truncated.
#[derive(Debug)]
pub(crate) enum MaybeTruncatedDescription {
    /// The description was truncated, providing a summary and the full
    /// markdown.
    Yes(Markup, Markup),
    /// The description was not truncated, providing the full markdown.
    No(Markup),
}

/// Render a markdown string, summarizing it if it exceeds the maximum length.
pub(crate) fn summarize_description_if_needed(description: &str) -> MaybeTruncatedDescription {
    if description.len() > MAX_MD_LENGTH {
        MaybeTruncatedDescription::Yes(
            html! { (format!("{}...", description[..MD_CLIP_LENGTH].trim_end())) },
            Markdown(description.to_string()).render(),
        )
    } else {
        MaybeTruncatedDescription::No(Markdown(description.to_string()).render())
    }
}
