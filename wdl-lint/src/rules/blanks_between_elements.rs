//! A lint rule for blank spacing between elements.

use rowan::NodeOrToken;
use wdl_ast::v1::InputSection;
use wdl_ast::AstNode;
use wdl_ast::Diagnostic;
use wdl_ast::Diagnostics;
use wdl_ast::Document;
use wdl_ast::Span;
use wdl_ast::SupportedVersion;
use wdl_ast::SyntaxElement;
use wdl_ast::SyntaxKind;
use wdl_ast::SyntaxNode;
use wdl_ast::SyntaxToken;
use wdl_ast::ToSpan;
use wdl_ast::VisitReason;
use wdl_ast::Visitor;

use crate::Rule;
use crate::Tag;
use crate::TagSet;

/// The identifier for the blanks between elements rule.
const ID: &str = "BlanksBetweenElements";

/// Creates an excessive blank line diagnostic.
fn excess_blank_line(span: Span) -> Diagnostic {
    Diagnostic::note("extra blank line(s) found")
        .with_rule(ID)
        .with_highlight(span)
        .with_fix("remove blank line(s)")
}

/// Creates a missing blank line diagnostic.
fn missing_blank_line(span: Span) -> Diagnostic {
    Diagnostic::note("missing blank line")
        .with_rule(ID)
        .with_highlight(span)
        .with_fix("add blank line before this element")
}

/// Detects unsorted input declarations.
#[derive(Default, Debug, Clone, Copy)]
pub struct BlanksBetweenElementsRule {
    /// Are we in a `input` section?
    input_section: bool,
    /// Are we in a `output` section?
    output_section: bool,
    /// Are we in a `meta` section?
    meta_section: bool,
    /// Are we in a `parameter_meta` section?
    parameter_meta_section: bool,
    /// Are we in a `runtime` section?
    runtime_section: bool,
    /// Are we in a `scatter` block?
    in_scatter: bool,
}

impl Rule for BlanksBetweenElementsRule {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Ensures that WDL elements are spaced appropriately."
    }

    fn explanation(&self) -> &'static str {
        "There should be a blank line between each WDL element at the root indentation level (such \
         as the import block and any task/workflow definitions) and between sections of a WDL task \
         or workflow. Never have a blank line when indentation levels are changing (such as \
         between the opening of a workflow definition and the meta section). There should also \
         never be blanks within a meta, parameter meta, input, output, or runtime section. See \
         example for a complete WDL document with proper spacing between elements. Note the blank \
         lines between meta, parameter meta, input, the first call or first private declaration, \
         output, and runtime for the example task. The blank line between the workflow definition \
         and the task definition is also important."
    }

    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Spacing])
    }
}

impl Visitor for BlanksBetweenElementsRule {
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

