//! Workflow Description Language (WDL) utilities.
//!
//! This crate is a convenience package that reëxports several WDL crates as
//! modules. For detailed information, see the module-level documentation.

#![warn(missing_docs)]

#[cfg(feature = "analysis")]
#[doc(inline)]
pub use wdl_analysis as analysis;
#[cfg(feature = "ast")]
#[doc(inline)]
pub use wdl_ast as ast;
#[cfg(feature = "cli")]
#[doc(inline)]
pub use wdl_cli as cli;
#[cfg(feature = "doc")]
#[doc(inline)]
pub use wdl_doc as doc;
#[cfg(feature = "engine")]
#[doc(inline)]
pub use wdl_engine as engine;
#[cfg(feature = "format")]
#[doc(inline)]
pub use wdl_format as format;
#[cfg(feature = "lint")]
#[doc(inline)]
pub use wdl_lint as lint;
#[cfg(feature = "lsp")]
#[doc(inline)]
pub use wdl_lsp as lsp;
