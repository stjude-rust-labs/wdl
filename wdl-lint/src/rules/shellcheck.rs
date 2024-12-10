//! A lint rule for running shellcheck against command sections.
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::Write;
use std::process;
use std::process::Stdio;
use std::sync::OnceLock;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use rand::distributions::Alphanumeric;
use rand::distributions::DistString;
use serde::Deserialize;
use serde_json;

use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::Diagnostic;
use wdl_ast::Diagnostics;
use wdl_ast::Document;
use wdl_ast::Span;
use wdl_ast::SupportedVersion;
use wdl_ast::SyntaxElement;
use wdl_ast::SyntaxKind;
use wdl_ast::ToSpan;
use wdl_ast::VisitReason;
use wdl_ast::Visitor;
use wdl_ast::support;
use wdl_ast::v1::CommandPart;
use wdl_ast::v1::CommandSection;
use wdl_ast::v1::Placeholder;
use wdl_ast::v1::StrippedCommandPart;
use wdl_ast::v1::TaskDefinition;

use crate::Rule;
use crate::Tag;
use crate::TagSet;
use crate::util::{count_leading_whitespace, lines_with_offset, program_exists};

/// The shellcheck executable
const SHELLCHECK_BIN: &str = "shellcheck";

/// shellcheck lints that we want to suppress
const SHELLCHECK_SUPPRESS: &[&str] = &[
    "1009", // the mentioned parser error was in...
    "1072", // Unexpected
    "1083", // this {/} is literal
];

/// ShellCheck: var is referenced by not assigned.
const SHELLCHECK_REFERENCED_UNASSIGNED: usize = 2154;

/// Whether or not shellcheck exists on the system
static SHELLCHECK_EXISTS: OnceLock<bool> = OnceLock::new();

/// The identifier for the command section ShellCheck rule.
const ID: &str = "CommandSectionShellCheck";

/// A ShellCheck comment.
///
/// The file and fix fields are ommitted as we have no use for them.
#[derive(Clone, Debug, Deserialize)]
struct ShellCheckDiagnostic {
    /// line number comment starts on
    pub line: usize,
    /// line number comment ends on
    #[serde(rename = "endLine")]
    pub end_line: usize,
    /// column comment starts on
    pub column: usize,
    /// column comment ends on
    #[serde(rename = "endColumn")]
    pub end_column: usize,
    /// severity of the comment
    pub level: String,
    /// shellcheck error code
    pub code: usize,
    /// message associated with the comment
    pub message: String,
}

/// Run shellcheck on a command.
///
/// writes command text to stdin of shellcheck process
/// and returns parsed `ShellCheckDiagnostic`s
fn run_shellcheck(command: &str) -> Result<Vec<ShellCheckDiagnostic>> {
    let mut sc_proc = process::Command::new(SHELLCHECK_BIN)
        .args([
            "-s",
            "bash",
            "-f",
            "json",
            "-e",
            &SHELLCHECK_SUPPRESS.join(","),
            "-S",
            "style",
            "-",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .context("spawning the `shellcheck` process")?;
    {
        let mut proc_stdin = sc_proc
            .stdin
            .take()
            .context("obtaining the STDIN handle of the `shellcheck` process")?;
        proc_stdin.write_all(command.as_bytes())?;
    }

    let output = sc_proc
        .wait_with_output()
        .context("waiting for the `shellcheck` process to complete")?;

    // shellcheck returns exit code 1 if
    // any checked files result in comments
    // so cannot check with status.success()
    match output.status.code() {
        Some(0) | Some(1) => serde_json::from_slice::<Vec<ShellCheckDiagnostic>>(&output.stdout)
            .context("deserializing STDOUT from `shellcheck` process"),
        Some(code) => bail!("unexpected `shellcheck` exit code: {}", code),
        None => bail!("the `shellcheck` process appears to have been interrupted"),
    }
}

/// Runs ShellCheck on a command section and reports violations
#[derive(Default, Debug, Clone, Copy)]
pub struct ShellCheckRule;

impl Rule for ShellCheckRule {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Ensures that command blocks are free of ShellCheck violations."
    }

    fn explanation(&self) -> &'static str {
        "ShellCheck (https://shellcheck.net) is a static analysis tool and linter for sh / bash. \
        The lints provided by ShellCheck help prevent common errors and \
        pitfalls in your scripts. Following its recommendations will increase \
        the robustness of your command sections."
    }

    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Correctness, Tag::Style, Tag::Portability])
    }

    fn exceptable_nodes(&self) -> Option<&'static [SyntaxKind]> {
        Some(&[
            SyntaxKind::VersionStatementNode,
            SyntaxKind::TaskDefinitionNode,
            SyntaxKind::CommandSectionNode,
        ])
    }
}

/// Convert a WDL `Placeholder` to a bash variable declaration.
///
/// returns a random string three characters shorter than the Placeholder's length
/// to account for '~{}'.
fn to_bash_var(placeholder: &Placeholder) -> String {
    let placeholder_len: usize = placeholder.syntax().text_range().len().into();
    Alphanumeric.sample_string(&mut rand::thread_rng(), placeholder_len - 3)
}

/// Retrieve all input and private declarations for a task.
fn gather_task_declarations(task: &TaskDefinition) -> HashSet<String> {
    let mut decls = HashSet::new();
    if let Some(input) = task.input() {
        for decl in input.declarations() {
            decls.insert(decl.name().as_str().to_owned());
        }
    }

    for decl in task.declarations() {
        decls.insert(decl.name().as_str().to_owned());
    }
    decls
}

