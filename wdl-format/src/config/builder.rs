//! Builders for formatting configuration.

use crate::Config;
use crate::config::Indent;
use crate::config::MaxLineLength;

/// An error related to a [`Builder`].
#[derive(Debug)]
pub enum Error {
    /// A required value was missing for a builder field.
    Missing(&'static str),

    /// An invalid value was provided for a builder field.
    Invalid(&'static str),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Missing(field) => write!(
                f,
                "missing required value for '{field}' in a formatter configuration builder"
            ),
            Error::Invalid(field) => write!(
                f,
                "invalid value for '{field}' in a formatter configuration builder"
            ),
        }
    }
}

impl std::error::Error for Error {}

/// A [`Result`](std::result::Result) with an [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// A builder for a [`Config`].
#[derive(Default)]
pub struct Builder {
    /// The number of characters to indent.
    indent: Option<Indent>,
    /// The maximum line length.
    max_line_length: Option<MaxLineLength>,
}

impl Builder {
    /// Sets the indentation level.
    ///
    /// This silently overwrites any previously provided value for the
    /// indentation level.
    pub fn indent(mut self, indent: Indent) -> Self {
        self.indent = Some(indent);
        self
    }

    /// Sets the maximum line length.
    ///
    /// This silently overwrites any previously provided value for the maximum
    /// line length.
    pub fn max_line_length(mut self, max_line_length: MaxLineLength) -> Self {
        self.max_line_length = Some(max_line_length);
        self
    }

    /// Consumes `self` to build a [`Config`].
    pub fn build(self) -> Config {
        let indent = self.indent.unwrap_or_default();
        let max_line_length = self.max_line_length.unwrap_or_default();
        Config {
            indent,
            max_line_length,
        }
    }
}
