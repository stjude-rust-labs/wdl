//! Tokens emitted during the formatting of particular elements.

use wdl_ast::SyntaxKind;
use wdl_ast::SyntaxTokenExt;

use crate::Token;
use crate::TokenStream;

/// The kind of comment.
#[derive(Debug, Eq, PartialEq)]
pub enum CommentKind {
    /// A comment on it's own line, indented to the same level as the code
    /// following it.
    Preceding,

    /// A comment on the same line as the code preceding it.
    Inline,
}

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
    /// The end of a section.
    BlankLine,

    /// The end of a line.
    LineEnd,

    /// The end of a word.
    WordEnd,

    /// The start of an indented block.
    IndentStart,

    /// The end of an indented block.
    IndentEnd,

    /// Literal text.
    Literal(String, SyntaxKind),

    /// A comment.
    Comment(String, CommentKind),
}

impl PreToken {
    /// Gets the [`SyntaxKind`] of the token if the token is a
    /// [`PreToken::Literal`].
    pub fn kind(&self) -> Option<&SyntaxKind> {
        match self {
            PreToken::BlankLine => None,
            PreToken::LineEnd => None,
            PreToken::WordEnd => None,
            PreToken::IndentStart => None,
            PreToken::IndentEnd => None,
            PreToken::Literal(_, kind) => Some(kind),
            PreToken::Comment(..) => None,
        }
    }
}

/// The line length to use when displaying pretokens.
const DISPLAY_LINE_LENGTH: usize = 90;

impl std::fmt::Display for PreToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PreToken::BlankLine => write!(f, "{}<BlankLine>", " ".repeat(DISPLAY_LINE_LENGTH)),
            PreToken::LineEnd => write!(f, "<EOL>"),
            PreToken::WordEnd => write!(f, "<WordEnd>"),
            PreToken::IndentStart => write!(f, "<IndentStart>"),
            PreToken::IndentEnd => write!(f, "<IndentEnd>"),
            PreToken::Literal(value, kind) => {
                write!(
                    f,
                    "{:width$}<Literal@{:?}>",
                    value,
                    kind,
                    width = DISPLAY_LINE_LENGTH
                )
            }
            PreToken::Comment(value, kind) => {
                let kind = match kind {
                    CommentKind::Preceding => "Preceding",
                    CommentKind::Inline => "Inline",
                };
                write!(
                    f,
                    "{:width$}<Comment@{:?}>",
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
    /// Inserts a blank line token to the stream if the stream does not already
    /// end with a blank line. Multiple blank lines are not allowed.
    pub fn blank_line(&mut self) {
        self.trim_end(&PreToken::BlankLine);
        self.0.push(PreToken::BlankLine);
    }

    /// Inserts an end of line token to the stream if the stream does not
    /// already end with an end of line token.
    pub fn end_line(&mut self) {
        self.trim_end(&PreToken::LineEnd);
        self.0.push(PreToken::LineEnd);
    }

    /// Inserts a word end token to the stream if the stream does not already
    /// end with a word end token.
    pub fn end_word(&mut self) {
        self.trim_end(&PreToken::WordEnd);
        self.0.push(PreToken::WordEnd);
    }

    /// Inserts an indent start token to the stream.
    pub fn increment_indent(&mut self) {
        self.0.push(PreToken::IndentStart);
    }

    /// Inserts an indent end token to the stream.
    pub fn decrement_indent(&mut self) {
        self.0.push(PreToken::IndentEnd);
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
                match token.kind() {
                    SyntaxKind::Whitespace => self.blank_line(),
                    SyntaxKind::Comment => {
                        let comment = PreToken::Comment(
                            token.text().trim_end().to_owned(),
                            CommentKind::Preceding,
                        );
                        self.0.push(comment);
                    }
                    _ => unreachable!("unexpected trivia: {:?}", token),
                };
            }
            if let Some(token) = syntax.inline_comment() {
                inline_comment = Some(PreToken::Comment(
                    token.text().trim_end().to_owned(),
                    CommentKind::Inline,
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
