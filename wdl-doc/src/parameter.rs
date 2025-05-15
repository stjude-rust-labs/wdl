//! Create HTML documentation for WDL parameters.

use maud::Markup;
use maud::html;
use maud::PreEscaped;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::v1::Decl;
use wdl_ast::v1::MetadataValue;

use crate::callable::Group;
use crate::meta::render_value;

/// Helper function to extract plain text from HTML content.
/// 
/// This function takes HTML content and returns a plain text version by:
/// 1. Removing all HTML tags while preserving their text content
/// 2. Converting common HTML entities to their character equivalents
/// 3. Normalizing whitespace (collapsing multiple spaces, trimming)
/// 
/// We use a simple state machine approach instead of regex to avoid dependencies.
/// The state machine tracks whether we're:
/// - Inside an HTML tag (to skip it)
/// - Inside an HTML entity (to convert it)
/// - In regular text (to keep it)
/// 
/// This is used to get an accurate character count of visible text and create
/// clean, HTML-free summaries for the details/summary elements.
fn extract_plain_text(html_content: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;        // True when between '<' and '>' chars
    let mut in_entity = false;     // True when between '&' and ';' chars
    let mut entity_buffer = String::new();
    
    // Process each character, maintaining state about tags and entities
    for c in html_content.chars() {
        match c {
            '<' => {
                in_tag = true;
                continue;
            }
            '>' => {
                if in_tag {
                    in_tag = false;
                    continue;
                }
                result.push(c);
            }
            '&' => {
                in_entity = true;
                entity_buffer.clear();
                entity_buffer.push('&');
            }
            ';' => {
                if in_entity {
                    in_entity = false;
                    // Convert common HTML entities to their character equivalents
                    let entity = entity_buffer.as_str();
                    if entity == "&lt" {
                        result.push('<');
                    } else if entity == "&gt" {
                        result.push('>');
                    } else if entity == "&amp" {
                        result.push('&');
                    } else if entity == "&quot" {
                        result.push('"');
                    } else if entity == "&apos" {
                        result.push('\'');
                    } else if entity == "&nbsp" {
                        result.push(' ');
                    } else {
                        // Keep unknown entities as-is to avoid data loss
                        result.push_str(&entity_buffer);
                        result.push(';');
                    }
                    entity_buffer.clear();
                } else {
                    result.push(c);
                }
            }
            _ => {
                if in_tag {
                    continue;  // Skip all characters inside tags
                } else if in_entity {
                    entity_buffer.push(c);  // Collect entity characters
                } else {
                    result.push(c);  // Keep normal text
                }
            }
        }
    }
    
    // Normalize whitespace similar to how a browser would:
    // - Collapse multiple spaces into one
    // - Trim leading/trailing spaces
    // This ensures consistent display and accurate length measurement
    let mut normalized = String::new();
    let mut last_was_space = true;  // Start true to trim leading whitespace
    
    for c in result.chars() {
        if c.is_whitespace() {
            if !last_was_space {
                normalized.push(' ');
                last_was_space = true;
            }
        } else {
            normalized.push(c);
            last_was_space = false;
        }
    }
    
    // Remove any trailing space that might remain
    if normalized.ends_with(' ') {
        normalized.pop();
    }
    
    normalized
}

/// Helper function to wrap content in details element if it exceeds a length threshold.
/// 
/// This function implements a collapsible content pattern using HTML5 details/summary elements.
/// It's specifically designed to handle long content in table cells by:
/// 1. Measuring the actual visible text length (not HTML markup)
/// 2. Creating a clean, HTML-free summary preview if content is long
/// 3. Preserving the full HTML content in the expanded view
/// 
/// The implementation ensures that:
/// - Short content is displayed directly without any wrapping
/// - Long content gets a readable preview that won't contain broken HTML
/// - Users can easily distinguish between summary and full content
/// - No JavaScript is required for the collapse/expand functionality
fn wrap_in_details_if_needed(content: Markup, threshold: usize) -> Markup {
    let content_str = content.into_string();
    
    // Get plain text version for accurate length measurement
    let plain_text = extract_plain_text(&content_str);
    
    // If content is short enough, return it unchanged
    if plain_text.len() <= threshold {
        return html! { (PreEscaped(content_str)) };
    }
    
    // Create a preview for the summary using plain text to avoid HTML tag issues
    let summary_text = if plain_text.len() <= threshold {
        plain_text.clone()
    } else {
        format!("{}... ", &plain_text[..threshold.min(plain_text.len())].trim())
    };
    
    html! {
        details {
            summary { 
                (summary_text)
                b { "Read more" }
            }
            div {
                (PreEscaped(content_str))
            }
        }
    }
}