/// Creates a "ShellCheck lint" diagnostic from a ShellCheckComment
fn shellcheck_lint(comment: &ShellCheckComment, span: Span) -> Diagnostic {
    dbg!(
        Diagnostic::note("`shellcheck` reported the following diagnostic")
            .with_rule(ID)
            .with_label(
                format!("SC{}[{}]: {}", comment.code, comment.level, comment.message),
                span,
            )
            .with_fix("address the diagnostics as recommended in the message")
    )
}

impl Visitor for ShellCheckRule {
    type State = Diagnostics;

    fn document(
        &mut self,
        _: &mut Self::State,
        reason: VisitReason,
        _: &Document,
        _: SupportedVersion,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        // Reset the visitor upon document entry
        *self = Default::default();
    }

    fn command_section(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        section: &CommandSection,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        if !SHELLCHECK_EXISTS.get_or_init(|| {
            if !program_exists(SHELLCHECK_BIN) {
                let command_keyword = support::token(section.syntax(), SyntaxKind::CommandKeyword)
                    .expect("should have a command keyword token");
                state.exceptable_add(
                    Diagnostic::error("running `shellcheck` on command section")
                        .with_label(
                            "could not find `shellcheck` executable.",
                            command_keyword.text_range().to_span(),
                        )
                        .with_rule(ID)
                        .with_fix("install shellcheck or disable this lint."),
                    SyntaxElement::from(section.syntax().clone()),
                    &self.exceptable_nodes(),
                );
                return false;
            }
            true
        }) {
            return;
        }

        // Collect declarations so we can ignore placeholder variables
        let parent_task = section.parent().into_task().expect("parent is a task");
        let mut decls = gather_task_declarations(&parent_task);

        // Replace all placeholders in the command with
        // dummy bash variables
        let mut sanitized_command = String::new();
        if let Some(cmd_parts) = section.strip_whitespace() {
            cmd_parts.iter().for_each(|part| match part {
                StrippedCommandPart::Text(text) => {
                    sanitized_command.push_str(text);
                }
                StrippedCommandPart::Placeholder(placeholder) => {
                    let bash_var = to_bash_var(placeholder);
                    // we need to save the var so we can suppress later
                    decls.insert(bash_var.clone());
                    let mut expansion = String::from("\"$");
                    expansion.push_str(&bash_var);
                    expansion.push('"');
                    sanitized_command.push_str(&expansion);
                }
            });
        } else {
            // This is the case where the command section contains
            // mixed indentation. We silently return and allow
            // the mixed indentation lint to report this.
            return;
        }

        // get leading whitespace so we can add it to each span
        let leading_whitespace = {
            if let Some(first_part) = section.parts().next() {
                match first_part {
                    CommandPart::Text(text) => {
                        let text_str = text
                            .as_str()
                            .strip_prefix("\n")
                            .unwrap_or_else(|| text.as_str());
                        count_leading_whitespace(text_str)
                    }
                    CommandPart::Placeholder(pholder) => {
                        let pholder_str = &pholder.syntax().text().to_string();
                        count_leading_whitespace(
                            pholder_str.strip_prefix("\n").unwrap_or(pholder_str),
                        )
                    }
                }
            } else {
                0
            }
        };

        // Map each actual line of the command to its corresponding
        // `CommandPart` and start position.
        let mut line_map = HashMap::new();
        let mut on_same_line = false;
        let mut line_num = 0usize;
        for part in section.parts() {
            match part {
                CommandPart::Text(ref text) => {
                    let mut last_line = 0usize;
                    for (_, start, _) in lines_with_offset(text.as_str()) {
                        // Check to see if we are still on the previous line
                        // This happens after we encounter a placeholder
                        if on_same_line {
                            on_same_line = false;
                            last_line = last_line.saturating_sub(1);
                        }
                        line_map.insert(line_num + last_line, (part.clone(), start));
                        last_line += 1;
                    }
                    line_num += last_line;
                }
                CommandPart::Placeholder(_) => {
                    on_same_line = true;
                }
            }
        }

        match run_shellcheck(&sanitized_command) {
            Ok(comments) => {
                for comment in comments {
                    // Skip declarations that shellcheck is unaware of.
                    if comment.code == SHELLCHECK_REFERENCED_UNASSIGNED
                        && decls.contains(comment.message.split_whitespace().next().unwrap_or(""))
                    {
                        continue;
                    }
                    let (rel_part, start) = line_map
                        .get(&comment.line)
                        .expect("shellcheck line corresponds to command line");
                    let rel_token = match rel_part {
                        CommandPart::Text(text) => text.syntax(),
                        CommandPart::Placeholder(phold) => {
                            &support::token(phold.syntax(), SyntaxKind::PlaceholderNode)
                                .expect("should have a placeholder node token")
                        }
                    };
                    let inner_span = {
                        let outer_span = rel_token.text_range().to_span();
                        Span::new(
                            outer_span.start() + leading_whitespace + start + comment.column - 1,
                            comment.end_column - comment.column,
                        )
                    };
                    state.exceptable_add(
                        shellcheck_lint(&comment, inner_span),
                        SyntaxElement::from(rel_token.clone()),
                        &self.exceptable_nodes(),
                    )
                }
            }
            Err(e) => {
                let command_keyword = support::token(section.syntax(), SyntaxKind::CommandKeyword)
                    .expect("should have a command keyword token");
                state.exceptable_add(
                    Diagnostic::error("running `shellcheck` on command section")
                        .with_label(e.to_string(), command_keyword.text_range().to_span())
                        .with_rule(ID)
                        .with_fix("address reported error."),
                    SyntaxElement::from(section.syntax().clone()),
                    &self.exceptable_nodes(),
                );
            }
        }
    }
}
