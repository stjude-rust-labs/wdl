// ... existing code ...

use std::collections::HashMap;
use std::sync::OnceLock;

static TOKEN_PATTERNS: OnceLock<HashMap<TokenKind, &'static str>> = OnceLock::new();

fn get_token_patterns() -> &'static HashMap<TokenKind, &'static str> {
    TOKEN_PATTERNS.get_or_init(|| {
        let mut patterns = HashMap::new();
        patterns.insert(TokenKind::Identifier, r"[a-zA-Z][a-zA-Z0-9_]*");
        patterns.insert(TokenKind::IntegerLiteral, r"[0-9]+");
        // ... other patterns ...
        patterns
    })
}

impl TokenKind {
    pub fn pattern(&self) -> &'static str {
        get_token_patterns().get(self).unwrap_or(&"")
    }
    
    // ... existing methods ...
}

// ... existing code ...