/// Whether a parameter is an input or output.
#[derive(Debug, Clone, Copy)]
pub enum InputOutput {
    /// An input parameter.
    Input,
    /// An output parameter.
    Output,
}

/// A parameter (input or output) in a workflow or task.
#[derive(Debug)]
pub struct Parameter {
    /// The declaration of the parameter.
    decl: Decl,
    /// Any meta entries associated with the parameter.
    meta: Option<MetadataValue>,
    /// Whether the parameter is an input or output.
    io: InputOutput,
}

impl Parameter {
    /// Create a new parameter.
    pub fn new(decl: Decl, meta: Option<MetadataValue>, io: InputOutput) -> Self {
        Self { decl, meta, io }
    }

    /// Get the name of the parameter.
    pub fn name(&self) -> String {
        self.decl.name().text().to_owned()
    }

    /// Get the type of the parameter.
    pub fn ty(&self) -> String {
        self.decl.ty().to_string()
    }

    /// Get whether the parameter is an input or output.
    pub fn io(&self) -> InputOutput {
        self.io
    }

    /// Get the Expr value of the parameter as a String.
    pub fn expr(&self) -> String {
        self.decl
            .expr()
            .map(|expr| expr.text().to_string())
            .unwrap_or("None".to_string())
    }

    /// Get whether the input parameter is required.
    ///
    /// Returns `None` for outputs.
    pub fn required(&self) -> Option<bool> {
        match self.io {
            InputOutput::Input => match self.decl.as_unbound_decl() {
                Some(d) => Some(!d.ty().is_optional()),
                _ => Some(false),
            },
            InputOutput::Output => None,
        }
    }

    /// Get the "group" of the parameter.
    pub fn group(&self) -> Option<Group> {
        if let Some(MetadataValue::Object(o)) = &self.meta {
            for item in o.items() {
                if item.name().text() == "group" {
                    if let MetadataValue::String(s) = item.value() {
                        return s.text().map(|t| t.text().to_string()).map(Group);
                    }
                }
            }
        }
        None
    }

    /// Get the description of the parameter.
    pub fn description(&self) -> Markup {
        let markup = if let Some(meta) = &self.meta {
            if let MetadataValue::String(_) = meta {
                render_value(meta)
            } else if let MetadataValue::Object(o) = meta {
                for item in o.items() {
                    if item.name().text() == "description" {
                        if let MetadataValue::String(_) = item.value() {
                            return wrap_in_details_if_needed(render_value(&item.value()), 80);
                        }
                    }
                }
                html! {}
            } else {
                html! {}
            }
        } else {
            html! {}
        };
        
        wrap_in_details_if_needed(markup, 80)
    }

    /// Render the remaining metadata as HTML.
    ///
    /// This will render any metadata that is not rendered elsewhere.
    pub fn render_remaining_meta(&self) -> Markup {
        let markup = if let Some(MetadataValue::Object(o)) = &self.meta {
            let filtered_items = o.items().filter(|item| {
                item.name().text() != "description" && item.name().text() != "group"
            });
            
            html! {
                ul {
                    @for item in filtered_items {
                        li {
                            b { (item.name().text()) ":" } " " (render_value(&item.value()))
                        }
                    }
                }
            }
        } else {
            html! {}
        };
        
        wrap_in_details_if_needed(markup, 80)
    }

    /// Render the parameter as HTML.
    pub fn render(&self) -> Markup {
        if self.required() == Some(true) {
            html! {
                tr {
                    td { (self.name()) }
                    td { code { (self.ty()) } }
                    td { (self.description()) }
                    td { (self.render_remaining_meta()) }
                }
            }
        } else {
            html! {
                tr {
                    td { (self.name()) }
                    td { code { (self.ty()) } }
                    td { code { (self.expr()) } }
                    td { (self.description()) }
                    td { (self.render_remaining_meta()) }
                }
            }
        }
    }
}
