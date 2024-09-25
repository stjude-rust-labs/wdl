//! Validation of string literals in an AST.

use rowan::ast::AstChildren;
use rowan::ast::AstNode;
use rowan::ast::support::children;
use wdl_grammar::ToSpan;
use wdl_grammar::lexer::v1::EscapeToken;
use wdl_grammar::lexer::v1::Logos;

use crate::AstToken;
use crate::Diagnostic;
use crate::Diagnostics;
use crate::Document;
use crate::Span;
use crate::SupportedVersion;
use crate::VisitReason;
use crate::Visitor;
use crate::v1;
use crate::v1::LiteralStringKind;
use crate::v1::PlaceholderOption;

/// Creates an "unknown escape sequence" diagnostic
fn unknown_escape_sequence(sequence: &str, span: Span) -> Diagnostic {
    Diagnostic::error(format!("unknown escape sequence `{sequence}`"))
        .with_label("this is not a valid WDL escape sequence", span)
}

/// Creates an "invalid line continuation" diagnostic
fn invalid_line_continuation(span: Span) -> Diagnostic {
    Diagnostic::error("literal strings may not contain line continuations")
        .with_label("remove this line continuation", span)
}

/// Creates an "invalid octal escape" diagnostic
fn invalid_octal_escape(span: Span) -> Diagnostic {
    Diagnostic::error("invalid octal escape sequence").with_label(
        "expected a sequence of three octal digits to follow this",
        span,
    )
}

/// Creates an "invalid hex escape" diagnostic
fn invalid_hex_escape(span: Span) -> Diagnostic {
    Diagnostic::error("invalid hex escape sequence").with_label(
        "expected a sequence of two hexadecimal digits to follow this",
        span,
    )
}

/// Creates an "invalid short unicode escape" diagnostic
fn invalid_short_unicode_escape(span: Span) -> Diagnostic {
    Diagnostic::error("invalid unicode escape sequence").with_label(
        "expected a sequence of four hexadecimal digits to follow this",
        span,
    )
}

/// Creates an "invalid unicode escape" diagnostic
fn invalid_unicode_escape(span: Span) -> Diagnostic {
    Diagnostic::error("invalid unicode escape sequence").with_label(
        "expected a sequence of eight hexadecimal digits to follow this",
        span,
    )
}

/// Creates a "must escape newline" diagnostic
fn must_escape_newline(span: Span) -> Diagnostic {
    Diagnostic::error("literal strings cannot contain newline characters")
        .with_label("escape this newline with `\\n`", span)
}

/// Creates a "must escape tab" diagnostic
fn must_escape_tab(span: Span) -> Diagnostic {
    Diagnostic::error("literal strings cannot contain tab characters")
        .with_label("escape this tab with `\\t`", span)
}

/// Creates a "multiple placeholder options" diagnostic.
fn multiple_placeholder_options(first: Span, additional: Span) -> Diagnostic {
    Diagnostic::error("a placeholder cannot have more than one option")
        .with_label("duplicate placeholder option is here", additional)
        .with_label("first placeholder option is here", first)
}

/// Used to check literal text in a string.
fn check_text(diagnostics: &mut Diagnostics, start: usize, text: &str) {
    let lexer = EscapeToken::lexer(text).spanned();
    for (token, span) in lexer {
        match token.expect("should lex") {
            EscapeToken::Valid
            | EscapeToken::ValidOctal
            | EscapeToken::ValidHex
            | EscapeToken::ValidUnicode
            | EscapeToken::Text => continue,
            EscapeToken::InvalidOctal => {
                diagnostics.add(invalid_octal_escape(Span::new(start + span.start, 1)))
            }
            EscapeToken::InvalidHex => diagnostics.add(invalid_hex_escape(Span::new(
                start + span.start,
                span.len(),
            ))),
            EscapeToken::InvalidShortUnicode => diagnostics.add(invalid_short_unicode_escape(
                Span::new(start + span.start, span.len()),
            )),
            EscapeToken::InvalidUnicode => diagnostics.add(invalid_unicode_escape(Span::new(
                start + span.start,
                span.len(),
            ))),
            EscapeToken::Continuation => diagnostics.add(invalid_line_continuation(Span::new(
                start + span.start,
                span.len(),
            ))),
            EscapeToken::Newline => diagnostics.add(must_escape_newline(Span::new(
                start + span.start,
                span.len(),
            ))),
            EscapeToken::Tab => {
                diagnostics.add(must_escape_tab(Span::new(start + span.start, span.len())))
            }
            EscapeToken::Unknown => diagnostics.add(unknown_escape_sequence(
                &text[span.start..span.end],
                Span::new(start + span.start, span.len()),
            )),
        }
    }
}

/// A visitor of literal text within an AST.
///
/// Ensures that string text:
///
/// * Does not contain characters that must be escaped.
/// * Does not contain invalid escape sequences.
/// * Strings and command placeholders do not contain more than one option.
#[derive(Default, Debug)]
pub struct LiteralTextVisitor;

impl Visitor for LiteralTextVisitor {
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

    fn string_text(&mut self, state: &mut Self::State, text: &v1::StringText) {
        let string = v1::LiteralString::cast(text.syntax().parent().expect("should have a parent"))
            .expect("node should cast");
        match string.kind() {
            LiteralStringKind::SingleQuoted | LiteralStringKind::DoubleQuoted => {
                // Check the text of a normal string to ensure escape sequences are correct and
                // characters that are required to be escaped are actually escaped.
                check_text(
                    state,
                    text.syntax().text_range().start().into(),
                    text.as_str(),
                );
            }
            LiteralStringKind::Multiline => {
                // Don't check the text of multiline strings as they are treated
                // like commands where almost all of the text is literal and the
                // only escape is escaping the closing `>>>`; the only
                // difference between a multiline string and a command is how
                // line continuation whitespace is normalized.
            }
        }
    }

    fn placeholder(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        placeholder: &v1::Placeholder,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        let mut placeholders: AstChildren<PlaceholderOption> = children(placeholder.syntax());
        if let Some(first) = placeholders.next() {
            for additional in placeholders {
                state.add(multiple_placeholder_options(
                    first.syntax().text_range().to_span(),
                    additional.syntax().text_range().to_span(),
                ));
            }
        }
    }
}
