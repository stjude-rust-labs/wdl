//! Validation rules for WDL 1.x abstract syntax trees.

use wdl_core as core;

use crate::v1;

/// Gets all WDL v1.x abstract syntax tree validation rules.
pub fn rules<'a>() -> Vec<Box<dyn core::validation::Rule<&'a v1::Document>>> {
    vec![]
}
