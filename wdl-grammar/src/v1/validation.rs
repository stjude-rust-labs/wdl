//! Validation rules for WDL 1.x parse trees.

use pest::iterators::Pair;

use wdl_core as core;

mod invalid_escape_character;

pub use invalid_escape_character::InvalidEscapeCharacter;

/// Gets all WDL v1.x parse tree validation rules.
pub fn rules<'a>() -> Vec<Box<dyn core::validation::Rule<&'a Pair<'a, crate::v1::Rule>>>> {
    vec![Box::new(InvalidEscapeCharacter)]
}