        if reason == VisitReason::Enter {
            // Reset the visitor upon document entry
            *self = Default::default();
        }
    }

    fn task_definition(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        task: &wdl_ast::v1::TaskDefinition,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        let first = is_first_element(task.syntax());
        let actual_start = skip_preceding_comments(task.syntax());
        check_prior_spacing(&actual_start, state, true, first);
    }

    fn workflow_definition(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        workflow: &wdl_ast::v1::WorkflowDefinition,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        let first = is_first_element(workflow.syntax());
        let actual_start = skip_preceding_comments(workflow.syntax());
        check_prior_spacing(&actual_start, state, true, first);
    }

    fn metadata_section(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        section: &wdl_ast::v1::MetadataSection,
    ) {
        if reason == VisitReason::Exit {
            self.meta_section = false;
            return;
        } else {
            self.meta_section = true;
        }

        let first = is_first_element(section.syntax());
        let actual_start = skip_preceding_comments(section.syntax());
        check_prior_spacing(&actual_start, state, true, first);
        flag_all_blanks(section.syntax(), state);
    }

    fn parameter_metadata_section(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        section: &wdl_ast::v1::ParameterMetadataSection,
    ) {
        if reason == VisitReason::Exit {
            self.parameter_meta_section = false;
            return;
        } else {
            self.parameter_meta_section = true;
        }

        let first = is_first_element(section.syntax());
        let actual_start = skip_preceding_comments(section.syntax());
        check_prior_spacing(&actual_start, state, true, first);
        flag_all_blanks(section.syntax(), state);
    }

    fn input_section(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        section: &InputSection,
    ) {
        if reason == VisitReason::Exit {
            self.input_section = false;
            return;
        } else {
            self.input_section = true;
        }

        let first = is_first_element(section.syntax());
        let actual_start = skip_preceding_comments(section.syntax());
        check_prior_spacing(&actual_start, state, true, first);
    }

    fn command_section(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        section: &wdl_ast::v1::CommandSection,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        let first = is_first_element(section.syntax());
        let actual_start = skip_preceding_comments(section.syntax());
        check_prior_spacing(&actual_start, state, true, first);
    }

    fn output_section(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        section: &wdl_ast::v1::OutputSection,
    ) {
        if reason == VisitReason::Exit {
            self.output_section = false;
            return;
        } else {
            self.output_section = true;
        }
        let first = is_first_element(section.syntax());
        let actual_start = skip_preceding_comments(section.syntax());
        check_prior_spacing(&actual_start, state, true, first);
    }

    fn runtime_section(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        section: &wdl_ast::v1::RuntimeSection,
    ) {
        if reason == VisitReason::Exit {
            self.runtime_section = false;
            return;
        } else {
            self.runtime_section = true;
        }

        flag_all_blanks(section.syntax(), state);
        let first = is_first_element(section.syntax());
        let actual_start = skip_preceding_comments(section.syntax());
        check_prior_spacing(&actual_start, state, true, first);
    }

    // call statement internal spacing is handled by the CallInputSpacing rule
    fn call_statement(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        stmt: &wdl_ast::v1::CallStatement,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        let first = is_first_element(stmt.syntax());

        let prev = skip_preceding_comments(stmt.syntax());

        let between_calls = stmt
            .syntax()
            .prev_sibling()
            .map(|s| s.kind() == SyntaxKind::CallStatementNode)
            .unwrap_or(false);

        if !between_calls {
            check_prior_spacing(&prev, state, true, first);
        }
    }

    fn scatter_statement(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        stmt: &wdl_ast::v1::ScatterStatement,
    ) {
        if reason == VisitReason::Exit {
            self.in_scatter = false;
            return;
        } else {
            self.in_scatter = true;
        }

        let prev_token = stmt
            .syntax()
            .prev_sibling_or_token()
            .and_then(SyntaxElement::into_token);

        let first = is_first_element(stmt.syntax());

        if let Some(token) = prev_token {
            if token.kind() == SyntaxKind::Whitespace {
                let count = token.to_string().chars().filter(|c| *c == '\n').count();
                if first && count > 1 {
                    state.add(excess_blank_line(token.text_range().to_span()));
                } else if !first && count < 2 {
                    state.add(missing_blank_line(stmt.syntax().text_range().to_span()));
                }
            }
        } else if !first {
            state.add(missing_blank_line(stmt.syntax().text_range().to_span()));
        }
    }

    // Import spacing is handled by the ImportWhitespace rule
    // fn import_statement

    fn struct_definition(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        def: &wdl_ast::v1::StructDefinition,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        let first = is_first_element(def.syntax());
        let actual_start = skip_preceding_comments(def.syntax());
        check_prior_spacing(&actual_start, state, true, first);
    }

    fn requirements_section(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        section: &wdl_ast::v1::RequirementsSection,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        let first = is_first_element(section.syntax());
        let actual_start = skip_preceding_comments(section.syntax());
        check_prior_spacing(&actual_start, state, true, first);
        flag_all_blanks(section.syntax(), state);
    }

    fn unbound_decl(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        decl: &wdl_ast::v1::UnboundDecl,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        let first = is_first_element(decl.syntax());
        let prior_node = decl.syntax().prev_sibling();

        let prior = decl.syntax().prev_sibling_or_token();
        if let Some(p) = prior {
            if p.kind() == SyntaxKind::Whitespace {
                let count = p.to_string().chars().filter(|c| *c == '\n').count();
                // If we're in an `input` or `output`, we should have no blank lines, so only
                // one `\n` is allowed.
                if self.input_section || self.output_section {
                    if count > 1 {
                        state.add(excess_blank_line(p.text_range().to_span()));
                    }
                } else if let Some(n) = prior_node {
                    match n.kind() {
                        SyntaxKind::BoundDeclNode | SyntaxKind::UnboundDeclNode => {
                            // If this is not the first (Un)BoundDeclNode,
                            // then blank lines are optional.
                            // More than one blank line will be caught by
                            // the Whitespace rule.
                        }
                        _ => {
                            if count < 2 && !first {
                                state.add(missing_blank_line(decl.syntax().text_range().to_span()));
                            }
                        }
                    }
                }
            }
        }
    }

    fn bound_decl(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        decl: &wdl_ast::v1::BoundDecl,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        let first = is_first_element(decl.syntax());
        let actual_start = skip_preceding_comments(decl.syntax());

        let prior_node = decl.syntax().prev_sibling();

        let prior = actual_start.prev_sibling_or_token();
        if let Some(p) = prior {
            if p.kind() == SyntaxKind::Whitespace {
                let count = p.to_string().chars().filter(|c| *c == '\n').count();
                // If we're in an `input` or `output`, we should have no blank lines, so only
                // one `\n` is allowed.
                if self.input_section || self.output_section {
                    if count > 1 {
                        state.add(excess_blank_line(p.text_range().to_span()));
                    }
                } else if let Some(n) = prior_node {
                    match n.kind() {
                        SyntaxKind::BoundDeclNode | SyntaxKind::UnboundDeclNode => {
                            // If this is not the first (Un)BoundDeclNode,
                            // then blank lines are optional.
                            // More than one blank line will be caught by
                            // the Whitespace rule.
                        }
                        _ => {
                            if count < 2 && !first {
                                state.add(missing_blank_line(decl.syntax().text_range().to_span()));
                            }
                        }
                    }
                }
            }
        }
    }

    fn conditional_statement(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        stmt: &wdl_ast::v1::ConditionalStatement,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        let first = is_first_element(stmt.syntax());
        let actual_start = skip_preceding_comments(stmt.syntax());
        check_prior_spacing(&actual_start, state, true, first);
    }
}

