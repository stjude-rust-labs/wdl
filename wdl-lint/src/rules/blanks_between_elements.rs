//! A lint rule for blank spacing between elements.

use wdl_ast::v1::InputSection;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::Comment;
use wdl_ast::Diagnostic;
use wdl_ast::Diagnostics;
use wdl_ast::Direction;
use wdl_ast::Document;
use wdl_ast::Span;
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
    /// Are we in a `struct` definition?
    struct_definition: bool,
    /// Are we in a `scatter` statement?
    scatter_statement: bool,
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

    fn document(&mut self, state: &mut Self::State, reason: VisitReason, doc: &Document) {
        if reason == VisitReason::Exit {
            return;
        }

        if reason == VisitReason::Enter {
            // Reset the visitor upon document entry
            *self = Default::default();
        }
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

        check_prior_spacing(section.syntax(), state);
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
        check_prior_spacing(section.syntax(), state);
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

        check_prior_spacing(section.syntax(), state);
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
        check_prior_spacing(section.syntax(), state);
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
        check_prior_spacing(section.syntax(), state);
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

        let prev_token = stmt
            .syntax()
            .prev_sibling_or_token()
            .and_then(SyntaxElement::into_token);

        let between_calls = stmt
            .syntax()
            .prev_sibling()
            .map(|s| s.kind() == SyntaxKind::CallStatementNode)
            .unwrap_or(false);

        if let Some(token) = prev_token {
            if token.kind() == SyntaxKind::Whitespace {
                let count = token.to_string().chars().filter(|c| *c == '\n').count();
                // If this is the first element in the block, then no blank lines are allowed.
                if is_first_element(stmt.syntax()) && count > 1 {
                    state.add(excess_blank_line(token.text_range().to_span()));
                } else if !between_calls && count < 2 {
                    state.add(missing_blank_line(stmt.syntax().text_range().to_span()));
                }
                // If we're between calls, then blank lines are optional.
            }
        }
    }

    fn scatter_statement(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        stmt: &wdl_ast::v1::ScatterStatement,
    ) {
        if reason == VisitReason::Exit {
            return;
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

        check_prior_spacing(def.syntax(), state);
        flag_all_blanks(def.syntax(), state);
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

        check_prior_spacing(section.syntax(), state);
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

        let prior_node = decl.syntax().prev_sibling();

        let prior = decl.syntax().prev_sibling_or_token();
        if let Some(p) = prior {
            match p.kind() {
                SyntaxKind::Whitespace => {
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
                                if count < 2 {
                                    state.add(missing_blank_line(
                                        decl.syntax().text_range().to_span(),
                                    ));
                                }
                            }
                        }
                    }
                }
                _ => {}
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

        let prior_node = decl.syntax().prev_sibling();

        let prior = decl.syntax().prev_sibling_or_token();
        if let Some(p) = prior {
            match p.kind() {
                SyntaxKind::Whitespace => {
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
                                if count < 2 {
                                    state.add(missing_blank_line(
                                        decl.syntax().text_range().to_span(),
                                    ));
                                }
                            }
                        }
                    }
                }
                _ => {}
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

        check_prior_spacing(stmt.syntax(), state);
    }

    fn metadata_object(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        object: &wdl_ast::v1::MetadataObject,
    ) {
        if reason == VisitReason::Exit {
            return;
        }
    }

    // TODO: This isn't available in the wdl-ast crate yet
    // fn metadata_object_item

    // A comment needs blank space before it if the next node
    fn comment(&mut self, state: &mut Self::State, comment: &Comment) {
        if comment_is_inline(comment) {
            return;
        }
    }
}

fn comment_is_inline(comment: &Comment) -> bool {
    let mut prior = comment.syntax().prev_sibling_or_token();
    while let Some(ref cur) = prior {
        match cur.kind() {
            SyntaxKind::Whitespace => {
                let count = cur.to_string().chars().filter(|c| *c == '\n').count();
                // We've found the beginning of the line
                if count > 0 {
                    return false;
                }
            }
            _ => {
                // Something other than whitespace precedes the comment
                break;
            }
        }
        prior = cur.prev_sibling_or_token();
    }
    // If the comment is the first thing in the current block, prior will be None.
    if prior.is_none() {
        return false;
    }
    true
}

fn is_first_element(syntax: &SyntaxNode) -> bool {
    if let Some(_) = syntax.prev_sibling() {
        return false;
    }
    true
}

fn flag_all_blanks(syntax: &SyntaxNode, state: &mut Diagnostics) {
    syntax.children_with_tokens().for_each(|c| match c.kind() {
        SyntaxKind::Whitespace => {
            let count = c.to_string().chars().filter(|c| *c == '\n').count();
            if count > 0 {
                state.add(excess_blank_line(c.text_range().to_span()));
            }
        }
        _ => {}
    });
}

fn check_prior_spacing(syntax: &SyntaxNode, state: &mut Diagnostics) {
    let first = is_first_element(syntax);

    let prior = syntax.prev_sibling_or_token();
    if let Some(p) = prior {
        match p.kind() {
            SyntaxKind::Whitespace => {
                let count = p.to_string().chars().filter(|c| *c == '\n').count();
                // If this is the first element.
                if first {
                    // We should have no blank lines, so only one `\n` is allowed.
                    if count > 1 {
                        state.add(excess_blank_line(p.text_range().to_span()));
                    }
                } else {
                    // If this is not the first element, we should have a blank line.
                    if count < 2 {
                        state.add(missing_blank_line(syntax.text_range().to_span()));
                    }
                }
            }
            SyntaxKind::Comment => {}
            _ => {}
        }
    }
}
