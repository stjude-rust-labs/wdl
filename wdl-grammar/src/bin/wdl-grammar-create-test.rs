//! A command-line tool to automatically generate tests for WDL syntax.
//!
//! This tool is only intended to be used in the development of the
//! `wdl-grammar` package.
//!
//! This tool is written very sloppily—please keep that in mind.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(missing_debug_implementations)]
#![deny(rustdoc::broken_intra_doc_links)]

use std::fs;
use std::path::Path;
use std::path::PathBuf;

use clap::Parser;
use log::LevelFilter;

use pest::Parser as _;

use pest::iterators::Pair;
use wdl_grammar as wdl;

/// An error related to the `wdl` command-line tool.
#[derive(Debug)]
pub enum Error {
    /// An input/output error.
    IoError(std::io::Error),

    /// Attempted to access a file, but it was missing.
    FileDoesNotExist(PathBuf),

    /// Not able to match the provided rule name to a defined rule.
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

/// A command-line tool to automatically generate tests for WDL syntax.
#[derive(Debug, Parser)]
pub struct Args {
    /// The path to the document.
    path: PathBuf,

    /// The rule to evaluate.
    #[arg(short = 'r', long, default_value = "document")]
    rule: String,
}

fn inner() -> Result<()> {
    let args = Args::parse();

    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();

    let (contents, rule) = parse_from_path(&args.rule, &args.path)?;
    let parse_tree: pest::iterators::Pairs<'_, wdl::Rule> =
        wdl::Parser::parse(rule, &contents).map_err(|err| Error::PestError(Box::new(err)))?;

    for pair in parse_tree {
        print_create_test_recursive(pair, 0);
    }

    Ok(())
}

fn print_create_test_recursive(pair: Pair<'_, wdl::Rule>, indent: usize) {
    let span = pair.as_span();
    let comment = pair
        .as_str()
        .lines()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    if !comment.is_empty() {
        println!("{}// `{}`", " ".repeat(indent), comment);
    }
    print!(
        "{}{:?}({}, {}",
        " ".repeat(indent),
        pair.as_rule(),
        span.start(),
        span.end()
    );

    let inner = pair.into_inner();

    if inner.peek().is_some() {
        println!(", [");

        for pair in inner {
            print_create_test_recursive(pair, indent + 2);
            println!(",");
        }

        print!("{}]", " ".repeat(indent));
    }

    print!(")");
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
        "if" => Some(wdl::Rule::r#if),
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
        "workflow_scatter" => Some(wdl::Rule::workflow_scatter),
        "workflow_call" => Some(wdl::Rule::workflow_call),
        "workflow_conditional" => Some(wdl::Rule::workflow_conditional),
        "postfix" => Some(wdl::Rule::postfix),
        _ => todo!("must implement mapping for rule: {rule}"),
    }
}

fn main() {
    match inner() {
        Ok(_) => {}
        Err(err) => eprintln!("{}", err),
    }
}
