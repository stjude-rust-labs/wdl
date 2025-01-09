//! The `wdl` command line tool.
//!
//! If you're here and not a developer of the `wdl` family of crates, you're
//! probably looking for
//! [Sprocket](https://github.com/stjude-rust-labs/sprocket) instead.
use std::fs;
use std::io::IsTerminal;
use std::io::Read;
use std::io::stderr;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use clap::Args;
use clap::Parser;
use clap::Subcommand;
use clap_verbosity_flag::Verbosity;
use codespan_reporting::files::SimpleFile;
use codespan_reporting::term::Config;
use codespan_reporting::term::emit;
use codespan_reporting::term::termcolor::ColorChoice;
use codespan_reporting::term::termcolor::StandardStream;
use colored::Colorize;
use tracing_log::AsTrace;
use wdl::ast::Diagnostic;
use wdl::ast::Document;
use wdl::ast::Validator;
use wdl::cli::analyze;
use wdl::cli::run;
use wdl::cli::validate_inputs;
use wdl::lint::LintVisitor;
use wdl_ast::Node;
use wdl_ast::Severity;
use wdl_doc::document_workspace;
use wdl_format::Formatter;
use wdl_format::element::node::AstNodeFormatExt as _;
use wdl_lint::rules::ShellCheckRule;

/// Emits the given diagnostics to the output stream.
///
/// The use of color is determined by the presence of a terminal.
///
/// In the future, we might want the color choice to be a CLI argument.
fn emit_diagnostics(path: &str, source: &str, diagnostics: &[Diagnostic]) -> Result<usize> {
    let file = SimpleFile::new(path, source);
    let mut stream = StandardStream::stdout(if std::io::stdout().is_terminal() {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
    });

    let mut errors = 0;
    for diagnostic in diagnostics.iter() {
        if diagnostic.severity() == Severity::Error {
            errors += 1;
        }

        emit(
            &mut stream,
            &Config::default(),
            &file,
            &diagnostic.to_codespan(),
        )
        .context("failed to emit diagnostic")?;
    }

    Ok(errors)
}

/// Reads source from the given path.
///
/// If the path is simply `-`, the source is read from STDIN.
fn read_source(path: &Path) -> Result<String> {
    if path.as_os_str() == "-" {
        let mut source = String::new();
        std::io::stdin()
            .read_to_string(&mut source)
            .context("failed to read source from stdin")?;
        Ok(source)
    } else {
        Ok(fs::read_to_string(path).with_context(|| {
            format!("failed to read source file `{path}`", path = path.display())
        })?)
    }
}

/// Parses a WDL source file and prints the syntax tree.
#[derive(Args)]
#[clap(disable_version_flag = true)]
pub struct ParseCommand {
    /// The path to the source WDL file.
    #[clap(value_name = "PATH")]
    pub path: PathBuf,
}

impl ParseCommand {
    /// Executes the `parse` subcommand.
    async fn exec(self) -> Result<()> {
        let source = read_source(&self.path)?;
        let (document, diagnostics) = Document::parse(&source);
        if !diagnostics.is_empty() {
            emit_diagnostics(&self.path.to_string_lossy(), &source, &diagnostics)?;
        }

        println!("{document:#?}");
        Ok(())
    }
}

/// Represents common analysis options.
#[derive(Args)]
pub struct AnalysisOptions {
    /// Denies all analysis rules by treating them as errors.
    #[clap(long, conflicts_with = "deny", conflicts_with = "except_all")]
    pub deny_all: bool,

    /// Except (ignores) all analysis rules.
    #[clap(long, conflicts_with = "except")]
    pub except_all: bool,

    /// Excepts (ignores) an analysis rule.
    #[clap(long)]
    pub except: Vec<String>,

    /// Denies an analysis rule by treating it as an error.
    #[clap(long)]
    pub deny: Vec<String>,
}

impl AnalysisOptions {
    /// Checks for conflicts in the analysis options.
    pub fn check_for_conflicts(&self) -> Result<()> {
        if let Some(id) = self.except.iter().find(|id| self.deny.contains(*id)) {
            bail!("rule `{id}` cannot be specified for both the `--except` and `--deny`",);
        }

        Ok(())
    }
}

/// Checks a WDL source file for errors.
#[derive(Args)]
#[clap(disable_version_flag = true)]
pub struct CheckCommand {
    /// The path or URL to the source WDL file.
    #[clap(value_name = "PATH or URL")]
    pub file: String,

    /// The analysis options.
    #[clap(flatten)]
    pub options: AnalysisOptions,
}

impl CheckCommand {
    /// Executes the `check` subcommand.
    async fn exec(self) -> Result<()> {
        self.options.check_for_conflicts()?;
        analyze(&self.file, self.options.except, false, false).await?;
        Ok(())
    }
}

