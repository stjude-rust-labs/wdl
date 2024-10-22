//! Postprocessed tokens.
//!
//! Generally speaking, unless you are working with the internals of code
//! formatting, you're not going to be working with these.

use std::collections::HashSet;
use std::fmt::Display;
use std::rc::Rc;

use wdl_ast::SyntaxKind;

use crate::Comment;
use crate::Config;
use crate::LineSpacingPolicy;
use crate::NEWLINE;
use crate::PreToken;
use crate::SPACE;
use crate::Token;
use crate::TokenStream;
use crate::Trivia;
use crate::config::Indent;

/// A postprocessed token.
#[derive(Clone, Eq, PartialEq)]
pub enum PostToken {
    /// A space.
    Space,

    /// A newline.
    Newline,

    /// One indentation.
    Indent,

    /// A string literal.
    Literal(Rc<String>),
}

impl std::fmt::Debug for PostToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Space => write!(f, "<SPACE>"),
            Self::Newline => write!(f, "<NEWLINE>"),
            Self::Indent => write!(f, "<INDENT>"),
            Self::Literal(value) => write!(f, "<LITERAL@{value}>"),
        }
    }
}

impl Token for PostToken {
    /// Returns a displayable version of the token.
    fn display<'a>(&'a self, config: &'a Config) -> impl Display + 'a {
        /// A displayable version of a [`PostToken`].
        struct Display<'a> {
            /// The token to display.
            token: &'a PostToken,
            /// The configuration to use.
            config: &'a Config,
        }

        impl std::fmt::Display for Display<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self.token {
                    PostToken::Space => write!(f, "{SPACE}"),
                    PostToken::Newline => write!(f, "{NEWLINE}"),
                    PostToken::Indent => {
                        let (c, n) = match self.config.indent() {
                            Indent::Spaces(n) => (' ', n),
                            Indent::Tabs => ('\t', 1),
                        };

                        for _ in 0..n {
                            write!(f, "{c}")?;
                        }

                        Ok(())
                    }
                    PostToken::Literal(value) => write!(f, "{value}"),
                }
            }
        }

        Display {
            token: self,
            config,
        }
    }
}

impl PostToken {
    /// Gets the length of the [`PostToken`].
    fn len(&self, config: &crate::Config) -> usize {
        match self {
            Self::Space => SPACE.len(),
            Self::Newline => 0,
            Self::Indent => config.indent().num(),
            Self::Literal(value) => value.len(),
        }
    }
}

impl TokenStream<PostToken> {
    /// Gets the length of the [`TokenStream`].
    fn len(&self, config: &Config) -> usize {
        self.iter().map(|t| t.len(config)).sum()
    }
}

/// A line break.
enum LineBreak {
    /// A line break that can be inserted before a token.
    Before,
    /// A line break that can be inserted after a token.
    After,
}

/// Returns whether a token can be line broken.
fn can_be_line_broken(kind: SyntaxKind) -> Option<LineBreak> {
    match kind {
        SyntaxKind::OpenBrace
        | SyntaxKind::OpenBracket
        | SyntaxKind::OpenParen
        | SyntaxKind::OpenHeredoc => Some(LineBreak::After),
        SyntaxKind::CloseBrace
        | SyntaxKind::CloseBracket
        | SyntaxKind::CloseParen
        | SyntaxKind::CloseHeredoc => Some(LineBreak::Before),
        _ => None,
    }
}

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
    line_spacing_policy: LineSpacingPolicy,
}

