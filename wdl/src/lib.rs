//! Workflow Description Language (WDL) document parsing and linting.
//!
//! There are three top-level modules to this crate:
//!
//! * `grammar` - used to parse WDL source into a Concrete Syntax Tree (CST).
//! * `ast` - used to parse a WDL document into an Abstract Syntax Tree (AST).
//! * `lint` - provides additional lint rules that can be used in a validation
//!   pass over a document.
//!
//! The above are re-exports of the individual `wdl-grammar`, `wdl-ast`, and
//! `wdl-lint` crates, respectively.
//!
//! The CST is based on the `rowan` crate and represents an immutable red-green
//! tree. Mutations to the tree require creating a new tree where unaffected
//! nodes are shared between the old and new trees; the cost of editing a node
//! of the tree depends solely on the depth of the node, as it must update the
//! parent chain to produce a new tree root.
//!
//! Note that in this implementation, the AST is a facade over the CST; each AST
//! representation internally holds a CST node or token. As a result, the AST is
//! very cheaply constructed and may be cheaply cloned at any level.
//!
//! # Examples
//!
//! An example of parsing WDL source into a CST and printing the tree:
//!
//! ```rust
//! use wdl::grammar::SyntaxTree;
//!
//! let (tree, diagnostics) = SyntaxTree::parse("version 1.1");
//! assert!(diagnostics.is_empty());
//! println!("{tree:#?}");
//! ```
//!
//! An example of parsing a WDL document into an AST and validating it:
//!
//! ```rust
//! # let source = "version 1.1\nworkflow test {}";
//! use wdl::ast::Document;
//! use wdl::ast::Validator;
//!
//! let (document, diagnostics) = Document::parse(source);
//! if !diagnostics.is_empty() {
//!     // Handle the failure to parse
//! }
//!
//! let mut validator = Validator::default();
//! if let Err(diagnostics) = validator.validate(&document) {
//!     // Handle the failure to validate
//! }
//! ```
//!
//! An example of parsing a WDL document and linting it:
//!
//! ```rust
//! # let source = "version 1.1\nworkflow test {}";
//! use wdl::ast::Document;
//! use wdl::ast::Validator;
//! use wdl::lint::LintVisitor;
//!
//! let (document, diagnostics) = Document::parse(source);
//! if !diagnostics.is_empty() {
//!     // Handle the failure to parse
//! }
//!
//! let mut validator = Validator::default();
//! validator.add_visitor(LintVisitor::default());
//! if let Err(diagnostics) = validator.validate(&document) {
//!     // Handle the failure to validate
//! }
//! ```

#![warn(missing_docs)]

use std::borrow::Cow;
use std::path::Path;
use std::path::PathBuf;
use std::path::absolute;
use std::time::Duration;

