//! Contains the `Formatter` struct, which is used to keep track of the
//! current formatting state. This includes the current indentation level and
//! whether the current line has been interrupted by comments.
//! The state becomes "interrupted" by comments when a comment forces a newline
//! where it would otherwise not be expected. In this case, the next line(s)
//! will be indented by one level.

use crate::Formattable;
use crate::NEWLINE;

/// Space constant used for formatting.
pub const SPACE: &str = " ";
/// Indentation constant used for formatting. Indentation is four spaces
/// per-level.
pub const INDENT: &str = "    ";
/// Inline comment space constant used for formatting.
///
/// Inline comments should start two spaces after the end of the element they
/// are commenting on.
pub const INLINE_COMMENT_SPACE: &str = "  ";

/// The `Formatter` struct is used to keep track of the current formatting
/// state. This includes the current indentation level and whether the current
/// line has been interrupted by comments.
#[derive(Debug, Clone, Copy, Default)]
pub struct Formatter {
    /// The current indentation level.
    indent_level: usize,
    /// Whether the current line has been interrupted by comments.
    interrupted_by_comments: bool,
}

impl Formatter {
    /// Format an element.
    pub fn format<F: std::fmt::Write, T: Formattable>(
        mut self,
        element: &T,
        writer: &mut F,
    ) -> std::fmt::Result {
        element.format(writer, &mut self)
    }

    /// Add the current indentation to the writer.
    /// The indentation level will be temporarily increased by one if the
    /// current line has been interrupted by comments.
    pub fn indent<T: std::fmt::Write>(&self, writer: &mut T) -> std::fmt::Result {
        write!(
            writer,
            "{}",
            INDENT.repeat(self.indent_level + (if self.interrupted_by_comments { 1 } else { 0 }))
        )
    }

    /// Add a space or an indentation to the writer. If the current line has
    /// been interrupted by comments, an indentation is added. Otherwise, a
    /// space is added.
    pub fn space_or_indent<T: std::fmt::Write>(&mut self, writer: &mut T) -> std::fmt::Result {
        if !self.interrupted_by_comments {
            write!(writer, "{}", SPACE)?;
        } else {
            self.indent(writer)?;
        }
        self.reset_interrupted();
        Ok(())
    }

    /// Add a level of indentation.
    pub fn increment_indent(&mut self) {
        self.indent_level += 1;
        self.reset_interrupted();
    }

    /// Remove a level of indentation.
    pub fn decrement_indent(&mut self) {
        self.indent_level = self.indent_level.saturating_sub(1);
        self.reset_interrupted();
    }

    /// Check if the current line has been interrupted by comments.
    pub fn interrupted(&self) -> bool {
        self.interrupted_by_comments
    }

    /// Interrupt the current line with comments.
    pub fn interrupt(&mut self) {
        self.interrupted_by_comments = true;
    }

    /// Reset the interrupted state.
    pub fn reset_interrupted(&mut self) {
        self.interrupted_by_comments = false;
    }

    pub fn format_preceding_trivia<F: std::fmt::Write>(
        &mut self,
        writer: &mut F,
        comments: Box<[String]>,
        would_be_interrupting: bool,
        respect_blank_lines: bool,
    ) -> std::fmt::Result {
        if would_be_interrupting && !comments.is_empty() && !self.interrupted_by_comments {
            write!(writer, "{}", NEWLINE)?;
            self.interrupt();
        }
        for comment in comments {
            if !respect_blank_lines && !comment.starts_with('#') {
                continue;
            }
            self.indent(writer)?;
            write!(writer, "{}{}", comment, NEWLINE)?;
        }
        Ok(())
    }

    pub fn format_inline_comment<F: std::fmt::Write>(
        &mut self,
        writer: &mut F,
        comment: Option<String>,
        would_be_interrupting: bool,
    ) -> std::fmt::Result {
        if let Some(ref comment) = comment {
            write!(writer, "{}{}{}", INLINE_COMMENT_SPACE, comment, NEWLINE)?;
        }
        if would_be_interrupting && comment.is_some() {
            self.interrupt();
        } else if !would_be_interrupting && comment.is_none() {
            write!(writer, "{}", NEWLINE)?;
        }
        Ok(())
    }
}
