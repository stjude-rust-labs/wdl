//! A lint rule for ensuring that imports are sorted lexicographically.

use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::Diagnostic;
use wdl_ast::Diagnostics;
use wdl_ast::Document;
use wdl_ast::Span;
use wdl_ast::SupportedVersion;
use wdl_ast::SyntaxKind;
use wdl_ast::ToSpan;
use wdl_ast::VisitReason;
use wdl_ast::Visitor;
use wdl_ast::v1::ImportStatement;

use crate::Rule;
use crate::Tag;
use crate::TagSet;

/// The identifier for the import sort rule.
const ID: &str = "ImportSort";

/// Creates an import not sorted diagnostic.
fn import_not_sorted(span: Span, sorted_imports: String) -> Diagnostic {
    Diagnostic::note("imports are not sorted lexicographically")
        .with_rule(ID)
        .with_highlight(span)
        .with_fix(format!(
            "sort the imports lexicographically:\n{}",
            sorted_imports
        ))
}
/// Creates an improper comment diagnostic.
fn improper_comment(span: Span) -> Diagnostic {
    Diagnostic::note("comments are not allowed within an import statement")
        .with_rule(ID)
        .with_highlight(span)
        .with_fix("remove the comment from the import statement")
}

/// Detects imports that are not sorted lexicographically.
#[derive(Default, Debug, Clone, Copy)]
pub struct ImportSortRule;

impl Rule for ImportSortRule {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Ensures that imports are sorted lexicographically."
    }

    fn explanation(&self) -> &'static str {
        "Imports should be sorted lexicographically to make it easier to find specific imports. \
         This rule ensures that imports are sorted in a consistent manner. Specifically, the \
         desired sort can be achieved with a GNU compliant `sort` and `LC_COLLATE=C`. No comments \
         are permitted within an import statement."
    }

    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Style, Tag::Clarity, Tag::Sorting])
    }

    fn exceptable_nodes(&self) -> Option<&'static [SyntaxKind]> {
        Some(&[SyntaxKind::VersionStatementNode])
    }
}

impl Visitor for ImportSortRule {
    type State = Diagnostics;

    fn document(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        doc: &Document,
        _: SupportedVersion,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        *self = Default::default();

        // Collect all import statements
        let imports: Vec<_> = doc
            .syntax()
            .children_with_tokens()
            .filter(|n| n.kind() == SyntaxKind::ImportStatementNode)
            .filter_map(|c| c.into_node())
            .collect();

        if imports.is_empty() {
            return;
        }

        // Clone imports for comparison
        let mut sorted_imports = imports.clone();
        sorted_imports.sort_by(|a, b| {
            let a_uri = ImportStatement::cast(a.clone())
                .expect("import statement")
                .uri()
                .text()
                .expect("import uri");
            let b_uri = ImportStatement::cast(b.clone())
                .expect("import statement")
                .uri()
                .text()
                .expect("import uri");
            a_uri.as_str().cmp(b_uri.as_str())
        });

        // Check if sorting is needed
        if imports != sorted_imports {
            // Find the first out-of-order import
            let first_out_of_order = sorted_imports
                .iter()
                .find(|sorted_import| {
                    let original_position = imports.iter().position(|orig| orig == *sorted_import);
                    let sorted_position = sorted_imports
                        .iter()
                        .position(|sorted| sorted == *sorted_import);
                    original_position != sorted_position
                })
                .unwrap();

            let span = imports
                .iter()
                .find(|orig| **orig == *first_out_of_order)
                .unwrap()
                .text_range()
                .to_span();
            let sorted_imports_text = sorted_imports
                .iter()
                .map(|import| import.text().to_string())
                .collect::<Vec<_>>()
                .join("\n");

            state.add(import_not_sorted(span, sorted_imports_text));
        }
    }

    fn import_statement(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        stmt: &ImportStatement,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        // Check for comments inside this import statement.
        let internal_comments = stmt
            .syntax()
            .children_with_tokens()
            .filter(|c| c.kind() == SyntaxKind::Comment)
            .map(|c| c.into_token().unwrap());

        for comment in internal_comments {
            // Since this rule can only be excepted in a document-wide fashion,
            // if the rule is running we can directly add the diagnostic
            // without checking for the exceptable nodes
            state.add(improper_comment(comment.text_range().to_span()));
        }
    }
}
