//! Builders for formatting configuration.

use crate::Config;
use crate::config::Indent;

/// The default maximum line length.
pub const DEFAULT_MAX_LINE_LENGTH: usize = 90;

/// An error related to a [`Builder`].
#[derive(Debug)]
pub enum Error {
    /// A required value was missing for a builder field.
    Missing(&'static str),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Missing(field) => write!(
                f,
                "missing required value for '{field}' in a formatter configuration builder"
            ),
        }
    }
}

impl std::error::Error for Error {}

/// A [`Result`](std::result::Result) with an [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// A builder for a [`Config`].
pub struct Builder {
    /// The number of characters to indent.
    indent: Option<Indent>,
    /// The maximum line length.
    max_line_length: Option<usize>,
}

impl Builder {
    /// Creates a new builder with default values.
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the indentation level.
    ///
    /// # Notes
    ///
    /// This silently overwrites any previously provided value for the
    /// indentation level.
    pub fn indent(mut self, indent: Indent) -> Self {
        self.indent = Some(indent);
        self
    }

    /// Consumes `self` and attempts to build a [`Config`].
    pub fn try_build(self) -> Result<Config> {
        let indent = self.indent.ok_or(Error::Missing("indent"))?;
        let max_line_length = self.max_line_length.unwrap_or(DEFAULT_MAX_LINE_LENGTH);

        Ok(Config {
            indent,
            max_line_length,
        })
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            indent: Some(Default::default()),
            max_line_length: Some(90),
        }
    }
}
