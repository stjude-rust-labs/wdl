//! Lint rules for WDL 1.x abstract syntax trees.

use wdl_core as core;

use crate::v1;

mod matching_parameter_meta;

pub use matching_parameter_meta::MatchingParameterMeta;

/// Gets all WDL v1.x abstract syntax tree lint rules.
pub fn rules<'a>() -> Vec<Box<dyn core::lint::Rule<&'a v1::Document>>> {
    vec![Box::new(MatchingParameterMeta)]
}
