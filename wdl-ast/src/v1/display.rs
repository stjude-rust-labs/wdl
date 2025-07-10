//! Provides rich documentation for WDL definitions used by LSP hovers and
//! completions.
//!
//! TODO: This functionality can be shared with `wdl-doc` ? It might make
//! sense to fold this into wdl-analysis maybe?

use std::fmt::{self};

use crate::AstNode;
use crate::AstToken;
use crate::v1::InputSection;
use crate::v1::MetadataArray;
use crate::v1::MetadataObjectItem;
use crate::v1::MetadataValue;
use crate::v1::OutputSection;
use crate::v1::ParameterMetadataSection;

/// Formats a metadata value.
pub fn format_meta_value(
    f: &mut fmt::Formatter<'_>,
    value: &MetadataValue,
    indent: usize,
) -> fmt::Result {
    let prefix = " ".repeat(indent);
    match value {
        MetadataValue::Boolean(b) => writeln!(f, "{prefix}- `{}`", b.value()),
        MetadataValue::Integer(i) => writeln!(f, "{prefix}- `{}`", i.value().unwrap_or(0)),
        MetadataValue::Float(fl) => {
            writeln!(f, "{prefix}- `{}`", fl.value().unwrap_or(0.0))
        }
        MetadataValue::String(s) => {
            if let Some(text) = s.text() {
                writeln!(f, "{prefix}- `{}`", text.text())?
            }
            Ok(())
        }
        MetadataValue::Null(_) => writeln!(f, "{prefix}- `null`"),
        MetadataValue::Object(obj) => write_meta_object(f, obj.items(), indent),
        MetadataValue::Array(arr) => write_meta_array(f, arr, indent),
    }
}

/// Formats a metadata object.
pub fn write_meta_object<Items: Iterator<Item = MetadataObjectItem>>(
    f: &mut fmt::Formatter<'_>,
    items: Items,
    indent: usize,
) -> fmt::Result {
    let prefix = " ".repeat(indent);
    for item in items {
        write!(f, "{prefix}- **{}**", item.name().text())?;
        format_meta_value(f, &item.value(), indent + 2)?;
    }
    Ok(())
}

/// Formats a metadata array.
fn write_meta_array(f: &mut fmt::Formatter<'_>, arr: &MetadataArray, indent: usize) -> fmt::Result {
    for value in arr.elements() {
        format_meta_value(f, &value, indent)?;
    }
    Ok(())
}

/// Gets the entire metadata value for a given parameter name.
pub fn get_param_meta(
    name: &str,
    param_meta: Option<&ParameterMetadataSection>,
) -> Option<MetadataValue> {
    param_meta
        .and_then(|pm| pm.items().find(|i| i.name().text() == name))
        .map(|item| item.value())
}

/// Formats the input section with parameter metadata.
pub fn write_input_section(
    f: &mut fmt::Formatter<'_>,
    input: Option<&InputSection>,
    param_meta: Option<&ParameterMetadataSection>,
) -> fmt::Result {
    if let Some(input) = input {
        if input.declarations().next().is_some() {
            writeln!(f, "\n**Inputs**")?;
            for decl in input.declarations() {
                let name = decl.name();
                let default = decl.expr().map(|e| e.text().to_string());

                write!(f, "- **{}**: `{}`", name.text(), decl.ty())?;
                if let Some(val) = default {
                    // default values
                    write!(f, " = *`{}`*", val.trim_start_matches(" = "))?;
                }

                if let Some(meta_val) = get_param_meta(name.text(), param_meta) {
                    writeln!(f)?;
                    format_meta_value(f, &meta_val, 2)?;
                    writeln!(f)?;
                } else {
                    writeln!(f)?;
                }
            }
        }
    }
    Ok(())
}

/// Formats the output section with parameter metadata.
pub fn write_output_section(
    f: &mut fmt::Formatter<'_>,
    output: Option<&OutputSection>,
    param_meta: Option<&ParameterMetadataSection>,
) -> fmt::Result {
    if let Some(output) = output {
        if output.declarations().next().is_some() {
            writeln!(f, "\n**Outputs**")?;
            for decl in output.declarations() {
                let name = decl.name();
                write!(f, "- **{}**: `{}`", name.text(), decl.ty())?;
                if let Some(meta_val) = get_param_meta(name.text(), param_meta) {
                    writeln!(f)?;
                    format_meta_value(f, &meta_val, 2)?;
                    writeln!(f)?;
                } else {
                    writeln!(f)?;
                }
            }
        }
    }
    Ok(())
}
