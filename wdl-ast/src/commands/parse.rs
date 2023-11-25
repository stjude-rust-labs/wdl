//! `wdl-ast parse`

use core::display;
use std::path::PathBuf;

use clap::Parser;

use log::warn;
use wdl_ast as ast;
use wdl_core as core;
use wdl_grammar as grammar;

/// An error related to the `wdl-ast parse` subcommand.
#[derive(Debug)]
pub enum Error {
    /// An abstract syntax tree error.
    Ast(ast::Error),

    /// An input/output error.
    InputOutput(std::io::Error),

    /// A grammar error.
    Grammar(grammar::Error<grammar::v1::Rule>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Ast(err) => write!(f, "ast error: {err}"),
            Error::InputOutput(err) => write!(f, "i/o error: {err}"),
            Error::Grammar(err) => write!(f, "grammar error: {err}"),
        }
    }
}

impl std::error::Error for Error {}

/// A [`Result`](std::result::Result) with an [`Error`].
type Result<T> = std::result::Result<T, Error>;

/// Arguments for the `wdl-ast parse` subcommand.
#[derive(Debug, Parser)]
pub struct Args {
    /// Path to the WDL document.
    #[clap(value_name = "PATH")]
    path: PathBuf,

    /// The Workflow Description Language (WDL) specification version to use.
    #[arg(value_name = "VERSION", short = 's', long, default_value_t, value_enum)]
    specification_version: core::Version,
}

/// Main function for this subcommand.
pub fn parse(args: Args) -> Result<()> {
    let contents = std::fs::read_to_string(args.path).map_err(Error::InputOutput)?;

    let document = match args.specification_version {
        core::Version::V1 => {
            let pt = grammar::v1::parse(&contents).map_err(Error::Grammar)?;

            ast::v1::parse(pt).map_err(Error::Ast)?
        }
    };

    if let Some(warnings) = document.warnings() {
        for warning in warnings {
            let mut buffer = String::new();
            warning
                .display(&mut buffer, display::Mode::OneLine)
                .unwrap();
            warn!("{}", buffer);
        }
    }

    dbg!(document);

    Ok(())
}
