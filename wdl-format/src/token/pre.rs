//! Tokens emitted during the formatting of particular elements.

use wdl_ast::SyntaxExt;
use wdl_ast::SyntaxKind;

use crate::Token;
use crate::TokenStream;

/// A token that can be written by elements.
///
/// These are tokens that are intended to be written directly by elements to a
/// [`TokenStream`](super::TokenStream) consisting of [`PreToken`]s. Note that
/// this will transformed into a [`TokenStream`](super::TokenStream) of
/// [`PostToken`](super::PostToken)s by a
/// [`Postprocessor`](super::Postprocessor) (authors of elements are never
/// expected to write [`PostToken`](super::PostToken)s directly).
#[derive(Debug, Eq, PartialEq)]
pub enum PreToken {
    /// A section spacer.
    SectionSpacer,

    /// Includes text literally in the output.
    Literal(String, SyntaxKind),
}

impl PreToken {
    /// Gets the [`SyntaxKind`] of the token if the token is a
    /// [`PreToken::Literal`].
    pub fn kind(&self) -> Option<&SyntaxKind> {
        match self {
            PreToken::Literal(_, kind) => Some(kind),
            _ => None,
        }
    }
}

/// The line length to use when displaying pretokens.
const DISPLAY_LINE_LENGTH: usize = 88;

impl std::fmt::Display for PreToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PreToken::SectionSpacer => write!(f, "{}<SPACER>", " ".repeat(DISPLAY_LINE_LENGTH)),
            PreToken::Literal(value, kind) => {
                write!(
                    f,
                    "{:width$}<Literal@{:?}>",
                    value,
                    kind,
                    width = DISPLAY_LINE_LENGTH
                )
            }
        }
    }
}

impl Token for PreToken {}

impl TokenStream<PreToken> {
    /// Inserts an element spacer to the stream.
    pub fn section_spacer(&mut self) {
        self.0.push(PreToken::SectionSpacer);
    }

    /// Pushes an AST token into the stream.
    ///
    /// This will also push any preceding or inline trivia into the stream.
    /// Any token may have preceding or inline trivia, unless that token is
    /// itself trivia (i.e. trivia cannot have trivia).
    pub fn push_ast_token(&mut self, token: &wdl_ast::Token) {
        let syntax = token.syntax();
        let kind = syntax.kind();
        let mut inline_comment = None;
        if !kind.is_trivia() {
            let preceding_trivia = syntax.preceding_trivia();
            for token in preceding_trivia {
                let trivia = match token.kind() {
                    SyntaxKind::Whitespace => {
                        PreToken::Literal(String::from("\n"), SyntaxKind::Whitespace)
                    }
                    SyntaxKind::Comment => {
                        PreToken::Literal(token.text().to_owned(), SyntaxKind::Comment)
                    }
                    _ => unreachable!("unexpected trivia: {:?}", token),
                };
                self.0.push(trivia);
            }
            if let Some(token) = syntax.inline_comment() {
                inline_comment = Some(PreToken::Literal(
                    token.text().to_owned(),
                    SyntaxKind::Comment,
                ));
            }
        }
        let token = PreToken::Literal(syntax.text().to_owned(), kind);
        self.0.push(token);

        if let Some(inline_comment) = inline_comment {
            self.0.push(inline_comment);
        }
    }

    /// Gets an iterator of references to each token in the stream.
    pub fn iter(&self) -> impl Iterator<Item = &PreToken> {
        self.0.iter()
    }
}
