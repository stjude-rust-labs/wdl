//! Configuration for max line length formatting.

/// The error type for max line length configuration.
pub enum Error {
    /// The max line length is invalid.
    Invalid(&'static str),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Invalid(field) => write!(
                f,
                "invalid value for '{field}' in a formatter configuration builder"
            ),
        }
    }
}

/// The default maximum line length.
pub const DEFAULT_MAX_LINE_LENGTH: usize = 90;

/// The maximum line length.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MaxLineLength(Option<usize>);

impl MaxLineLength {
    /// Creates a new `MaxLineLength` with the provided value.
    pub fn with_value(value: usize) -> Result<Self, Error> {
        let val = match value {
            0 => Self(None),
            60..=240 => Self(Some(value)),
            _ => return Err(Error::Invalid("max_line_length")),
        };
        Ok(val)
    }

    /// Gets the maximum line length.
    pub fn get(&self) -> Option<usize> {
        self.0
    }
}

impl Default for MaxLineLength {
    fn default() -> Self {
        Self(Some(DEFAULT_MAX_LINE_LENGTH))
    }
}