/// Runs lint rules against a WDL source file.
#[derive(Args)]
#[clap(disable_version_flag = true)]
pub struct LintCommand {
    /// The path to the source WDL file.
    #[clap(value_name = "PATH")]
    pub path: PathBuf,
    /// Enable shellcheck lints.
    #[clap(long, action)]
    pub shellcheck: bool,
}

impl LintCommand {
    /// Executes the `lint` subcommand.
    async fn exec(self) -> Result<()> {
        let source = read_source(&self.path)?;
        let (document, diagnostics) = Document::parse(&source);
        if !diagnostics.is_empty() {
            emit_diagnostics(&self.path.to_string_lossy(), &source, &diagnostics)?;

            bail!(
                "aborting due to previous {count} diagnostic{s}",
                count = diagnostics.len(),
                s = if diagnostics.len() == 1 { "" } else { "s" }
            );
        }

        let mut validator = Validator::default();
        validator.add_visitor(LintVisitor::default());
        if self.shellcheck {
            validator.add_visitor(ShellCheckRule);
        }
        if let Err(diagnostics) = validator.validate(&document) {
            emit_diagnostics(&self.path.to_string_lossy(), &source, &diagnostics)?;

            bail!(
                "aborting due to previous {count} diagnostic{s}",
                count = diagnostics.len(),
                s = if diagnostics.len() == 1 { "" } else { "s" }
            );
        }

        Ok(())
    }
}

/// Analyzes a WDL source file.
#[derive(Args)]
#[clap(disable_version_flag = true)]
pub struct AnalyzeCommand {
    /// The path or URL to the source WDL file.
    #[clap(value_name = "PATH or URL")]
    pub file: String,

    /// The analysis options.
    #[clap(flatten)]
    pub options: AnalysisOptions,

    /// Whether or not to run lints as part of analysis.
    #[clap(long)]
    pub lint: bool,
}

impl AnalyzeCommand {
    /// Executes the `analyze` subcommand.
    async fn exec(self) -> Result<()> {
        self.options.check_for_conflicts()?;
        let results = analyze(&self.file, self.options.except, self.lint, false).await?;
        println!("{:#?}", results);
        Ok(())
    }
}

/// Formats a WDL source file.
#[derive(Args)]
#[clap(disable_version_flag = true)]
pub struct FormatCommand {
    /// The path to the source WDL file.
    #[clap(value_name = "PATH")]
    pub path: PathBuf,
}

impl FormatCommand {
    /// Executes the `format` subcommand.
    async fn exec(self) -> Result<()> {
        let source = read_source(&self.path)?;

        let (document, diagnostics) = Document::parse(&source);
        assert!(diagnostics.is_empty());

        if !diagnostics.is_empty() {
            emit_diagnostics(&self.path.to_string_lossy(), &source, &diagnostics)?;

            bail!(
                "aborting due to previous {count} diagnostic{s}",
                count = diagnostics.len(),
                s = if diagnostics.len() == 1 { "" } else { "s" }
            );
        }

        let document = Node::Ast(document.ast().into_v1().unwrap()).into_format_element();
        let formatter = Formatter::default();

        match formatter.format(&document) {
            Ok(formatted) => print!("{formatted}"),
            Err(err) => bail!(err),
        };

        Ok(())
    }
}

/// Finds a file matching the given name in the given directory.
///
/// This function will return the first match it finds, at any depth.
fn find_file_in_directory(name: &str, dir: &Path) -> Option<PathBuf> {
    fs::read_dir(dir)
        .ok()?
        .filter_map(|entry| entry.ok())
        .find_map(|entry| {
            let path = entry.path();
            if path.is_dir() {
                find_file_in_directory(name, &path)
            } else if path.file_name().map(|f| f == name).unwrap_or(false) {
                Some(path)
            } else {
                None
            }
        })
}

/// Document a workspace.
#[derive(Args)]
#[clap(disable_version_flag = true)]
pub struct DocCommand {
    /// The path to the workspace.
    #[clap(value_name = "PATH")]
    pub path: PathBuf,

    /// Whether or not to open the generated documentation in the default
    /// browser.
    #[clap(long)]
    pub open: bool,
}

impl DocCommand {
    /// Executes the `doc` subcommand.
    async fn exec(self) -> Result<()> {
        document_workspace(self.path.clone()).await?;

        if self.open {
            // find the first `$path/docs/**/index.html` file in the workspace
            // TODO: once we have a homepage, open that instead.
            if let Some(index) = find_file_in_directory("index.html", &self.path.join("docs")) {
                webbrowser::open(&index.as_path().to_string_lossy())
                    .context("failed to open browser")?;
            } else {
                eprintln!("failed to find `index.html` in workspace");
            }
        }

        Ok(())
    }
}

