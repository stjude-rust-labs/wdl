//! Lint rules for WDL 1.x parse trees.

use pest::iterators::Pair;

mod missing_runtime_block;
mod mixed_indentation;
mod no_curly_commands;
mod snake_case;
mod whitespace;

pub use missing_runtime_block::MissingRuntimeBlock;
pub use mixed_indentation::MixedIndentation;
pub use no_curly_commands::NoCurlyCommands;
pub use snake_case::SnakeCase;
pub use whitespace::Whitespace;

/// Gets all WDL v1.x parse tree lint rules.
pub fn rules<'a>() -> Vec<Box<dyn wdl_core::concern::lint::Rule<&'a Pair<'a, crate::v1::Rule>>>> {
    vec![
        // v1::W001
        Box::new(Whitespace),
        // v1::W002
        Box::new(NoCurlyCommands),
        // v1::W004
        Box::new(MixedIndentation),
        // v1::W005
        Box::new(MissingRuntimeBlock),
        // v1::W006
        Box::new(SnakeCase),
    ]
}
