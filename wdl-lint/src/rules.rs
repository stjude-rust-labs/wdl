//! Module for the lint rules.

mod call_input_spacing;
mod command_mixed_indentation;
mod double_quotes;
mod ending_newline;
mod import_placement;
mod import_sort;
mod import_whitespace;
mod inconsistent_newlines;
mod input_not_sorted;
mod line_width;
mod matching_parameter_meta;
mod missing_metas;
mod missing_output;
mod missing_runtime;
mod no_curly_commands;
mod nonmatching_output;
mod pascal_case;
mod preamble_comments;
mod preamble_whitespace;
mod snake_case;
mod whitespace;

pub use call_input_spacing::*;
pub use command_mixed_indentation::*;
pub use double_quotes::*;
pub use ending_newline::*;
pub use import_placement::*;
pub use import_sort::*;
pub use import_whitespace::*;
pub use inconsistent_newlines::*;
pub use input_not_sorted::*;
pub use line_width::*;
pub use matching_parameter_meta::*;
pub use missing_metas::*;
pub use missing_output::*;
pub use missing_runtime::*;
pub use no_curly_commands::*;
pub use nonmatching_output::*;
pub use pascal_case::*;
pub use preamble_comments::*;
pub use preamble_whitespace::*;
pub use snake_case::*;
pub use whitespace::*;
