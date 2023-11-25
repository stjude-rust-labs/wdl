//! Lint rules for WDL 1.x parse trees.

use pest::iterators::Pair;

use wdl_core as core;

mod no_curly_commands;
mod whitespace;

pub use no_curly_commands::NoCurlyCommands;
pub use whitespace::Whitespace;

/// Gets all WDL v1.x parse tree lint rules.
pub fn rules<'a>() -> Vec<Box<dyn core::lint::Rule<&'a Pair<'a, crate::v1::Rule>>>> {
    vec![Box::new(Whitespace), Box::new(NoCurlyCommands)]
}
