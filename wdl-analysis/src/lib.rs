//! Analysis of Workflow Description Language (WDL) documents.
//!
//! An analyzer can be used to implement the [Language Server Protocol (LSP)](https://microsoft.github.io/language-server-protocol/).

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(missing_debug_implementations)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(rustdoc::broken_intra_doc_links)]

mod analyzer;
pub mod diagnostics;
pub mod document;
pub mod eval;
mod graph;
mod queue;
mod rayon;
mod rules;
pub mod stdlib;
pub mod types;

use std::time::Duration;

pub use analyzer::*;
use anyhow::bail;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
pub use rules::*;
use tokio::fs;
use url::Url;

/// The delay in showing the progress bar.
///
/// This is to prevent the progress bar from flashing on the screen for
/// very short analyses.
const PROGRESS_BAR_DELAY_BEFORE_RENDER: Duration = Duration::from_secs(2);

/// Analyze the document or directory, returning [`AnalysisResult`]s.
pub async fn analyze(
    file: &str,
    exceptions: Vec<String>,
    lint: bool,
    shellcheck: bool,
) -> anyhow::Result<Vec<AnalysisResult>> {
    let rules = rules();
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
                    wdl_lint::LintVisitor::new(wdl_lint::rules().into_iter().filter_map(|rule| {
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
