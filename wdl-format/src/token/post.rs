//! Postprocessed tokens.
//!
//! Generally speaking, unless you are working with the internals of code
//! formatting, you're not going to be working with these.

use wdl_ast::SyntaxKind;

use crate::NEWLINE;
use crate::PreToken;
use crate::SPACE;
use crate::Token;
use crate::TokenStream;

/// A postprocessed token.
///
/// Note that this will transformed into a [`TokenStream`](super::TokenStream)
/// of [`PostToken`](super::PostToken)s by a
/// [`Postprocessor`](super::Postprocessor) (authors of elements are never
/// expected to write [`PostToken`](super::PostToken)s directly).
#[derive(Eq, PartialEq)]
pub enum PostToken {
    /// A space.
    Space,

    /// A newline.
    Newline,

    /// A string literal.
    Literal(String),
}

impl std::fmt::Debug for PostToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Space => write!(f, "<SPACE>"),
            Self::Newline => write!(f, "<NEWLINE>"),
            Self::Literal(value) => write!(f, "<LITERAL> {value}"),
        }
    }
}

impl std::fmt::Display for PostToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PostToken::Space => write!(f, "{SPACE}"),
            PostToken::Newline => write!(f, "{NEWLINE}"),
            PostToken::Literal(value) => write!(f, "{value}"),
        }
    }
}

impl Token for PostToken {}

/// The state of the postprocessor.
#[derive(Default, Eq, PartialEq)]
enum State {
    /// The start of a line in the document.
    #[default]
    StartOfLine,

    /// The middle of a line.
    MiddleOfLine,
}

/// A postprocessor of [tokens](PreToken).
#[derive(Default)]
pub struct Postprocessor(State);

impl Postprocessor {
    /// Runs the postprocessor.
    pub fn run(&mut self, input: TokenStream<PreToken>) -> TokenStream<PostToken> {
        let mut output = TokenStream::<PostToken>::default();

        for token in input {
            self.step(token, &mut output)
        }

        output.trim_while(|token| matches!(token, PostToken::Space | PostToken::Newline));
        output.push(PostToken::Newline);

        output
    }

    /// Takes a step of a [`PreToken`] stream and processes the appropriate
    /// [`PostToken`]s.
    pub fn step(&mut self, token: PreToken, stream: &mut TokenStream<PostToken>) {
        match token {
            PreToken::SectionSpacer => {
                if self.0 != State::StartOfLine {
                    self.newline(stream)
                }

                self.newline(stream);
            }
            PreToken::Literal(value, kind) => {
                match self.0 {
                    State::StartOfLine | State::MiddleOfLine => {
                        stream.push(PostToken::Literal(value));
                    }
                }

                if kind == SyntaxKind::Comment {
                    self.newline(stream);
                } else {
                    stream.push(PostToken::Space);
                    self.0 = State::MiddleOfLine;
                }
            }
        }
    }

    /// Adds a newline to the stream and modifies the state accordingly.
    fn newline(&mut self, stream: &mut TokenStream<PostToken>) {
        stream.trim_end(&PostToken::Space);
        stream.push(PostToken::Newline);
        self.0 = State::StartOfLine;
    }
}
