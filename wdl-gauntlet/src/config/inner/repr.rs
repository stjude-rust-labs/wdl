//! Representation of ignored errors as stored in the configuration file.

// This compiler attribute is added because `serde_with` generates a struct
// below that does not have any documentation. The only way to silence the
// warning is to allow missing docs for this file.
#![allow(missing_docs)]

use serde::Deserialize;
use serde::Serialize;

use crate::config::inner::Errors;
use crate::document;

/// A representation of an error to ignore as serialized in the configuration
/// file.
///
/// In short, I wanted to convert the [`Errors`] object to something more
/// visually understandable in the configuration file. Thus, the only purpose of
/// this struct is to serialize and deserialize entries in that
/// [`HashMap`](std::collections::HashMap) in a prettier way.
#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Error {
    /// The document identifier converted to a [`String`].
    pub document: String,

    /// The error converted to a [`String`].
    pub error: String,
}

serde_with::serde_conv!(
    pub ErrorsAsReprs,
    Errors,
    |errors: &Errors| {
        let mut result = errors
            .iter()
            .map(|(document, error)| Error {
                document: document.to_string(),
                error: error.clone(),
            })
            .collect::<Vec<_>>();
        result.sort();
        result
    },
    |errors: Vec<Error>| -> Result<_, document::identifier::Error> {
        errors
            .into_iter()
            .map(|repr| {
                let identifier = repr.document.parse::<document::Identifier>()?;
                Ok((identifier, repr.error))
            })
            .collect::<Result<Errors, _>>()
    }
);
