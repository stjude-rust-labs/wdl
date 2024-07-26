//! Contains the `State` struct, which is used to keep track of the
//! current formatting state. This includes the current indentation level and
//! whether the current line has been interrupted by comments.
//! The state becomes "interrupted" by comments when a comment forces a newline
//! where it would otherwise not be expected. In this case, the next line(s)
//! will be indented by one level.
use anyhow::Result;

/// Space constant used for formatting.
pub const SPACE: &str = " ";
/// Indentation constant used for formatting.
pub const INDENT: &str = "    ";

/// The `State` struct is used to keep track of the current formatting
/// state. This includes the current indentation level and whether the current
/// line has been interrupted by comments.
#[derive(Debug, Clone, Copy, Default)]
pub struct State {
    /// The current indentation level.
    indent_level: usize,
    /// Whether the current line has been interrupted by comments.
    interrupted_by_comments: bool,
}

impl State {
    /// Add the current indentation to the writer.
    /// The indentation level will be temporarily increased by one if the
    /// current line has been interrupted by comments.
    pub fn indent<T: std::fmt::Write>(&self, writer: &mut T) -> Result<()> {
        write!(
            writer,
            "{}",
            INDENT.repeat(self.indent_level + (if self.interrupted_by_comments { 1 } else { 0 }))
        )?;
        Ok(())
    }

    /// Add a space or an indentation to the writer. If the current line has
    /// been interrupted by comments, an indentation is added. Otherwise, a
    /// space is added.
    pub fn space_or_indent<T: std::fmt::Write>(&mut self, writer: &mut T) -> Result<()> {
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
}
