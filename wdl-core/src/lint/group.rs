//! Lint groups.

/// A lint group.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Group {
    /// Rules associated with having a complete document.
    Completeness,

    /// Rules associated with the style of a document.
    Style,

    /// Rules often considered overly opinionated.
    ///
    /// These rules are disabled by default but can be turned on individually.
    Pedantic,
}

impl std::fmt::Display for Group {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Group::Completeness => write!(f, "Completeness"),
            Group::Style => write!(f, "Style"),
            Group::Pedantic => write!(f, "Pedantic"),
        }
    }
}
