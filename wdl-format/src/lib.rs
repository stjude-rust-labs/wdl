//! Formatting facilities for WDL.

pub mod config;
pub mod element;
mod token;
pub mod v1;

use std::fmt::Write;

pub use config::Config;
pub use token::*;
use wdl_ast::Element;
use wdl_ast::Node as AstNode;

use crate::element::FormatElement;

/// Newline constant used for formatting on windows platforms.
#[cfg(windows)]
pub const NEWLINE: &str = "\r\n";
/// Newline constant used for formatting on non-windows platforms.
#[cfg(not(windows))]
pub const NEWLINE: &str = "\n";

/// A space.
pub const SPACE: &str = " ";

/// Returns exactly one entity from an enumerable list of entities (usually a
/// [`Vec`]).
#[macro_export]
macro_rules! exactly_one {
    ($entities:expr, $name:expr) => {
        match $entities.len() {
            0 => unreachable!("we should never have zero {}", $name),
            // SAFETY: we just checked to ensure that exactly
            // one element exists, so this will always unwrap.
            1 => $entities.pop().unwrap(),
            _ => unreachable!("we should not have two or more {}", $name),
        }
    };
}

/// An element that can be written to a token stream.
pub trait Writable {
    /// Writes the element to the token stream.
    fn write(&self, stream: &mut TokenStream<PreToken>);
}

impl Writable for &FormatElement {
    fn write(&self, stream: &mut TokenStream<PreToken>) {
        match self.element() {
            Element::Node(node) => match node {
                AstNode::AccessExpr(_) => todo!(),
                AstNode::AdditionExpr(_) => todo!(),
                AstNode::ArrayType(_) => todo!(),
                AstNode::Ast(_) => v1::format_ast(self, stream),
                AstNode::BoundDecl(_) => todo!(),
                AstNode::CallAfter(_) => todo!(),
                AstNode::CallAlias(_) => todo!(),
                AstNode::CallExpr(_) => todo!(),
                AstNode::CallInputItem(_) => todo!(),
                AstNode::CallStatement(_) => {
                    v1::workflow::call::format_call_statement(self, stream)
                }
                AstNode::CallTarget(_) => v1::workflow::call::format_call_target(self, stream),
                AstNode::CommandSection(_) => todo!(),
                AstNode::ConditionalStatement(_) => todo!(),
                AstNode::DefaultOption(_) => todo!(),
                AstNode::DivisionExpr(_) => todo!(),
                AstNode::EqualityExpr(_) => todo!(),
                AstNode::ExponentiationExpr(_) => todo!(),
                AstNode::GreaterEqualExpr(_) => todo!(),
                AstNode::GreaterExpr(_) => todo!(),
                AstNode::IfExpr(_) => todo!(),
                AstNode::ImportAlias(_) => todo!(),
                AstNode::ImportStatement(_) => todo!(),
                AstNode::IndexExpr(_) => todo!(),
                AstNode::InequalityExpr(_) => todo!(),
                AstNode::InputSection(_) => todo!(),
                AstNode::LessEqualExpr(_) => todo!(),
                AstNode::LessExpr(_) => todo!(),
                AstNode::LiteralArray(_) => todo!(),
                AstNode::LiteralBoolean(_) => todo!(),
                AstNode::LiteralFloat(_) => todo!(),
                AstNode::LiteralHints(_) => todo!(),
                AstNode::LiteralHintsItem(_) => todo!(),
                AstNode::LiteralInput(_) => todo!(),
                AstNode::LiteralInputItem(_) => todo!(),
                AstNode::LiteralInteger(_) => todo!(),
                AstNode::LiteralMap(_) => todo!(),
                AstNode::LiteralMapItem(_) => todo!(),
                AstNode::LiteralNone(_) => todo!(),
                AstNode::LiteralNull(_) => todo!(),
                AstNode::LiteralObject(_) => todo!(),
                AstNode::LiteralObjectItem(_) => todo!(),
                AstNode::LiteralOutput(_) => todo!(),
                AstNode::LiteralOutputItem(_) => todo!(),
                AstNode::LiteralPair(_) => todo!(),
                AstNode::LiteralString(_) => todo!(),
                AstNode::LiteralStruct(_) => todo!(),
                AstNode::LiteralStructItem(_) => todo!(),
                AstNode::LogicalAndExpr(_) => todo!(),
                AstNode::LogicalNotExpr(_) => todo!(),
                AstNode::LogicalOrExpr(_) => todo!(),
                AstNode::MapType(_) => todo!(),
                AstNode::MetadataArray(_) => todo!(),
                AstNode::MetadataObject(_) => todo!(),
                AstNode::MetadataObjectItem(_) => todo!(),
                AstNode::MetadataSection(_) => todo!(),
                AstNode::ModuloExpr(_) => todo!(),
                AstNode::MultiplicationExpr(_) => todo!(),
                AstNode::NameRef(_) => todo!(),
                AstNode::NegationExpr(_) => todo!(),
                AstNode::OutputSection(_) => todo!(),
                AstNode::PairType(_) => todo!(),
                AstNode::ObjectType(_) => todo!(),
                AstNode::ParameterMetadataSection(_) => todo!(),
                AstNode::ParenthesizedExpr(_) => todo!(),
                AstNode::Placeholder(_) => todo!(),
                AstNode::PrimitiveType(_) => todo!(),
                AstNode::RequirementsItem(_) => todo!(),
                AstNode::RequirementsSection(_) => todo!(),
                AstNode::RuntimeItem(_) => todo!(),
                AstNode::RuntimeSection(_) => todo!(),
                AstNode::ScatterStatement(_) => todo!(),
                AstNode::SepOption(_) => todo!(),
                AstNode::StructDefinition(_) => todo!(),
                AstNode::SubtractionExpr(_) => todo!(),
                AstNode::TaskDefinition(_) => v1::task::format_task_definition(self, stream),
                AstNode::TaskHintsItem(_) => todo!(),
                AstNode::TaskHintsSection(_) => todo!(),
                AstNode::TrueFalseOption(_) => todo!(),
                AstNode::TypeRef(_) => todo!(),
                AstNode::UnboundDecl(_) => todo!(),
                AstNode::VersionStatement(_) => v1::format_version_statement(self, stream),
                AstNode::WorkflowDefinition(_) => {
                    v1::workflow::format_workflow_definition(self, stream)
                }
                AstNode::WorkflowHintsItem(_) => todo!(),
                AstNode::WorkflowHintsSection(_) => todo!(),
            },
            Element::Token(token) => {
                stream.push_ast_token(token);
            }
        }
    }
}

