//! Linting.

use to_snake_case::ToSnakeCase as _;

mod group;
mod level;
pub mod warning;

pub use group::Group;
pub use level::Level;
pub use warning::Warning;

use crate::Code;

/// A [`Result`](std::result::Result) returned from a lint check.
pub type Result = std::result::Result<Option<Vec<Warning>>, Box<dyn std::error::Error>>;

/// A tree linter.
#[derive(Debug)]
pub struct Linter;

impl Linter {
    /// Lints a tree according to a set of lint rules and returns a
    /// set of lint warnings (if any are detected).
    pub fn lint<'a, E>(tree: &'a E, rules: Vec<Box<dyn Rule<&'a E>>>) -> Result {
        let warnings = rules
            .iter()
            .map(|rule| rule.check(tree))
            .collect::<std::result::Result<Vec<Option<Vec<Warning>>>, Box<dyn std::error::Error>>>(
            )?
            .into_iter()
            .flatten()
            .flatten()
            .collect::<Vec<Warning>>();

        match warnings.is_empty() {
            true => Ok(None),
            false => Ok(Some(warnings)),
        }
    }
}

/// A lint rule.
pub trait Rule<E>: std::fmt::Debug + Sync {
    /// The name of the lint rule.
    fn name(&self) -> String {
        format!("{:?}", self).to_snake_case()
    }

    /// Get the code for this lint rule.
    fn code(&self) -> Code;

    /// Get the lint group for this lint rule.
    fn group(&self) -> Group;

    /// Checks the tree according to the implemented lint rule.
    fn check(&self, tree: E) -> Result;
}
