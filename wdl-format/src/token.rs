//! Tokens used during formatting.

mod post;
mod pre;

use std::fmt::Display;

pub use post::*;
pub use pre::*;

/// Tokens that are streamable.
pub trait Token: Display + Eq + PartialEq {}

/// A stream of tokens. Tokens in this case are either [`PreToken`]s or
/// [`PostToken`]s. Note that, unless you are working on formatting
/// specifically, you should never need to work with [`PostToken`]s.
#[derive(Debug)]

pub struct TokenStream<T: Token>(Vec<T>);

impl<T: Token> Default for TokenStream<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: Token> std::fmt::Display for TokenStream<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for value in &self.0 {
            write!(f, "{value}")?;
        }

        Ok(())
    }
}

impl<T: Token> TokenStream<T> {
    /// Pushes a token into the stream.
    pub fn push(&mut self, token: T) {
        self.0.push(token);
    }

    /// Removes any number of `token`s at the end of the stream.
    pub fn trim_end(&mut self, token: &T) {
        while Some(token) == self.0.last() {
            let _ = self.0.pop();
        }
    }

    /// Removes any number of `token`s at the end of the stream.
    pub fn trim_while<F: Fn(&T) -> bool>(&mut self, predicate: F) {
        while let Some(token) = self.0.last() {
            if !predicate(token) {
                break;
            }

            let _ = self.0.pop();
        }
    }
}

impl<T: Token> IntoIterator for TokenStream<T> {
    type IntoIter = std::vec::IntoIter<Self::Item>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
