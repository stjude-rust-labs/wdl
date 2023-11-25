//! A parse tree.

use pest::iterators::Pair;
use pest::RuleType;

use wdl_core as core;

use core::lint;

/// A parse tree with a set of lint [`Warning`](lint::Warning)s.
///
/// **Note:** this struct implements [`std::ops::Deref`] for the native Pest
/// parse tree ([`Pair`]), so you can treat this exactly as if you were
/// workings with [`Pair`] directly.
#[derive(Debug)]
pub struct Tree<'a, R: RuleType> {
    /// The inner Pest parse tree.
    inner: Pair<'a, R>,

    /// The lint warnings associated with the parse tree.
    warnings: Option<Vec<lint::Warning>>,
}

impl<'a, R: RuleType> Tree<'a, R> {
    /// Creates a new [`Tree`].
    pub fn new(inner: Pair<'a, R>, warnings: Option<Vec<lint::Warning>>) -> Self {
        Self { inner, warnings }
    }

    /// Gets the inner [Pest parse tree](Pair) for the [`Tree`] by reference.
    pub fn inner(&self) -> &Pair<'a, R> {
        &self.inner
    }

    /// Consumes `self` to return the inner [Pest parse tree](Pair) from the
    /// [`Tree`].
    pub fn into_inner(self) -> Pair<'a, R> {
        self.inner
    }

    /// Gets the [`Warning`](lint::Warning)s from the [`Tree`] by reference.
    pub fn warnings(&self) -> Option<&Vec<lint::Warning>> {
        self.warnings.as_ref()
    }
}

impl<'a, R: RuleType> std::ops::Deref for Tree<'a, R> {
    type Target = Pair<'a, R>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use wdl_core::lint::Linter;

    use pest::Parser as _;

    use super::*;
    use crate::v1::Parser;
    use crate::v1::Rule;

    #[test]
    fn new() -> Result<(), Box<dyn std::error::Error>> {
        let tree = Parser::parse(Rule::document, "version 1.1\n \n")?
            .next()
            .unwrap();
        let warnings = Linter::lint(&tree, crate::v1::lint::rules())?;

        let tree = Tree::new(tree, warnings);
        assert_eq!(
            tree.warnings().unwrap().first().unwrap().to_string(),
            String::from("[v1::001::Style/Low] line contains only whitespace (2:1-2:1)")
        );

        let mut items = tree.into_inner().into_inner();
        assert_eq!(items.len(), 5);
        assert_eq!(items.next().unwrap().as_str(), "version 1.1");
        assert_eq!(items.next().unwrap().as_str(), "\n");
        assert_eq!(items.next().unwrap().as_str(), " ");
        assert_eq!(items.next().unwrap().as_str(), "\n");
        assert_eq!(items.next().unwrap().as_str(), "");

        Ok(())
    }
}
