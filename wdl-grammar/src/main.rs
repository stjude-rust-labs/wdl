//! A command-line tool for parsing and exploring Workflow Description Language
//! (WDL) documents.
//!
//! This tool is intended to be used as a utility to test and develop the
//! [`wdl`](https://crates.io/wdl) crate.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(missing_debug_implementations)]
#![deny(rustdoc::broken_intra_doc_links)]

use std::fs;
use std::path::Path;
use std::path::PathBuf;

use clap::Parser;
use clap::Subcommand;
use log::LevelFilter;

use pest::Parser as _;

use wdl_grammar as wdl;

/// An error related to the `wdl` command-line tool.
#[derive(Debug)]
pub enum Error {
    /// An input/output error.
    IoError(std::io::Error),

    /// Attempted to access a file, but it was missing.
    FileDoesNotExist(PathBuf),

    /// Not able to match the file stem to a rule.
    RuleMismatch(PathBuf),

    /// An error from Pest.
    PestError(Box<pest::error::Error<wdl::Rule>>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IoError(err) => write!(f, "i/o error: {err}"),
            Error::FileDoesNotExist(path) => write!(f, "file does not exist: {}", path.display()),
            Error::RuleMismatch(path) => {
                write!(f, "cannot match rule from file: {}", path.display())
            }
            Error::PestError(err) => write!(f, "pest error:\n{err}"),
        }
    }
}

impl std::error::Error for Error {}

type Result<T> = std::result::Result<T, Error>;

/// Arguments for the `parse` subcommand.
#[derive(Debug, Parser)]
pub struct ParseArgs {
    /// The path to the document.
    path: PathBuf,

    /// The rule to evaluate.
    #[arg(short = 'r', long, default_value = "document")]
    rule: String,
}

/// Subcommands for the `wdl` command-line tool.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Parses the Workflow Description Language document and prints the parse
    /// tree.
    Parse(ParseArgs),
}

/// Parse and describe Workflow Description Language documents.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
struct Args {
    /// The subcommand to execute.
    #[command(subcommand)]
    command: Command,
}

fn inner() -> Result<()> {
    let args = Args::parse();

    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();

    match args.command {
        Command::Parse(args) => {
            let (contents, rule) = parse_from_path(&args.rule, &args.path)?;
            let mut parse_tree = wdl::Parser::parse(rule, &contents)
                .map_err(|err| Error::PestError(Box::new(err)))?;

            // For documents, we don't care about the parent element: it is much
            // more informative to see the children of the document split by
            // spaces. This is a stylistic choice.
            match rule {
                wdl::Rule::document => {
                    for element in parse_tree.next().unwrap().into_inner() {
                        dbg!(element);
                    }
                }
                _ => {
                    dbg!(parse_tree);
                }
            };
        }
    }

    Ok(())
}

fn parse_from_path(rule: impl AsRef<str>, path: impl AsRef<Path>) -> Result<(String, wdl::Rule)> {
    let rule = rule.as_ref();
    let path = path.as_ref();

    let rule = map_rule(rule)
        .map(Ok)
        .unwrap_or_else(|| Err(Error::RuleMismatch(path.to_path_buf())))?;

    let contents = fs::read_to_string(path).map_err(Error::IoError)?;
Ok((contents, rule))
}

fn map_rule(rule: &str) -> Option<wdl::Rule> {
    match rule {
        "document" => Some(wdl::Rule::document),
        "task" => Some(wdl::Rule::task),
        "core" => Some(wdl::Rule::core),
        "expression" => Some(wdl::Rule::expression),
        "object_literal" => Some(wdl::Rule::object_literal),
        "task_metadata_object" => Some(wdl::Rule::task_metadata_object),
        "task_parameter_metadata" => Some(wdl::Rule::task_parameter_metadata),
        "workflow_metadata_kv" => Some(wdl::Rule::workflow_metadata_kv),
        "command_heredoc_interpolated_contents" => {
            Some(wdl::Rule::command_heredoc_interpolated_contents)
        }
        _ => todo!("must implement mapping for rule: {rule}"),
    }
}

fn main() {
    match inner() {
        Ok(_) => {}
        Err(err) => eprintln!("{}", err),
    }
}
