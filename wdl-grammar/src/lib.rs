//! A crate providing facilities for parsing the Workflow Description Language
//! (WDL) using [`pest`](https://pest.rs). federation API server along with the

#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(missing_debug_implementations)]
#![deny(rustdoc::broken_intra_doc_links)]

mod grammar;

use pest_derive::Parser;

#[derive(Debug, Parser)]
#[grammar = "grammar/wdl.pest"]
pub struct Parser;
