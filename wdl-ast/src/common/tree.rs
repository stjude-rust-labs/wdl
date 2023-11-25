//! An abstract syntax tree.

use wdl_core as core;

use core::lint;

use crate::v1::Document;

/// An abstract syntax tree with a set of lint [`Warning`](lint::Warning)s.
///
/// **Note:** this struct implements [`std::ops::Deref`] for a parsed WDL
/// [`Document`], so you can treat this exactly as if you were workings with a
/// [`Document`] directly.
#[derive(Debug)]
pub struct Tree {
    /// The inner document.
    inner: Document,

    /// The lint warnings associated with the parse tree.
    warnings: Option<Vec<lint::Warning>>,
}

impl Tree {
    /// Creates a new [`Tree`].
    pub fn new(inner: Document, warnings: Option<Vec<lint::Warning>>) -> Self {
        Self { inner, warnings }
    }

    /// Gets the inner [`Document`] for the [`Tree`] by reference.
    pub fn inner(&self) -> &Document {
        &self.inner
    }

    /// Consumes `self` to return the inner [`Document`] from the [`Tree`].
    pub fn into_inner(self) -> Document {
        self.inner
    }

    /// Gets the [`Warning`](lint::Warning)s from the [`Tree`] by reference.
    pub fn warnings(&self) -> Option<&Vec<lint::Warning>> {
        self.warnings.as_ref()
    }
}

impl std::ops::Deref for Tree {
    type Target = Document;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