/// Validates an input JSON file against a WDL task or workflow.
#[derive(Args)]
#[clap(disable_version_flag = true)]
pub struct ValidateCommand {
    /// The path or URL to the source WDL file.
    #[clap(value_name = "PATH or URL")]
    pub document: String,

    /// The path to the inputs file.
    #[clap(long, value_name = "INPUTS")]
    pub inputs: PathBuf,
}

impl ValidateCommand {
    /// Executes the `validate` subcommand.
    async fn exec(self) -> Result<()> {
        validate_inputs(
            &self.document,
            &self.inputs,
            &mut StandardStream::stderr(ColorChoice::Auto),
            &Config::default(),
        )
        .await
    }
}

/// Runs a WDL workflow or task using local execution.
#[derive(Args)]
#[clap(disable_version_flag = true)]
pub struct RunCommand {
    /// The path or URL to the source WDL file.
    #[clap(value_name = "PATH or URL")]
    pub file: String,

    /// The path to the inputs file; defaults to an empty set of inputs.
    #[clap(short, long, value_name = "INPUTS", conflicts_with = "name")]
    pub inputs: Option<PathBuf>,

    /// The name of the workflow or task to run; defaults to the name specified
    /// in the inputs file; required if the inputs file is not specified.
    #[clap(short, long, value_name = "NAME")]
    pub name: Option<String>,

    /// The task execution output directory; defaults to the task name.
    #[clap(short, long, value_name = "OUTPUT_DIR")]
    pub output: Option<PathBuf>,

    /// Overwrites the task execution output directory if it exists.
    #[clap(long)]
    pub overwrite: bool,

    /// The analysis options.
    #[clap(flatten)]
    pub options: AnalysisOptions,
}

impl RunCommand {
    /// Executes the `run` subcommand.
    async fn exec(self) -> Result<()> {
        self.options.check_for_conflicts()?;

        run(
            &self.file,
            self.inputs,
            self.name.clone(),
            self.output.unwrap_or_else(|| {
                PathBuf::from(self.name.unwrap_or_else(|| "outputs".to_string()))
            }),
            &mut StandardStream::stderr(ColorChoice::Auto),
            &Config::default(),
        )
        .await
    }
}

/// A tool for parsing, validating, and linting WDL source code.
///
/// This command line tool is intended as an entrypoint to work with and develop
/// the `wdl` family of crates. It is not intended to be used by the broader
/// community. If you are interested in a command line tool designed to work
/// with WDL documents more generally, have a look at the `sprocket` command
/// line tool.
///
/// Link: https://github.com/stjude-rust-labs/sprocket
#[derive(Parser)]
#[clap(
    bin_name = "wdl",
    version,
    propagate_version = true,
    arg_required_else_help = true
)]
struct App {
    /// The subcommand to use.
    #[command(subcommand)]
    command: Command,

    /// The verbosity flags.
    #[command(flatten)]
    verbose: Verbosity,
}

#[derive(Subcommand)]
enum Command {
    /// Parses a WDL file.
    Parse(ParseCommand),

    /// Checks a WDL file.
    Check(CheckCommand),

    /// Lints a WDL file.
    Lint(LintCommand),

    /// Analyzes a WDL workspace.
    Analyze(AnalyzeCommand),

    /// Formats a WDL file.
    Format(FormatCommand),

    /// Documents a workspace.
    Doc(DocCommand),

    /// Validates an input file.
    Validate(ValidateCommand),

    /// Runs a workflow or task.
    Run(RunCommand),
}

#[tokio::main]
async fn main() -> Result<()> {
    let app = App::parse();

    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(app.verbose.log_level_filter().as_trace())
        .with_writer(std::io::stderr)
        .with_ansi(stderr().is_terminal())
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    if let Err(e) = match app.command {
        Command::Parse(cmd) => cmd.exec().await,
        Command::Check(cmd) => cmd.exec().await,
        Command::Lint(cmd) => cmd.exec().await,
        Command::Analyze(cmd) => cmd.exec().await,
        Command::Format(cmd) => cmd.exec().await,
        Command::Doc(cmd) => cmd.exec().await,
        Command::Validate(cmd) => cmd.exec().await,
        Command::Run(cmd) => cmd.exec().await,
    } {
        eprintln!(
            "{error}: {e:?}",
            error = if std::io::stderr().is_terminal() {
                "error".red().bold()
            } else {
                "error".normal()
            }
        );
        std::process::exit(1);
    }

    Ok(())
}
