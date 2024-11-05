//! Library for generating HTML documentation from WDL files.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(missing_debug_implementations)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(rustdoc::broken_intra_doc_links)]

pub mod parameter;
pub mod r#struct;

use std::fmt::Display;

use html::content;
use html::text_content;
use wdl_ast::AstToken;
use wdl_ast::Version;

/// A WDL document.
#[derive(Debug)]
pub struct Document {
    /// The name of the document.
    ///
    /// This is the filename of the document without the extension.
    name: String,
    /// The version of the document.
    version: Version,
    /// The structs in the document.
    structs: Vec<r#struct::Struct>,
}

impl Document {
    /// Create a new document.
    pub fn new(name: String, version: Version, structs: Vec<r#struct::Struct>) -> Self {
        Self {
            name,
            version,
            structs,
        }
    }

    /// Get the name of the document.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the version of the document.
    pub fn version(&self) -> &Version {
        &self.version
    }

    /// Get the structs in the document.
    pub fn structs(&self) -> &[r#struct::Struct] {
        &self.structs
    }
}

impl Display for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let document_name = content::Heading1::builder()
            .text(self.name().to_owned())
            .build();
        let version = text_content::Paragraph::builder()
            .text(format!("Version: {}", self.version().as_str()))
            .build();

        let mut structs = text_content::UnorderedList::builder();
        for r#struct in self.structs() {
            structs.push(
                text_content::ListItem::builder()
                    .text(r#struct.to_string())
                    .build(),
            );
        }
        let structs = structs.build();

        write!(f, "{}", document_name)?;
        write!(f, "{}", version)?;
        write!(f, "{}", structs)
    }
}
