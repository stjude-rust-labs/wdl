use std::borrow::Cow;

use crate::Writable;

#[derive(Debug, Eq, PartialEq)]
pub enum Token<'a> {
    Indent,
    Dedent,
    Literal(Cow<'a, str>),
}

impl<'a> From<&'a str> for Token<'a> {
    fn from(value: &'a str) -> Self {
        Token::Literal(Cow::Borrowed(value))
    }
}

impl From<String> for Token<'_> {
    fn from(value: String) -> Self {
        Token::Literal(Cow::Owned(value))
    }
}

#[derive(Debug, Default)]
pub struct TokenStream<'a>(pub(crate) Vec<Token<'a>>);

impl<'a> TokenStream<'a> {
    pub fn indent(&mut self) {
        self.0.push(Token::Indent);
    }

    pub fn dedent(&mut self) {
        self.0.push(Token::Dedent);
    }

    pub fn write<W: Writable<'a> + 'a>(&mut self, value: W) {
        value.write(self);
    }

    pub fn indented<F: FnMut(&mut Self)>(&mut self, mut f: F) {
        // Indents the block.
        self.indent();

        // Runs the inner function.
        f(self);

        // Dedents the block.
        self.dedent();
    }

    pub fn inner(&self) -> &Vec<Token<'a>> {
        &self.0
    }

    pub fn into_inner(self) -> Vec<Token<'a>> {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::*;

    #[test]
    fn smoke() {
        let mut stream = TokenStream::default();
        stream.indented(|stream| {
            stream.write("Hello, world!");
        });

        assert_eq!(
            stream.into_inner(),
            vec![
                Token::Indent,
                Token::Literal(Cow::Owned("Hello, world!".to_string())),
                Token::Dedent
            ]
        )
    }
}