#[cfg(feature = "analysis")]
use anyhow::Context;
#[cfg(feature = "analysis")]
use anyhow::Result;
#[cfg(feature = "analysis")]
use anyhow::anyhow;
#[cfg(feature = "analysis")]
use anyhow::bail;
#[cfg(feature = "engine")]
use codespan_reporting::files::SimpleFile;
#[cfg(feature = "engine")]
use codespan_reporting::term::emit;
#[cfg(feature = "analysis")]
use indicatif::ProgressBar;
#[cfg(feature = "analysis")]
use indicatif::ProgressStyle;
#[cfg(feature = "engine")]
use serde_json::to_string_pretty;
#[cfg(feature = "analysis")]
use tokio::fs;
#[cfg(feature = "analysis")]
use url::Url;
#[cfg(feature = "analysis")]
#[doc(inline)]
pub use wdl_analysis as analysis;
#[cfg(feature = "analysis")]
use wdl_analysis::AnalysisResult;
#[cfg(feature = "analysis")]
use wdl_analysis::Analyzer;
#[cfg(feature = "analysis")]
use wdl_analysis::DiagnosticsConfig;
#[cfg(feature = "analysis")]
use wdl_analysis::path_to_uri;
#[cfg(feature = "analysis")]
use wdl_analysis::rules as analysis_rules;
#[cfg(feature = "ast")]
#[doc(inline)]
pub use wdl_ast as ast;
#[cfg(feature = "doc")]
#[doc(inline)]
pub use wdl_doc as doc;
#[cfg(feature = "engine")]
#[doc(inline)]
pub use wdl_engine as engine;
#[cfg(feature = "engine")]
use wdl_engine::Engine;
#[cfg(feature = "engine")]
use wdl_engine::EvaluationError;
#[cfg(feature = "engine")]
use wdl_engine::Inputs;
#[cfg(feature = "engine")]
use wdl_engine::local::LocalTaskExecutionBackend;
#[cfg(feature = "engine")]
use wdl_engine::v1::TaskEvaluator;
#[cfg(feature = "format")]
#[doc(inline)]
pub use wdl_format as format;
#[cfg(feature = "grammar")]
#[doc(inline)]
pub use wdl_grammar as grammar;
#[cfg(feature = "engine")]
use wdl_grammar::Diagnostic;
#[cfg(feature = "engine")]
use wdl_grammar::Severity;
#[cfg(feature = "lint")]
#[doc(inline)]
pub use wdl_lint as lint;
#[cfg(feature = "analysis")]
use wdl_lint::rules as lint_rules;
#[cfg(feature = "lsp")]
#[doc(inline)]
pub use wdl_lsp as lsp;

/// The delay in showing the progress bar.
///
/// This is to prevent the progress bar from flashing on the screen for
/// very short analyses.
const PROGRESS_BAR_DELAY_BEFORE_RENDER: Duration = Duration::from_secs(2);

#[cfg(feature = "analysis")]
/// Analyze the document or directory, returning [`AnalysisResult`]s.
pub async fn analyze(
    file: &str,
    exceptions: Vec<String>,
    lint: bool,
    shellcheck: bool,
) -> anyhow::Result<Vec<AnalysisResult>> {
    let rules = analysis_rules();
    let rules = rules
        .iter()
        .filter(|rule| !exceptions.iter().any(|e| e == rule.id()));
    let rules_config = DiagnosticsConfig::new(rules);

    let analyzer = Analyzer::new_with_validator(
        rules_config,
        move |bar: ProgressBar, kind, completed, total| async move {
            if bar.elapsed() < PROGRESS_BAR_DELAY_BEFORE_RENDER {
                return;
            }

            if completed == 0 || bar.length() == Some(0) {
                bar.set_length(total.try_into().unwrap());
                bar.set_message(format!("{kind}"));
            }

            bar.set_position(completed.try_into().unwrap());
        },
        move || {
            let mut validator = wdl_ast::Validator::default();

            if lint {
                let visitor =
                    wdl_lint::LintVisitor::new(lint_rules().into_iter().filter_map(|rule| {
                        if exceptions.iter().any(|e| e == rule.id()) {
                            None
                        } else {
                            Some(rule)
                        }
                    }));
                validator.add_visitor(visitor);

                if shellcheck {
                    let rule: Vec<Box<dyn wdl_lint::Rule>> =
                        vec![Box::<wdl_lint::rules::ShellCheckRule>::default()];
                    let visitor = wdl_lint::LintVisitor::new(rule);
                    validator.add_visitor(visitor);
                }
            }

            validator
        },
    );

    if let Ok(url) = Url::parse(file) {
        analyzer.add_document(url).await?;
    } else if fs::metadata(&file).await?.is_dir() {
        analyzer.add_directory(file.into()).await?;
    } else if let Some(url) = path_to_uri(file) {
        analyzer.add_document(url).await?;
    } else {
        bail!("failed to convert `{file}` to a URI", file = file)
    }

    let bar = ProgressBar::new(0);
    bar.set_style(
        ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {msg} {pos}/{len}")
            .unwrap(),
    );

    let results = analyzer.analyze(bar.clone()).await?;

    Ok(results)
}

