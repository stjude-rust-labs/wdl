// ... existing code ...

use std::borrow::Cow;

impl Lexer {
    // ... existing methods ...
    
    // Optimize string handling to avoid unnecessary allocations
    pub fn get_string_value(&self, token: &Token) -> Cow<'_, str> {
        match token.kind {
            TokenKind::StringLiteral => {
                // Extract string content without unnecessary allocation
                let content = &self.source[token.span.start + 1..token.span.end - 1];
                
                // Only allocate if we need to process escape sequences
                if content.contains('\\') {
                    // Process escape sequences
                    Cow::Owned(self.process_escape_sequences(content))
                } else {
                    // No escape sequences, return as-is
                    Cow::Borrowed(content)
                }
            }
            // ... other cases ...
            _ => Cow::Borrowed(""),
        }
    }
    
    // ... existing methods ...
}

// ... existing code ...