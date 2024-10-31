//! Indentation within formatting configuration.

/// The default indentation.
pub const DEFAULT_INDENT: Indent = Indent::Spaces(4);

/// An indentation level.
#[derive(Clone, Copy, Debug)]
pub enum Indent {
    /// Tabs.
    Tabs,

    /// Spaces.
    Spaces(usize),
}

impl Default for Indent {
    fn default() -> Self {
        DEFAULT_INDENT
    }
}

impl Indent {
    /// Gets the number of characters to indent.
    pub fn num(&self) -> usize {
        match self {
            Indent::Tabs => 1,
            Indent::Spaces(n) => *n,
        }
    }
}