#[cfg(feature = "engine")]
/// Validates the inputs for a task or workflow.
pub async fn validate_inputs(
    document: &str,
    inputs: &Path,
    stream: &mut codespan_reporting::term::termcolor::StandardStream,
    config: &codespan_reporting::term::Config,
) -> anyhow::Result<()> {
    if Path::new(&document).is_dir() {
        bail!("expected a WDL document, found a directory");
    }

    let results = analyze(document, vec![], false, false).await?;

    let uri = Url::parse(document)
        .unwrap_or_else(|_| path_to_uri(document).expect("file should be a local path"));

    let result = results
        .iter()
        .find(|r| **r.document().uri() == uri)
        .context("failed to find document in analysis results")?;
    let analyzed_document = result.document();

    let diagnostics: Cow<'_, [Diagnostic]> = match result.error() {
        Some(e) => vec![Diagnostic::error(format!(
            "failed to read `{document}`: {e:#}"
        ))]
        .into(),
        None => analyzed_document.diagnostics().into(),
    };

    if let Some(diagnostic) = diagnostics.iter().find(|d| d.severity() == Severity::Error) {
        let source = result.document().node().syntax().text().to_string();
        let file = SimpleFile::new(&document, &source);

        emit(stream, config, &file, &diagnostic.to_codespan()).expect("should emit");

        bail!(
            "document `{document}` contains at least one diagnostic error!\ncan't validate inputs",
            document = document
        );
    }

    let result = match Inputs::parse(analyzed_document, inputs) {
        Ok(Some((name, inputs))) => match inputs {
            Inputs::Task(inputs) => {
                match inputs
                    .validate(
                        analyzed_document,
                        analyzed_document
                            .task_by_name(&name)
                            .expect("task should exist"),
                        None,
                    )
                    .with_context(|| {
                        format!("failed to validate inputs for task `{name}`", name = name)
                    }) {
                    Ok(()) => String::new(),
                    Err(e) => format!("{e:?}"),
                }
            }
            Inputs::Workflow(inputs) => {
                let workflow = analyzed_document.workflow().expect("workflow should exist");
                match inputs
                    .validate(analyzed_document, workflow, None)
                    .with_context(|| {
                        format!(
                            "failed to validate inputs for workflow `{name}`",
                            name = name
                        )
                    }) {
                    Ok(()) => String::new(),
                    Err(e) => format!("{e:?}"),
                }
            }
        },
        Ok(None) => String::new(),
        Err(e) => format!("{e:?}"),
    };

    if !result.is_empty() {
        bail!("failed to validate inputs:\n{result}");
    }

    Ok(())
}

