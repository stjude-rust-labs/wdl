//! Common functionality used across the `wdl` family of crates.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(missing_debug_implementations)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(rustdoc::broken_intra_doc_links)]

mod code;
pub mod display;
pub mod lint;
pub mod location;
pub mod validation;
mod version;

pub use code::Code;
pub use location::Location;
pub use version::Version;
