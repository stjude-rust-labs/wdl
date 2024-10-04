//! Indentation within formatting configuration.

use std::num::NonZeroUsize;
use std::sync::LazyLock;

/// The default indentation.
pub static DEFAULT_INDENT: LazyLock<Indent> =
    LazyLock::new(|| Indent::Spaces(NonZeroUsize::new(4).unwrap()));

/// An indentation level.
#[derive(Clone, Copy, Debug)]
pub enum Indent {
    /// Tabs.
    Tabs(NonZeroUsize),

    /// Spaces.
    Spaces(NonZeroUsize),
}

impl Default for Indent {
    fn default() -> Self {
        *DEFAULT_INDENT
    }
}
