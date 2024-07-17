//! Configuration for formatting.

mod builder;
mod indent;

pub use builder::Builder;
pub use indent::Indent;

/// Configuration for formatting.
#[derive(Debug, Default)]
pub struct Config {
    /// The number of characters to indent.
    indent: Indent,
}

impl Config {
    /// Gets the indent level of the configuration.
    pub fn indent(&self) -> Indent {
        self.indent
    }
}