impl Postprocessor {
    /// Runs the postprocessor.
    pub fn run(&mut self, input: TokenStream<PreToken>, config: &Config) -> TokenStream<PostToken> {
        let mut output = TokenStream::<PostToken>::default();
        let mut buffer = TokenStream::<PreToken>::default();

        for token in input {
            match token {
                PreToken::LineEnd => {
                    self.flush(&buffer, &mut output, config);
                    self.trim_whitespace(&mut output);
                    output.push(PostToken::Newline);

                    buffer.clear();
                    buffer.push(token);
                    self.position = LinePosition::StartOfLine;
                }
                _ => {
                    buffer.push(token);
                }
            }
        }

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
                self.line_spacing_policy = policy;
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
                stream.push(PostToken::Literal(value));
                self.position = LinePosition::MiddleOfLine;
            }
            PreToken::Trivia(trivia) => match trivia {
                Trivia::BlankLine => match self.line_spacing_policy {
                    LineSpacingPolicy::Always => {
                        self.blank_line(stream);
                    }
                    LineSpacingPolicy::BeforeComments => {
                        if matches!(next, Some(&PreToken::Trivia(Trivia::Comment(_)))) {
                            self.blank_line(stream);
                        }
                    }
                },
                Trivia::Comment(comment) => {
                    match comment {
                        Comment::Preceding(value) => {
                            if !matches!(
                                stream.0.last(),
                                Some(&PostToken::Newline) | Some(&PostToken::Indent) | None
                            ) {
                                self.interrupted = true;
                            }
                            self.end_line(stream);
                            stream.push(PostToken::Literal(value));
                            self.position = LinePosition::MiddleOfLine;
                        }
                        Comment::Inline(value) => {
                            // assert!(self.position == LinePosition::MiddleOfLine);
                            if let Some(next) = next {
                                if next != &PreToken::LineEnd {
                                    self.interrupted = true;
                                }
                            }
                            self.trim_last_line(stream);
                            stream.push(PostToken::Space);
                            stream.push(PostToken::Space);
                            stream.push(PostToken::Literal(value));
                        }
                    }
                    self.end_line(stream);
                }
            },
        }
    }

    /// Flushes the `in_stream` buffer to the `out_stream`.
    fn flush(
        &mut self,
        in_stream: &TokenStream<PreToken>,
        out_stream: &mut TokenStream<PostToken>,
        config: &Config,
    ) {
        let mut post_buffer = TokenStream::<PostToken>::default();
        let mut pre_buffer = in_stream.iter().peekable();
        while let Some(token) = pre_buffer.next() {
            let next = pre_buffer.peek().copied();
            self.step(token.clone(), next, &mut post_buffer);
        }

        if config.max_line_length().is_none()
            || post_buffer.len(config) <= config.max_line_length().unwrap()
        {
            dbg!("no line breaks needed");
            out_stream.extend(post_buffer);
            return;
        }
        let max_length = config.max_line_length().unwrap();
        dbg!("splitting line");
        dbg!("in_stream ={:#?}", &in_stream);
        dbg!("post_buffer ={:#?}", &post_buffer);

        let mut line_breaks: Vec<usize> = Vec::new();
        for (i, token) in in_stream.iter().enumerate() {
            if let PreToken::Literal(_, kind) = token {
                match can_be_line_broken(*kind) {
                    Some(LineBreak::Before) => {
                        line_breaks.push(i);
                    }
                    Some(LineBreak::After) => {
                        line_breaks.push(i + 1);
                    }
                    None => {}
                }
            }
        }
        // Deduplicate the line breaks.
        let line_breaks = line_breaks.into_iter().collect::<HashSet<usize>>();

        let mut inserted_line_breaks;
        for max_line_breaks in 1..=line_breaks.len() {
            let mut pre_buffer = in_stream.iter().enumerate().peekable();
            inserted_line_breaks = 0;
            post_buffer.clear();

            while let Some((i, token)) = pre_buffer.next() {
                if inserted_line_breaks < max_line_breaks && line_breaks.contains(&i) {
                    inserted_line_breaks += 1;
                    self.step(PreToken::LineEnd, None, &mut post_buffer);
                    // self.interrupted = true;
                }
                self.step(
                    token.clone(),
                    pre_buffer.peek().map(|(_, t)| t).copied(),
                    &mut post_buffer,
                );
            }

            let mut last_line = TokenStream::<PostToken>::default();
            post_buffer
                .iter()
                .rev()
                .take_while(|t| *t != &PostToken::Newline)
                .for_each(|t| last_line.push(t.clone()));
            if last_line.len(config) <= max_length {
                break;
            }
        }

        out_stream.extend(post_buffer);
    }

    /// Trims any and all whitespace from the end of the stream.
    fn trim_whitespace(&self, stream: &mut TokenStream<PostToken>) {
        stream.trim_while(|token| {
            matches!(
                token,
                PostToken::Space | PostToken::Newline | PostToken::Indent
            )
        });
    }

    /// Trims spaces and indents (and not newlines) from the end of the stream.
    fn trim_last_line(&self, stream: &mut TokenStream<PostToken>) {
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
    /// [`LinePosition::StartOfLine`]. This does not change the state.
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
        if !stream.is_empty() {
            stream.push(PostToken::Newline);
        }
        stream.push(PostToken::Newline);
        self.position = LinePosition::StartOfLine;
        self.indent(stream);
    }
}
