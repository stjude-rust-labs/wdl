//! Lexing and parsing for Workflow Description Language (WDL) documents.
//!
//! This module implements an infallible WDL parser based on the `logos` crate
//! for lexing and the `rowan` crate for concrete syntax tree (CST)
//! representation.
//!
//! Mutations to the tree CST are non-destructive, and involve creating a new
//! tree where unaffected nodes are shared between the old and new trees. The
//! cost of editing a node of the tree depends solely on the depth of the node,
//! as it must update the parent chain to produce a new tree root.
//!
//! The main entrypoint to the parser is [`SyntaxTree::parse()`] which returns a
//! [`ParseResult`].
//!
//! # Examples
//!
//! An example of parsing WDL source into a CST and printing the tree:
//!
//! ```rust
//! use wdl_ast::concrete::ParseResult;
//! use wdl_ast::concrete::SyntaxTree;
//!
//! let ParseResult { tree, diagnostics } = SyntaxTree::parse("version 1.1");
//! assert!(diagnostics.is_empty());
//! println!("{tree:#?}");
//! ```

pub mod grammar;
pub mod lexer;
pub mod parser;
mod tree;

pub use tree::*;
