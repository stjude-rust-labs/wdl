use std::fmt::Write;

use anyhow::Result;

/// Space constant used for formatting.
pub const SPACE: &str = " ";
/// Indentation constant used for formatting.
pub const INDENT: &str = "    ";

pub struct FormatState {
    indent_level: usize,
    interrupted_by_comments: bool,
}

impl Default for FormatState {
    fn default() -> Self {
        FormatState {
            indent_level: 0,
            interrupted_by_comments: false,
        }
    }
}

impl FormatState {
    pub fn indent(&self, buffer: &mut String) -> Result<()> {
        let indent =
            INDENT.repeat(self.indent_level + (if self.interrupted_by_comments { 1 } else { 0 }));
        write!(buffer, "{}", indent)?;
        Ok(())
    }

    pub fn space_or_indent(&mut self, buffer: &mut String) -> Result<()> {
        if !self.interrupted_by_comments {
            write!(buffer, "{}", SPACE)?;
        } else {
            self.indent(buffer)?;
        }
        self.reset_interrupted();
        Ok(())
    }

    pub fn increment_indent(&mut self) {
        self.indent_level += 1;
    }

    pub fn decrement_indent(&mut self) {
        self.indent_level = self.indent_level.saturating_sub(1);
    }

    pub fn interrupted(&self) -> bool {
        self.interrupted_by_comments
    }

    pub fn interrupt(&mut self) {
        self.interrupted_by_comments = true;
    }

    pub fn reset_interrupted(&mut self) {
        self.interrupted_by_comments = false;
    }
}
