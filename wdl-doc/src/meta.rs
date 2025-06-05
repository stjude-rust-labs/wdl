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
pub type MetaMap = BTreeMap<String, MetadataValue>;

/// Render a [`MetadataValue`] as HTML.
pub(crate) fn render_value(value: &MetadataValue, summarize_if_needed: bool) -> Markup {
    fn render_value_inner(value: &MetadataValue, summarize_if_needed: bool) -> Markup {
        match value {
            MetadataValue::String(s) => {
                let inner_text = s.text().map(|t| t.text().to_string()).unwrap_or_default();
                if summarize_if_needed {
                    return html! { (summarize_markdown_if_needed(inner_text)) };
                }
                Markdown(inner_text).render()
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
    summarize_if_needed: bool,
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

    let any_filtered_items = !filtered_items.is_empty();
    let custom_key_present =
        help_item.is_some() || external_help_item.is_some() || warning_item.is_some();
    if !any_filtered_items && !custom_key_present {
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
        div class="metadata__container" {
            @if let Some(help) = help_item {
                article class="prose" {
                    (render_value(help, summarize_if_needed))
                }
            }
            @if let Some(on_click) = external_link_on_click {
                button class="hover:cursor-pointer flex items-center gap-2" x-on:click=(on_click) {
                    b { "Go to External Documentation" }
                    img src=(assets.join("link.svg").to_string_lossy()) alt="External Documentation Icon" class="size-5";
                }
            }
            @if let Some(warning) = warning_item {
                div class="metadata__warning" {
                    img src=(assets.join("information-circle.svg").to_string_lossy()) alt="Warning Icon" class="size-5";
                    p { (render_value(warning, summarize_if_needed)) }
                }
            }
            @if any_filtered_items {
                div class="parameter__table-outer-container" {
                    div class="parameter__table-inner-container" {
                        table class="parameter__table" {
                            tbody {
                                @for (k, v) in filtered_items {
                                    tr {
                                        td class="text-mono" {
                                            (k)
                                        }
                                        td {
                                            (render_value(v, summarize_if_needed))
                                        }
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
