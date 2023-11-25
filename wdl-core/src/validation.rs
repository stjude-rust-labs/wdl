//! Validation.

use to_snake_case::ToSnakeCase as _;

pub mod error;

pub use error::Error;

use crate::Code;

/// A [`Result`](std::result::Result) with a validation [`Error`].
pub type Result = std::result::Result<(), Error>;

/// A parse tree validator.
#[derive(Debug)]
pub struct Validator;

impl Validator {
    /// Validates a tree according to a set of validation rules.
    pub fn validate<'a, E>(tree: &'a E, rules: Vec<Box<dyn Rule<&'a E>>>) -> Result {
        rules.iter().try_for_each(|rule| rule.validate(tree))
    }
}

/// A validation rule.
pub trait Rule<E>: std::fmt::Debug + Sync {
    /// The name of the validation rule.
    ///
    /// This is what will show up in style guides, it is required to be snake
    /// case (even though the rust struct is camel case).
    fn name(&self) -> String {
        format!("{:?}", self).to_snake_case()
    }

    /// Get the code for this validation rule.
    fn code(&self) -> Code;

    /// Checks the tree according to the implemented validation rule.
    fn validate(&self, tree: E) -> Result;
}
