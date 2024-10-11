//! Postprocessed tokens.
//!
//! Generally speaking, unless you are working with the internals of code
//! formatting, you're not going to be working with these.

use wdl_ast::SyntaxKind;

use crate::Comment;
use crate::LineSpacingPolicy;
use crate::NEWLINE;
use crate::PreToken;
use crate::SPACE;
use crate::Token;
use crate::TokenStream;
use crate::Trivia;

/// A postprocessed token.
#[derive(Eq, PartialEq)]
pub enum PostToken {
    /// A space.
    Space,

    /// A newline.
    Newline,

    /// One indentation.
    Indent,

    /// A string literal.
    Literal(String),
}

impl std::fmt::Debug for PostToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Space => write!(f, "<SPACE>"),
            Self::Newline => write!(f, "<NEWLINE>"),
            Self::Indent => write!(f, "<INDENT>"),
            Self::Literal(value) => write!(f, "<LITERAL> {value}"),
        }
    }
}

impl std::fmt::Display for PostToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PostToken::Space => write!(f, "{SPACE}"),
            PostToken::Newline => write!(f, "{NEWLINE}"),
            PostToken::Indent => write!(f, "    "), // 4 spaces TODO replace
            PostToken::Literal(value) => write!(f, "{value}"),
        }
    }
}

impl Token for PostToken {}

/// Current position in a line.
#[derive(Default, Eq, PartialEq)]
enum LinePosition {
    /// The start of a line.
    #[default]
    StartOfLine,

    /// The middle of a line.
    MiddleOfLine,
}

/// A postprocessor of [tokens](PreToken).
#[derive(Default)]
pub struct Postprocessor {
    /// The current position in the line.
    position: LinePosition,

    /// The current indentation level.
    indent_level: usize,

    /// Whether the current line has been interrupted by trivia.
    interrupted: bool,

    /// Whether blank lines are allowed in the current context.
    blank_lines_allowed: LineSpacingPolicy,
}

impl Postprocessor {
    /// Runs the postprocessor.
    pub fn run(&mut self, input: TokenStream<PreToken>) -> TokenStream<PostToken> {
        let mut output = TokenStream::<PostToken>::default();

        let mut stream = input.into_iter().peekable();
        while let Some(token) = stream.next() {
            self.step(token, stream.peek(), &mut output);
        }

        self.trim_whitespace(&mut output);
        output.push(PostToken::Newline);

        output
    }

    /// Takes a step of a [`PreToken`] stream and processes the appropriate
    /// [`PostToken`]s.
    pub fn step(
        &mut self,
        token: PreToken,
        next: Option<&PreToken>,
        stream: &mut TokenStream<PostToken>,
    ) {
        match token {
            PreToken::BlankLine => {
                self.blank_line(stream);
            }
            PreToken::LineEnd => {
                self.interrupted = false;
                self.end_line(stream);
            }
            PreToken::WordEnd => {
                stream.trim_end(&PostToken::Space);

                if self.position == LinePosition::MiddleOfLine {
                    stream.push(PostToken::Space);
                } else {
                    // We're at the start of a line, so we don't need to add a
                    // space.
                }
            }
            PreToken::IndentStart => {
                self.indent_level += 1;
                self.end_line(stream);
            }
            PreToken::IndentEnd => {
                self.indent_level = self.indent_level.saturating_sub(1);
                self.end_line(stream);
            }
            PreToken::LineSpacingPolicy(policy) => {
                self.blank_lines_allowed = policy;
            }
            PreToken::Literal(value, kind) => {
                assert!(kind != SyntaxKind::Comment && kind != SyntaxKind::Whitespace);
                if self.interrupted
                    && matches!(
                        kind,
                        SyntaxKind::OpenBrace
                            | SyntaxKind::OpenBracket
                            | SyntaxKind::OpenParen
                            | SyntaxKind::OpenHeredoc
                    )
                    && stream.0.last() == Some(&PostToken::Indent)
                {
                    stream.0.pop();
                }
                stream.push(PostToken::Literal(value.to_owned()));
                self.position = LinePosition::MiddleOfLine;
            }
            PreToken::Trivia(trivia) => match trivia {
                Trivia::BlankLine => {
                    if self.blank_lines_allowed == LineSpacingPolicy::Yes {
                        self.blank_line(stream);
                    } else {
                        todo!("handle line spacing policy")
                    }
                }
                Trivia::Comment(comment) => match comment {
                    Comment::Preceding(value) => {
                        if !matches!(
                            stream.0.last(),
                            Some(&PostToken::Newline) | Some(&PostToken::Indent) | None
                        ) {
                            self.interrupted = true;
                        }
                        self.end_line(stream);
                        stream.push(PostToken::Literal(value.to_owned()));
                        self.position = LinePosition::MiddleOfLine;
                        self.end_line(stream);
                    }
                    Comment::Inline(value) => {
                        assert!(self.position == LinePosition::MiddleOfLine);
                        if let Some(next) = next {
                            if next != &PreToken::LineEnd {
                                self.interrupted = true;
                            }
                        }
                        self.trim_last_line(stream);
                        stream.push(PostToken::Space);
                        stream.push(PostToken::Space);
                        stream.push(PostToken::Literal(value.to_owned()));
                        self.end_line(stream);
                    }
                },
            },
        }
    }

    /// Trims any and all whitespace from the end of the stream.
    fn trim_whitespace(&mut self, stream: &mut TokenStream<PostToken>) {
        stream.trim_while(|token| {
            matches!(
                token,
                PostToken::Space | PostToken::Newline | PostToken::Indent
            )
        });
    }

    /// Trims spaces and indents (and not newlines) from the end of the stream.
    fn trim_last_line(&mut self, stream: &mut TokenStream<PostToken>) {
        stream.trim_while(|token| matches!(token, PostToken::Space | PostToken::Indent));
    }

    /// Ends the current line without resetting the interrupted flag.
    ///
    /// Removes any trailing spaces or indents and adds a newline only if state
    /// is not [`LinePosition::StartOfLine`]. State is then set to
    /// [`LinePosition::StartOfLine`]. Safe to call multiple times in a row.
    fn end_line(&mut self, stream: &mut TokenStream<PostToken>) {
        self.trim_last_line(stream);
        if self.position != LinePosition::StartOfLine {
            stream.push(PostToken::Newline);
        }
        self.position = LinePosition::StartOfLine;
        self.indent(stream);
    }

    /// Pushes the current indentation level to the stream.
    /// This should only be called when the state is
    /// [`LinePosition::StartOfLine`].
    fn indent(&self, stream: &mut TokenStream<PostToken>) {
        assert!(self.position == LinePosition::StartOfLine);

        let level = if self.interrupted {
            self.indent_level + 1
        } else {
            self.indent_level
        };

        for _ in 0..level {
            stream.push(PostToken::Indent);
        }
    }

    /// Creates a blank line and then indents.
    fn blank_line(&mut self, stream: &mut TokenStream<PostToken>) {
        self.trim_whitespace(stream);
        stream.push(PostToken::Newline);
        stream.push(PostToken::Newline);
        self.position = LinePosition::StartOfLine;
        self.indent(stream);
    }
}
