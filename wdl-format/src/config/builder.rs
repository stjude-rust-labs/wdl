//! Builders for formatting configuration.

use crate::Config;
use crate::config::Indent;
use crate::config::MaxLineLength;
use crate::config::indent::MAX_SPACE_INDENT;

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
pub struct Builder {
    /// The number of characters to indent.
    indent: Option<Indent>,
    /// The maximum line length.
    max_line_length: Option<MaxLineLength>,
}

impl Builder {
    /// Creates a new builder with default values.
    pub fn new(indent: Option<Indent>, max_line_length: Option<MaxLineLength>) -> Result<Self> {
        let indent = indent.unwrap_or_default();
        let max_line_length = max_line_length.unwrap_or_default();
        Ok(Self::default()
            .indent(indent)?
            .max_line_length(max_line_length))
    }

    /// Sets the indentation level.
    ///
    /// # Notes
    ///
    /// This silently overwrites any previously provided value for the
    /// indentation level.
    pub fn indent(mut self, indent: Indent) -> Result<Self> {
        if let Indent::Spaces(n) = indent {
            if n > MAX_SPACE_INDENT {
                return Err(Error::Invalid("indent"));
            }
        }
        self.indent = Some(indent);
        Ok(self)
    }

    /// Sets the maximum line length.
    ///
    /// # Notes
    ///
    /// This silently overwrites any previously provided value for the maximum
    /// line length.
    pub fn max_line_length(mut self, max_line_length: MaxLineLength) -> Self {
        self.max_line_length = Some(max_line_length);
        self
    }

    /// Consumes `self` and attempts to build a [`Config`].
    pub fn try_build(self) -> Result<Config> {
        let indent = self.indent.ok_or(Error::Missing("indent"))?;
        let max_line_length = self
            .max_line_length
            .ok_or(Error::Missing("max_line_length"))?;

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
            max_line_length: Some(Default::default()),
        }
    }
}