/// Check if the given syntax node is the first element in the block.
fn is_first_element(syntax: &SyntaxNode) -> bool {
    let mut prev = syntax.prev_sibling_or_token();
    while let Some(ref cur) = prev {
        match cur {
            NodeOrToken::Token(t) => {
                if t.kind() == SyntaxKind::OpenBrace {
                    return true;
                }
            }
            NodeOrToken::Node(_) => {
                return false;
            }
        }
        prev = cur.prev_sibling_or_token();
    }
    true
}

/// Some sections do not allow blank lines, so detect and flag them.
fn flag_all_blanks(syntax: &SyntaxNode, state: &mut Diagnostics) {
    syntax.descendants_with_tokens().for_each(|c| {
        if c.kind() == SyntaxKind::Whitespace {
            let count = c.to_string().chars().filter(|c| *c == '\n').count();
            if count > 1 {
                state.add(excess_blank_line(c.text_range().to_span()));
            }
        }
    });
}

/// Check that an item has space prior to it.
/// element_spacing indicates if spacing is required (true) or not (false).
fn check_prior_spacing(
    syntax: &NodeOrToken<SyntaxNode, SyntaxToken>,
    state: &mut Diagnostics,
    element_spacing: bool,
    first: bool,
) {
    if let Some(prior) = syntax.prev_sibling_or_token() {
        match prior.kind() {
            SyntaxKind::Whitespace => {
                let count = prior.to_string().chars().filter(|c| *c == '\n').count();
                if first || !element_spacing {
                    // first element cannot have a blank line before it
                    if count > 1 {
                        state.add(excess_blank_line(prior.text_range().to_span()));
                    }
                } else if count < 2 && element_spacing {
                    state.add(missing_blank_line(syntax.text_range().to_span()));
                }
            }
            // Something other than whitespace precedes
            _ => {
                // If we require between element spacing and are not the first element,
                // we're missing a blank line.
                if element_spacing && !first {
                    state.add(missing_blank_line(syntax.text_range().to_span()));
                }
            }
        }
    } else {
        // If nothing precedes the Node/Token, we must be the first element
        if !first {
            unreachable!("Non-first element missing prior element")
        }
    }
}

/// For a given node, walk background until a non-comment or blank line is
/// found. This allows us to skip comments that are "attached" to the current
/// node.
fn skip_preceding_comments(syntax: &SyntaxNode) -> NodeOrToken<SyntaxNode, SyntaxToken> {
    let mut preceding_comments = Vec::new();

    let mut prev = syntax.prev_sibling_or_token();
    while let Some(cur) = prev {
        match cur.kind() {
            SyntaxKind::Comment => {
                // Ensure this comment "belongs" to the root element.
                // A preceding comment on a blank line is considered to belong to the element.
                // Othewise, the comment "belongs" to whatever
                // else is on that line.
                if let Some(before_cur) = cur.prev_sibling_or_token() {
                    match before_cur.kind() {
                        SyntaxKind::Whitespace => {
                            if before_cur.to_string().contains('\n') {
                                // The 'cur' comment is on is on its own line.
                                // It "belongs" to the current element.
                                preceding_comments.push(cur.clone());
                            }
                        }
                        _ => {
                            // The 'cur' comment is on the same line as this
                            // token. It "belongs"
                            // to whatever is currently being processed.
                        }
                    }
                }
            }
            SyntaxKind::Whitespace => {
                // Ignore
                if cur.to_string().chars().filter(|c| *c == '\n').count() > 1 {
                    // We've backed up to an empty line, so we can stop
                    break;
                }
            }
            _ => {
                // We've backed up to non-trivia, so we can stop
                break;
            }
        }
        prev = cur.prev_sibling_or_token()
    }

    return preceding_comments
        .last()
        .unwrap_or(&NodeOrToken::Node(syntax.clone()))
        .clone();
}