#[cfg(feature = "engine")]
/// Run a WDL task or workflow.
pub async fn run(
    file: &str,
    inputs: Option<PathBuf>,
    name: Option<String>,
    output: PathBuf,
    stream: &mut codespan_reporting::term::termcolor::StandardStream,
    config: &codespan_reporting::term::Config,
) -> Result<()> {
    if Path::new(file).is_dir() {
        anyhow::bail!("expected a WDL document, found a directory");
    }

    let results = analyze(file, vec![], false, false).await?;

    let uri = Url::parse(file)
        .unwrap_or_else(|_| path_to_uri(file).expect("file should be a local path"));

    let result = results
        .iter()
        .find(|r| **r.document().uri() == uri)
        .context("failed to find document in analysis results")?;
    let document = result.document();

    let mut engine = Engine::new(LocalTaskExecutionBackend::new());
    let (path, name, inputs) = if let Some(path) = inputs {
        let abs_path = absolute(&path).with_context(|| {
            format!(
                "failed to determine the absolute path of `{path}`",
                path = path.display()
            )
        })?;
        match Inputs::parse(document, &abs_path)? {
            Some((name, inputs)) => (Some(path), name, inputs),
            None => bail!(
                "inputs file `{path}` is empty; use the `--name` option to specify the name of \
                 the task or workflow to run",
                path = path.display()
            ),
        }
    } else if let Some(name) = name {
        if document.task_by_name(&name).is_some() {
            (None, name, Inputs::Task(Default::default()))
        } else if document.workflow().is_some() {
            if name != document.workflow().unwrap().name() {
                bail!("document does not contain a workflow named `{name}`");
            }
            (None, name, Inputs::Workflow(Default::default()))
        } else {
            bail!("document does not contain a task or workflow named `{name}`");
        }
    } else {
        let mut iter = document.tasks();
        let (name, inputs) = iter
            .next()
            .map(|t| (t.name().to_string(), Inputs::Task(Default::default())))
            .or_else(|| {
                document
                    .workflow()
                    .map(|w| (w.name().to_string(), Inputs::Workflow(Default::default())))
            })
            .context("inputs file is empty and the WDL document contains no tasks or workflow")?;

        if iter.next().is_some() {
            bail!("inputs file is empty and the WDL document contains more than one task");
        }

        (None, name, inputs)
    };

    match inputs {
        Inputs::Task(mut inputs) => {
            let task = document
                .task_by_name(&name)
                .ok_or_else(|| anyhow!("document does not contain a task named `{name}`"))?;

            // Ensure all the paths specified in the inputs file are relative to the file's
            // directory
            if let Some(path) = path.as_ref().and_then(|p| p.parent()) {
                inputs.join_paths(task, path);
            }

            let mut evaluator = TaskEvaluator::new(&mut engine);
            match evaluator
                .evaluate(document, task, &inputs, &output, &name)
                .await
            {
                Ok(evaluated) => match evaluated.into_result() {
                    Ok(outputs) => {
                        println!("{}", to_string_pretty(&outputs)?);
                    }
                    Err(e) => match e {
                        EvaluationError::Source(diagnostic) => {
                            let file = SimpleFile::new(
                                uri.to_string(),
                                document.node().syntax().text().to_string(),
                            );
                            emit(stream, config, &file, &diagnostic.to_codespan())?;

                            bail!("aborting due to task evaluation failure");
                        }
                        EvaluationError::Other(e) => return Err(e),
                    },
                },
                Err(e) => match e {
                    EvaluationError::Source(diagnostic) => {
                        let file = SimpleFile::new(
                            uri.to_string(),
                            document.node().syntax().text().to_string(),
                        );
                        emit(stream, config, &file, &diagnostic.to_codespan())?;

                        bail!("aborting due to task evaluation failure");
                    }
                    EvaluationError::Other(e) => return Err(e),
                },
            }
        }
        Inputs::Workflow(mut inputs) => {
            let workflow = document
                .workflow()
                .ok_or_else(|| anyhow!("document does not contain a workflow"))?;

            // Ensure all the paths specified in the inputs file are relative to the file's
            // directory
            if let Some(path) = path.as_ref().and_then(|p| p.parent()) {
                inputs.join_paths(workflow, path);
            }

            bail!("running workflows is not yet supported")
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    /// This is a test for checking that the reserved rules in `wdl-lint` match
    /// those from `wdl-analysis`.
    #[cfg(all(feature = "analysis", feature = "lint"))]
    #[test]
    fn reserved_rule_ids() {
        let rules: HashSet<_> = wdl_analysis::rules().iter().map(|r| r.id()).collect();
        let reserved: HashSet<_> = wdl_lint::RESERVED_RULE_IDS.iter().copied().collect();

        for id in &reserved {
            if !rules.contains(id) {
                panic!("analysis rule `{id}` is not in the reservation set");
            }
        }

        for id in &rules {
            if !reserved.contains(id) {
                panic!("reserved rule `{id}` is not an analysis rule");
            }
        }
    }
}