/// A formatter.
#[derive(Debug, Default)]
pub struct Formatter {
    /// The configuration.
    config: Config,
}

impl Formatter {
    /// Creates a new formatter.
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Gets the configuration for this formatter.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Formats an element.
    pub fn format<W: Writable>(&self, element: W) -> std::result::Result<String, std::fmt::Error> {
        let mut result = String::new();

        for token in self.to_stream(element) {
            write!(result, "{token}")?;
        }

        Ok(result)
    }

    /// Gets the [`PostToken`] stream.
    ///
    /// # Notes
    ///
    /// * This shouldn't be exposed publicly.
    fn to_stream<W: Writable>(&self, element: W) -> TokenStream<PostToken> {
        let mut stream = TokenStream::default();
        element.write(&mut stream);

        let mut postprocessor = Postprocessor::default();
        postprocessor.run(stream)
    }
}

#[cfg(test)]
mod tests {
    use wdl_ast::Document;
    use wdl_ast::Node;

    use crate::Formatter;
    use crate::element::node::AstNodeFormatExt as _;

    #[test]
    fn smoke() {
        let (document, diagnostics) = Document::parse(
            "version 1.2

# This is a comment attached to the task.
task foo # This is an inline comment on the task ident.
{

} # This is an inline comment on the task.

# This is a comment attached to the workflow.
workflow bar # This is an inline comment on the workflow ident.
{
  # This is attached to the call.
  call foo {}
} # This is an inline comment on the workflow.",
        );

        assert!(diagnostics.is_empty());
        let document = Node::Ast(document.ast().into_v1().unwrap()).into_format_element();
        let stream = Formatter::default().to_stream(&document).to_string();
        println!("{stream}");
    }
}
