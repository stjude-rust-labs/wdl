use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::SyntaxElement;
use wdl_ast::Token;
use wdl_ast::WorkflowDescriptionLanguage;

use crate::TokenStream;
use crate::Writable;

type DynAstNode<'a> = &'a dyn AstNode<Language = WorkflowDescriptionLanguage>;
type DynAstToken<'a> = &'a dyn AstToken;

pub struct FormatNode<'a>(&'a dyn DynAstNode);

impl<'a> FormatNode<'a> {
    pub fn new<T: AstNode<Language = WorkflowDescriptionLanguage> + 'a>(value: &'a T) -> Self {
        Self(value as DynAstNode)
    }

    pub fn collate(&self) -> FormatElement<'_> {}
}

pub trait AstNodeFormatExt: AstNode<Language = WorkflowDescriptionLanguage> {
    fn as_format_node(&self) -> FormatNode<'_>
    where
        Self: Sized,
    {
        FormatNode::new(self)
    }
}

impl<T: AstNode<Language = WorkflowDescriptionLanguage>> AstNodeFormatExt for T {}

pub struct FormatToken<'a>(DynAstToken<'a>);

impl<'a> FormatToken<'a> {
    pub fn new<T: AstToken + 'a>(value: &'a T) -> Self {
        Self(value as DynAstToken)
    }
}

pub trait AstTokenFormatExt: AstToken {
    fn as_format_token(&self) -> FormatToken<'_>
    where
        Self: Sized,
    {
        FormatToken::new(self)
    }
}

impl<T: AstToken> AstTokenFormatExt for T {}

impl<'a> Writable<'a> for FormatToken<'a> {
    fn write(&self, stream: &mut TokenStream<'a>) {
        stream.write(self.0.as_str());
    }
}

pub enum FormatElement<'a> {
    Node(FormatNode<'a>),
    Token(FormatToken<'a>),
}

impl From<Token> for FormatElement<'_> {
    fn from(value: Token) -> Self {}
}

#[cfg(test)]
mod tests {
    use wdl_ast::Document;

    use crate::node::AstNodeFormatExt as _;

    #[test]
    fn smoke() {
        let (document, diagnostics) = Document::parse(
            "version 1.2

# This is a comment attached to the task.
task foo # This is an inline comment.
{

}

# This is a comment attached to the workflow.
workflow bar # This is inline with the workflow
{
  # This is attached to the call.
  call foo {}
}",
        );

        assert!(diagnostics.is_empty());

        let ast = document.ast();
        let ast = ast.as_v1().unwrap();
        let node = ast.as_format_node();
    }
}
