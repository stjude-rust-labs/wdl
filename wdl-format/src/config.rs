//! Formatting configuration.

mod builder;
mod indent;

pub use builder::Builder;
pub use indent::Indent;

/// Configuration for formatting.
#[derive(Clone, Copy, Debug, Default)]
pub struct Config {
    /// The number of characters to indent.
    indent: Indent,
    /// The maximum line length.
    max_line_length: usize,
}

impl Config {
    /// Gets the indent level of the configuration.
    pub fn indent(&self) -> Indent {
        self.indent
    }

    /// Gets the maximum line length of the configuration.
    pub fn max_line_length(&self) -> usize {
        self.max_line_length
    }
}
