//! Module for the concrete syntax tree (CST) representation.

pub mod dive;

use std::borrow::Cow;
use std::collections::VecDeque;
use std::fmt;

use rowan::Direction;
use rowan::GreenNodeBuilder;
use rowan::GreenNodeData;
use strum::VariantArray;

use super::Diagnostic;
use super::grammar;
use super::lexer::Lexer;
use super::parser::Event;
use crate::parser::Parser;

/// Represents the kind of syntax element (node or token) in a WDL concrete
/// syntax tree (CST).
///
/// Nodes have at least one token child and represent a syntactic construct.
///
/// Tokens are terminal and represent any span of the source.
///
/// This enumeration is a union of all supported WDL tokens and nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, VariantArray)]
#[repr(u16)]
pub enum SyntaxKind {
    /// The token is unknown to WDL.
    Unknown,
    /// The token represents unparsed source.
    ///
    /// Unparsed source occurs in WDL source files with unsupported versions.
    Unparsed,
    /// A whitespace token.
    Whitespace,
    /// A comment token.
    Comment,
    /// A WDL version token.
    Version,
    /// A literal float token.
    Float,
    /// A literal integer token.
    Integer,
    /// An identifier token.
    Ident,
    /// A single quote token.
    SingleQuote,
    /// A double quote token.
    DoubleQuote,
    /// An open heredoc token.
    OpenHeredoc,
    /// A close heredoc token.
    CloseHeredoc,
    /// The `Array` type keyword token.
    ArrayTypeKeyword,
    /// The `Boolean` type keyword token.
    BooleanTypeKeyword,
    /// The `File` type keyword token.
    FileTypeKeyword,
    /// The `Float` type keyword token.
    FloatTypeKeyword,
    /// The `Int` type keyword token.
    IntTypeKeyword,
    /// The `Map` type keyword token.
    MapTypeKeyword,
    /// The `Object` type keyword token.
    ObjectTypeKeyword,
    /// The `Pair` type keyword token.
    PairTypeKeyword,
    /// The `String` type keyword token.
    StringTypeKeyword,
    /// The `after` keyword token.
    AfterKeyword,
    /// The `alias` keyword token.
    AliasKeyword,
    /// The `as` keyword token.
    AsKeyword,
    /// The `call` keyword token.
    CallKeyword,
    /// The `command` keyword token.
    CommandKeyword,
    /// The `else` keyword token.
    ElseKeyword,
    /// The `false` keyword token.
    FalseKeyword,
    /// The `if` keyword token.
    IfKeyword,
    /// The `in` keyword token.
    InKeyword,
    /// The `import` keyword token.
    ImportKeyword,
    /// The `input` keyword token.
    InputKeyword,
    /// The `meta` keyword token.
    MetaKeyword,
    /// The `None` keyword.
    NoneKeyword,
    /// The `null` keyword token.
    NullKeyword,
    /// The `object` keyword token.
    ObjectKeyword,
    /// The `output` keyword token.
    OutputKeyword,
    /// The `parameter_meta` keyword token.
    ParameterMetaKeyword,
    /// The `runtime` keyword token.
    RuntimeKeyword,
    /// The `scatter` keyword token.
    ScatterKeyword,
    /// The `struct` keyword token.
    StructKeyword,
    /// The `task` keyword token.
    TaskKeyword,
    /// The `then` keyword token.
    ThenKeyword,
    /// The `true` keyword token.
    TrueKeyword,
    /// The `version` keyword token.
    VersionKeyword,
    /// The `workflow` keyword token.
    WorkflowKeyword,
    /// The 1.2 `Directory` type keyword token.
    DirectoryTypeKeyword,
    /// The 1.2 `hints` keyword token.
    HintsKeyword,
    /// The 1.2 `requirements` keyword token.
    RequirementsKeyword,
    /// The `{` symbol token.
    OpenBrace,
    /// The `}` symbol token.
    CloseBrace,
    /// The `[` symbol token.
    OpenBracket,
    /// The `]` symbol token.
    CloseBracket,
    /// The `=` symbol token.
    Assignment,
    /// The `:` symbol token.
    Colon,
    /// The `,` symbol token.
    Comma,
    /// The `(` symbol token.
    OpenParen,
    /// The `)` symbol token.
    CloseParen,
    /// The `?` symbol token.
    QuestionMark,
    /// The `!` symbol token.
    Exclamation,
    /// The `+` symbol token.
    Plus,
    /// The `-` symbol token.
    Minus,
    /// The `||` symbol token.
    LogicalOr,
    /// The `&&` symbol token.
    LogicalAnd,
    /// The `*` symbol token.
    Asterisk,
    /// The `**` symbol token.
    Exponentiation,
    /// The `/` symbol token.
    Slash,
    /// The `%` symbol token.
    Percent,
    /// The `==` symbol token.
    Equal,
    /// The `!=` symbol token.
    NotEqual,
    /// The `<=` symbol token.
    LessEqual,
    /// The `>=` symbol token.
    GreaterEqual,
    /// The `<` symbol token.
    Less,
    /// The `>` symbol token.
    Greater,
    /// The `.` symbol token.
    Dot,
    /// A literal text part of a string.
    LiteralStringText,
    /// A literal text part of a command.
    LiteralCommandText,
    /// A placeholder open token.
    PlaceholderOpen,

    /// Abandoned nodes are nodes that encountered errors.
    ///
    /// Children of abandoned nodes are re-parented to the parent of
    /// the abandoned node.
    ///
    /// As this is an internal implementation of error recovery,
    /// hide this variant from the documentation.
    #[doc(hidden)]
    Abandoned,
    /// Represents the WDL document root node.
    RootNode,
    /// Represents a version statement node.
    VersionStatementNode,
    /// Represents an import statement node.
    ImportStatementNode,
    /// Represents an import alias node.
    ImportAliasNode,
    /// Represents a struct definition node.
    StructDefinitionNode,
    /// Represents a task definition node.
    TaskDefinitionNode,
    /// Represents a workflow definition node.
    WorkflowDefinitionNode,
    /// Represents an unbound declaration node.
    UnboundDeclNode,
    /// Represents a bound declaration node.
    BoundDeclNode,
    /// Represents an input section node.
    InputSectionNode,
    /// Represents an output section node.
    OutputSectionNode,
    /// Represents a command section node.
    CommandSectionNode,
    /// Represents a requirements section node.
    RequirementsSectionNode,
    /// Represents a requirements item node.
    RequirementsItemNode,
    /// Represents a hints section node in a task.
    TaskHintsSectionNode,
    /// Represents a hints section node in a workflow.
    WorkflowHintsSectionNode,
    /// Represents a hints item node in a task.
    TaskHintsItemNode,
    /// Represents a hints item node in a workflow.
    WorkflowHintsItemNode,
    /// Represents a literal object in a workflow hints item value.
    WorkflowHintsObjectNode,
    /// Represents an item in a workflow hints object.
    WorkflowHintsObjectItemNode,
    /// Represents a literal array in a workflow hints item value.
    WorkflowHintsArrayNode,
    /// Represents a runtime section node.
    RuntimeSectionNode,
    /// Represents a runtime item node.
    RuntimeItemNode,
    /// Represents a primitive type node.
    PrimitiveTypeNode,
    /// Represents a map type node.
    MapTypeNode,
    /// Represents an array type node.
    ArrayTypeNode,
    /// Represents a pair type node.
    PairTypeNode,
    /// Represents an object type node.
    ObjectTypeNode,
    /// Represents a type reference node.
    TypeRefNode,
    /// Represents a metadata section node.
    MetadataSectionNode,
    /// Represents a parameter metadata section node.
    ParameterMetadataSectionNode,
    /// Represents a metadata object item node.
    MetadataObjectItemNode,
    /// Represents a metadata object node.
    MetadataObjectNode,
    /// Represents a metadata array node.
    MetadataArrayNode,
    /// Represents a literal integer node.
    LiteralIntegerNode,
    /// Represents a literal float node.
    LiteralFloatNode,
    /// Represents a literal boolean node.
    LiteralBooleanNode,
    /// Represents a literal `None` node.
    LiteralNoneNode,
    /// Represents a literal null node.
    LiteralNullNode,
    /// Represents a literal string node.
    LiteralStringNode,
    /// Represents a literal pair node.
    LiteralPairNode,
    /// Represents a literal array node.
    LiteralArrayNode,
    /// Represents a literal map node.
    LiteralMapNode,
    /// Represents a literal map item node.
    LiteralMapItemNode,
    /// Represents a literal object node.
    LiteralObjectNode,
    /// Represents a literal object item node.
    LiteralObjectItemNode,
    /// Represents a literal struct node.
    LiteralStructNode,
    /// Represents a literal struct item node.
    LiteralStructItemNode,
    /// Represents a literal hints node.
    LiteralHintsNode,
    /// Represents a literal hints item node.
    LiteralHintsItemNode,
    /// Represents a literal input node.
    LiteralInputNode,
    /// Represents a literal input item node.
    LiteralInputItemNode,
    /// Represents a literal output node.
    LiteralOutputNode,
    /// Represents a literal output item node.
    LiteralOutputItemNode,
    /// Represents a parenthesized expression node.
    ParenthesizedExprNode,
    /// Represents a name reference node.
    NameRefNode,
    /// Represents an `if` expression node.
    IfExprNode,
    /// Represents a logical not expression node.
    LogicalNotExprNode,
    /// Represents a negation expression node.
    NegationExprNode,
    /// Represents a logical `OR` expression node.
    LogicalOrExprNode,
    /// Represents a logical `AND` expression node.
    LogicalAndExprNode,
    /// Represents an equality expression node.
    EqualityExprNode,
    /// Represents an inequality expression node.
    InequalityExprNode,
    /// Represents a "less than" expression node.
    LessExprNode,
    /// Represents a "less than or equal to" expression node.
    LessEqualExprNode,
    /// Represents a "greater than" expression node.
    GreaterExprNode,
    /// Represents a "greater than or equal to" expression node.
    GreaterEqualExprNode,
    /// Represents an addition expression node.
    AdditionExprNode,
    /// Represents a subtraction expression node.
    SubtractionExprNode,
    /// Represents a multiplication expression node.
    MultiplicationExprNode,
    /// Represents a division expression node.
    DivisionExprNode,
    /// Represents a modulo expression node.
    ModuloExprNode,
    /// Represents a exponentiation expr node.
    ExponentiationExprNode,
    /// Represents a call expression node.'
    CallExprNode,
    /// Represents an index expression node.
    IndexExprNode,
    /// Represents an an access expression node.
    AccessExprNode,
    /// Represents a placeholder node in a string literal.
    PlaceholderNode,
    /// Placeholder `sep` option node.
    PlaceholderSepOptionNode,
    /// Placeholder `default` option node.
    PlaceholderDefaultOptionNode,
    /// Placeholder `true`/`false` option node.
    PlaceholderTrueFalseOptionNode,
    /// Represents a conditional statement node.
    ConditionalStatementNode,
    /// Represents a scatter statement node.
    ScatterStatementNode,
    /// Represents a call statement node.
    CallStatementNode,
    /// Represents a call target node in a call statement.
    CallTargetNode,
    /// Represents a call alias node in a call statement.
    CallAliasNode,
    /// Represents an `after` clause node in a call statement.
    CallAfterNode,
    /// Represents a call input item node.
    CallInputItemNode,

    // WARNING: this must always be the last variant.
    /// The exclusive maximum syntax kind value.
    MAX,
}

impl SyntaxKind {
    /// Returns whether the token is a symbolic [`SyntaxKind`].
    ///
    /// Generally speaking, symbolic [`SyntaxKind`]s have special meanings
    /// during parsing—they are not real elements of the grammar but rather an
    /// implementation detail.
    pub fn is_symbolic(&self) -> bool {
        matches!(
            self,
            SyntaxKind::Abandoned | SyntaxKind::Unknown | SyntaxKind::Unparsed | SyntaxKind::MAX
        )
    }
}

/// Every [`SyntaxKind`] variant.
pub static ALL_SYNTAX_KIND: &[SyntaxKind] = SyntaxKind::VARIANTS;

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        rowan::SyntaxKind(kind as u16)
    }
}

impl SyntaxKind {
    /// Describes the syntax kind.
    pub fn describe(&self) -> &'static str {
        match self {
            SyntaxKind::Unknown => unreachable!(),
            SyntaxKind::Unparsed => unreachable!(),
            SyntaxKind::Whitespace => "whitespace",
            SyntaxKind::Comment => "comment",
            SyntaxKind::Version => "version",
            SyntaxKind::Float => "float",
            SyntaxKind::Integer => "integer",
            SyntaxKind::Ident => "identifier",
            SyntaxKind::SingleQuote => "single quote",
            SyntaxKind::DoubleQuote => "double quote",
            SyntaxKind::OpenHeredoc => "open heredoc",
            SyntaxKind::CloseHeredoc => "close heredoc",
            SyntaxKind::ArrayTypeKeyword => "`Array` type keyword",
            SyntaxKind::BooleanTypeKeyword => "`Boolean` type keyword",
            SyntaxKind::FileTypeKeyword => "`File` type keyword",
            SyntaxKind::FloatTypeKeyword => "`Float` type keyword",
            SyntaxKind::IntTypeKeyword => "`Int` type keyword",
            SyntaxKind::MapTypeKeyword => "`Map` type keyword",
            SyntaxKind::ObjectTypeKeyword => "`Object` type keyword",
            SyntaxKind::PairTypeKeyword => "`Pair` type keyword",
            SyntaxKind::StringTypeKeyword => "`String` type keyword",
            SyntaxKind::AfterKeyword => "`after` keyword",
            SyntaxKind::AliasKeyword => "`alias` keyword",
            SyntaxKind::AsKeyword => "`as` keyword",
            SyntaxKind::CallKeyword => "`call` keyword",
            SyntaxKind::CommandKeyword => "`command` keyword",
            SyntaxKind::ElseKeyword => "`else` keyword",
            SyntaxKind::FalseKeyword => "`false` keyword",
            SyntaxKind::IfKeyword => "`if` keyword",
            SyntaxKind::InKeyword => "`in` keyword",
            SyntaxKind::ImportKeyword => "`import` keyword",
            SyntaxKind::InputKeyword => "`input` keyword",
            SyntaxKind::MetaKeyword => "`meta` keyword",
            SyntaxKind::NoneKeyword => "`None` keyword",
            SyntaxKind::NullKeyword => "`null` keyword",
            SyntaxKind::ObjectKeyword => "`object` keyword",
            SyntaxKind::OutputKeyword => "`output` keyword",
            SyntaxKind::ParameterMetaKeyword => "`parameter_meta` keyword",
            SyntaxKind::RuntimeKeyword => "`runtime` keyword",
            SyntaxKind::ScatterKeyword => "`scatter` keyword",
            SyntaxKind::StructKeyword => "`struct` keyword",
            SyntaxKind::TaskKeyword => "`task` keyword",
            SyntaxKind::ThenKeyword => "`then` keyword",
            SyntaxKind::TrueKeyword => "`true` keyword",
            SyntaxKind::VersionKeyword => "`version` keyword",
            SyntaxKind::WorkflowKeyword => "`workflow` keyword",
            SyntaxKind::DirectoryTypeKeyword => "`Directory` type keyword",
            SyntaxKind::HintsKeyword => "`hints` keyword",
            SyntaxKind::RequirementsKeyword => "`requirements` keyword",
            SyntaxKind::OpenBrace => "`{` symbol",
            SyntaxKind::CloseBrace => "`}` symbol",
            SyntaxKind::OpenBracket => "`[` symbol",
            SyntaxKind::CloseBracket => "`]` symbol",
            SyntaxKind::Assignment => "`=` symbol",
            SyntaxKind::Colon => "`:` symbol",
            SyntaxKind::Comma => "`,` symbol",
            SyntaxKind::OpenParen => "`(` symbol",
            SyntaxKind::CloseParen => "`)` symbol",
            SyntaxKind::QuestionMark => "`?` symbol",
            SyntaxKind::Exclamation => "`!` symbol",
            SyntaxKind::Plus => "`+` symbol",
            SyntaxKind::Minus => "`-` symbol",
            SyntaxKind::LogicalOr => "`||` symbol",
            SyntaxKind::LogicalAnd => "`&&` symbol",
            SyntaxKind::Asterisk => "`*` symbol",
            SyntaxKind::Exponentiation => "`**` symbol",
            SyntaxKind::Slash => "`/` symbol",
            SyntaxKind::Percent => "`%` symbol",
            SyntaxKind::Equal => "`==` symbol",
            SyntaxKind::NotEqual => "`!=` symbol",
            SyntaxKind::LessEqual => "`<=` symbol",
            SyntaxKind::GreaterEqual => "`>=` symbol",
            SyntaxKind::Less => "`<` symbol",
            SyntaxKind::Greater => "`>` symbol",
            SyntaxKind::Dot => "`.` symbol",
            SyntaxKind::LiteralStringText => "literal string text",
            SyntaxKind::LiteralCommandText => "literal command text",
            SyntaxKind::PlaceholderOpen => "placeholder open",
            SyntaxKind::Abandoned => unreachable!(),
            SyntaxKind::RootNode => "root node",
            SyntaxKind::VersionStatementNode => "version statement",
            SyntaxKind::ImportStatementNode => "import statement",
            SyntaxKind::ImportAliasNode => "import alias",
            SyntaxKind::StructDefinitionNode => "struct definition",
            SyntaxKind::TaskDefinitionNode => "task definition",
            SyntaxKind::WorkflowDefinitionNode => "workflow definition",
            SyntaxKind::UnboundDeclNode => "declaration without assignment",
            SyntaxKind::BoundDeclNode => "declaration with assignment",
            SyntaxKind::InputSectionNode => "input section",
            SyntaxKind::OutputSectionNode => "output section",
            SyntaxKind::CommandSectionNode => "command section",
            SyntaxKind::RequirementsSectionNode => "requirements section",
            SyntaxKind::RequirementsItemNode => "requirements item",
            SyntaxKind::TaskHintsSectionNode | SyntaxKind::WorkflowHintsSectionNode => {
                "hints section"
            }
            SyntaxKind::TaskHintsItemNode | SyntaxKind::WorkflowHintsItemNode => "hints item",
            SyntaxKind::WorkflowHintsObjectNode => "literal object",
            SyntaxKind::WorkflowHintsObjectItemNode => "literal object item",
            SyntaxKind::WorkflowHintsArrayNode => "literal array",
            SyntaxKind::RuntimeSectionNode => "runtime section",
            SyntaxKind::RuntimeItemNode => "runtime item",
            SyntaxKind::PrimitiveTypeNode => "primitive type",
            SyntaxKind::MapTypeNode => "map type",
            SyntaxKind::ArrayTypeNode => "array type",
            SyntaxKind::PairTypeNode => "pair type",
            SyntaxKind::ObjectTypeNode => "object type",
            SyntaxKind::TypeRefNode => "type reference",
            SyntaxKind::MetadataSectionNode => "metadata section",
            SyntaxKind::ParameterMetadataSectionNode => "parameter metadata section",
            SyntaxKind::MetadataObjectItemNode => "metadata object item",
            SyntaxKind::MetadataObjectNode => "metadata object",
            SyntaxKind::MetadataArrayNode => "metadata array",
            SyntaxKind::LiteralIntegerNode => "literal integer",
            SyntaxKind::LiteralFloatNode => "literal float",
            SyntaxKind::LiteralBooleanNode => "literal boolean",
            SyntaxKind::LiteralNoneNode => "literal `None`",
            SyntaxKind::LiteralNullNode => "literal null",
            SyntaxKind::LiteralStringNode => "literal string",
            SyntaxKind::LiteralPairNode => "literal pair",
            SyntaxKind::LiteralArrayNode => "literal array",
            SyntaxKind::LiteralMapNode => "literal map",
            SyntaxKind::LiteralMapItemNode => "literal map item",
            SyntaxKind::LiteralObjectNode => "literal object",
            SyntaxKind::LiteralObjectItemNode => "literal object item",
            SyntaxKind::LiteralStructNode => "literal struct",
            SyntaxKind::LiteralStructItemNode => "literal struct item",
            SyntaxKind::LiteralHintsNode => "literal hints",
            SyntaxKind::LiteralHintsItemNode => "literal hints item",
            SyntaxKind::LiteralInputNode => "literal input",
            SyntaxKind::LiteralInputItemNode => "literal input item",
            SyntaxKind::LiteralOutputNode => "literal output",
            SyntaxKind::LiteralOutputItemNode => "literal output item",
            SyntaxKind::ParenthesizedExprNode => "parenthesized expression",
            SyntaxKind::NameRefNode => "name reference",
            SyntaxKind::IfExprNode => "`if` expression",
            SyntaxKind::LogicalNotExprNode => "logical not expression",
            SyntaxKind::NegationExprNode => "negation expression",
            SyntaxKind::LogicalOrExprNode => "logical OR expression",
            SyntaxKind::LogicalAndExprNode => "logical AND expression",
            SyntaxKind::EqualityExprNode => "equality expression",
            SyntaxKind::InequalityExprNode => "inequality expression",
            SyntaxKind::LessExprNode => "less than expression",
            SyntaxKind::LessEqualExprNode => "less than or equal to expression",
            SyntaxKind::GreaterExprNode => "greater than expression",
            SyntaxKind::GreaterEqualExprNode => "greater than or equal to expression",
            SyntaxKind::AdditionExprNode => "addition expression",
            SyntaxKind::SubtractionExprNode => "subtraction expression",
            SyntaxKind::MultiplicationExprNode => "multiplication expression",
            SyntaxKind::DivisionExprNode => "division expression",
            SyntaxKind::ModuloExprNode => "modulo expression",
            SyntaxKind::ExponentiationExprNode => "exponentiation expression",
            SyntaxKind::CallExprNode => "call expression",
            SyntaxKind::IndexExprNode => "index expression",
            SyntaxKind::AccessExprNode => "access expression",
            SyntaxKind::PlaceholderNode => "placeholder",
            SyntaxKind::PlaceholderSepOptionNode => "placeholder `sep` option",
            SyntaxKind::PlaceholderDefaultOptionNode => "placeholder `default` option",
            SyntaxKind::PlaceholderTrueFalseOptionNode => "placeholder `true`/`false` option",
            SyntaxKind::ConditionalStatementNode => "conditional statement",
            SyntaxKind::ScatterStatementNode => "scatter statement",
            SyntaxKind::CallStatementNode => "call statement",
            SyntaxKind::CallTargetNode => "call target",
            SyntaxKind::CallAliasNode => "call alias",
            SyntaxKind::CallAfterNode => "call `after` clause",
            SyntaxKind::CallInputItemNode => "call input item",
            SyntaxKind::MAX => unreachable!(),
        }
    }

    /// Returns whether the [`SyntaxKind`] is trivia.
    pub fn is_trivia(&self) -> bool {
        matches!(self, SyntaxKind::Whitespace | SyntaxKind::Comment)
    }
}

/// Represents the Workflow Definition Language (WDL).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorkflowDescriptionLanguage;

impl rowan::Language for WorkflowDescriptionLanguage {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        assert!(raw.0 <= SyntaxKind::MAX as u16);
        unsafe { std::mem::transmute::<u16, SyntaxKind>(raw.0) }
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

/// Represents a node in the concrete syntax tree.
pub type SyntaxNode = rowan::SyntaxNode<WorkflowDescriptionLanguage>;
/// Represents a token in the concrete syntax tree.
pub type SyntaxToken = rowan::SyntaxToken<WorkflowDescriptionLanguage>;
/// Represents an element (node or token) in the concrete syntax tree.
pub type SyntaxElement = rowan::SyntaxElement<WorkflowDescriptionLanguage>;
/// Represents node children in the concrete syntax tree.
pub type SyntaxNodeChildren = rowan::SyntaxNodeChildren<WorkflowDescriptionLanguage>;

/// Represents an untyped concrete syntax tree.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SyntaxTree(SyntaxNode);

impl SyntaxTree {
    /// Parses WDL source to produce a syntax tree.
    ///
    /// A syntax tree is always returned, even for invalid WDL documents.
    ///
    /// Additionally, the list of diagnostics encountered during the parse is
    /// returned; if the list is empty, the tree is syntactically correct.
    ///
    /// However, additional validation is required to ensure the source is
    /// a valid WDL document.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use wdl_grammar::SyntaxTree;
    /// let (tree, diagnostics) = SyntaxTree::parse("version 1.1");
    /// assert!(diagnostics.is_empty());
    /// println!("{tree:#?}");
    /// ```
    pub fn parse(source: &str) -> (Self, Vec<Diagnostic>) {
        let parser = Parser::new(Lexer::new(source));
        let (events, mut diagnostics) = grammar::document(source, parser);
        diagnostics.sort();

        Self::build(source, events, diagnostics)
    }

    /// Builds the concrete syntax tree from a list of parser events.
    fn build(
        source: &str,
        mut events: Vec<Event>,
        diagnostics: Vec<Diagnostic>,
    ) -> (Self, Vec<Diagnostic>) {
        let mut builder = GreenNodeBuilder::default();
        let mut ancestors = Vec::new();

        for i in 0..events.len() {
            match std::mem::replace(&mut events[i], Event::abandoned()) {
                Event::NodeStarted {
                    kind,
                    forward_parent,
                } => {
                    // Walk the forward parent chain, if there is one, and push
                    // each forward parent to the ancestors list
                    ancestors.push(kind);
                    let mut idx = i;
                    let mut fp: Option<usize> = forward_parent;
                    while let Some(distance) = fp {
                        idx += distance;
                        fp = match std::mem::replace(&mut events[idx], Event::abandoned()) {
                            Event::NodeStarted {
                                kind,
                                forward_parent,
                            } => {
                                ancestors.push(kind);
                                forward_parent
                            }
                            _ => unreachable!(),
                        };
                    }

                    // As the current node was pushed first and then its ancestors, walk
                    // the list in reverse to start the "oldest" ancestor first
                    for kind in ancestors.drain(..).rev() {
                        if kind != SyntaxKind::Abandoned {
                            builder.start_node(kind.into());
                        }
                    }
                }
                Event::NodeFinished => builder.finish_node(),
                Event::Token { kind, span } => {
                    builder.token(kind.into(), &source[span.start()..span.end()])
                }
            }
        }

        (Self(SyntaxNode::new_root(builder.finish())), diagnostics)
    }

    /// Gets the root syntax node of the tree.
    pub fn root(&self) -> &SyntaxNode {
        &self.0
    }

    /// Gets a copy of the underlying root green node for the tree.
    pub fn green(&self) -> Cow<'_, GreenNodeData> {
        self.0.green()
    }

    /// Converts the tree into a syntax node.
    pub fn into_syntax(self) -> SyntaxNode {
        self.0
    }
}

impl fmt::Display for SyntaxTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Debug for SyntaxTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Gathers substantial trivia (comments and blank lines) from a
/// [`SyntaxToken`].
///
/// Whitespace is only considered substantial if it contains more than one
/// newline and is between comments. Comments are always considered substantial.
fn gather_substantial_trivia(
    source: &SyntaxToken,
    direction: Direction,
    break_on_newline: bool,
) -> Box<[SyntaxToken]> {
    /// Adds the token to the currently collecting buffer in the right place
    /// depending in the direction we are traversing.
    fn push_results(
        token: SyntaxToken,
        results: &mut VecDeque<SyntaxToken>,
        direction: &Direction,
    ) {
        match direction {
            Direction::Next => results.push_back(token),
            Direction::Prev => results.push_front(token),
        }
    }

    let mut results = VecDeque::new();
    let mut cur = match direction {
        Direction::Next => source.next_token(),
        Direction::Prev => source.prev_token(),
    };
    while let Some(t) = cur {
        if !t.kind().is_trivia() {
            break;
        }

        match t.kind() {
            SyntaxKind::Comment => {
                // Check if t is a comment on its own line.
                // If direction is 'Next' then we already know that the
                // comment is on its own line.
                if direction == Direction::Prev {
                    if let Some(prev) = t.prev_token() {
                        if prev.kind() == SyntaxKind::Whitespace {
                            let newlines = prev.text().chars().filter(|c| *c == '\n').count();

                            // If there are newlines in 'prev' then we know
                            // that the comment is on its own line.
                            // The comment may still be on its own line if
                            // 'prev' does not have newlines and nothing comes
                            // before 'prev'.
                            if newlines == 0 && prev.prev_token().is_some() {
                                break;
                            }
                        } else {
                            // There is something else on this line before the comment.
                            break;
                        }
                    }
                }
                push_results(t.clone(), &mut results, &direction);
            }
            SyntaxKind::Whitespace => {
                let newlines = t.text().chars().filter(|c| *c == '\n').count();

                if break_on_newline && newlines > 0 {
                    break;
                }

                if newlines > 1 {
                    push_results(t.clone(), &mut results, &direction);
                }
            }
            // SAFETY: we just filtered out any non-comment and
            // non-whitespace nodes above, so this should never occur.
            _ => unreachable!(),
        }
        cur = match direction {
            Direction::Next => t.next_token(),
            Direction::Prev => t.prev_token(),
        };
    }

    // // Remove leading and trailing whitespace from results.
    // while let Some(t) = results.front() {
    //     if t.kind() == SyntaxKind::Whitespace {
    //         results.pop_front();
    //     } else {
    //         break;
    //     }
    // }
    // while let Some(t) = results.back() {
    //     if t.kind() == SyntaxKind::Whitespace {
    //         results.pop_back();
    //     } else {
    //         break;
    //     }
    // }

    // NOTE: most of the time, this conversion will be O(1). Occassionally
    // it will be O(n). No allocations will ever be done. Thus, the
    // ammortized cost of this is quite cheap.
    Vec::from(results).into_boxed_slice()
}

/// An extension trait for [`SyntaxNode`]s, [`SyntaxToken`]s, and
/// [`SyntaxElement`]s.
pub trait SyntaxExt {
    /// Returns whether `self` matches the provided element.
    fn matches(&self, other: &SyntaxElement) -> bool;

    /// Gets the siblings with tokens.
    ///
    /// **NOTE:** this needed because Rowan does not encapsulate this
    /// functionality in a trait. Once wrapped here, most of the functions
    /// provided by this extension trait can just be provided, which simplifies
    /// the code. Generally speaking, this should just defer to the underlying
    /// `siblings_with_tokens` method for each type.
    fn siblings_with_tokens(&self, direction: Direction)
    -> Box<dyn Iterator<Item = SyntaxElement>>;

    /// Returns all of the siblings _before_ the current element.
    ///
    /// The siblings are returned in the order they were parsed.
    fn preceding_siblings(&self) -> Box<[SyntaxElement]> {
        let mut results = VecDeque::new();

        self.siblings_with_tokens(Direction::Prev)
            // NOTE: this `skip_while` is necessary because
            // `siblings_with_tokens` returns the current node.
            .skip_while(|e| self.matches(e))
            .for_each(|e| results.push_front(e));

        // NOTE: most of the time, this conversion will be O(1). Occassionally
        // it will be O(n). No allocations will ever be done. Thus, the
        // ammortized cost of this is quite cheap.
        Vec::from(results).into_boxed_slice()
    }

    /// Returns all of the siblings _after_ the current element.
    ///
    /// The siblings are returned in the order they were parsed.
    fn succeeding_siblings(&self) -> Box<[SyntaxElement]> {
        let mut results = Vec::new();

        self.siblings_with_tokens(Direction::Next)
            // NOTE: this `skip_while` is necessary because
            // `siblings_with_tokens` returns the current node.
            .skip_while(|e| self.matches(e))
            .for_each(|e| results.push(e));

        // NOTE: this should always be O(1) and never require any additional
        // allocations.
        results.into_boxed_slice()
    }

    /// Gets all elements that are adjacent to a particular element (not
    /// including the element itself). This means in both the forward and
    /// reverse direction.
    ///
    /// The siblings are returned in the order they were parsed.
    fn adjacent(&self) -> Box<[SyntaxElement]> {
        let mut results = Vec::from(self.preceding_siblings());
        results.extend(self.succeeding_siblings().iter().cloned());

        // NOTE: this should always be O(1) and never require any additional
        // allocations.
        results.into_boxed_slice()
    }
}

impl SyntaxExt for SyntaxNode {
    fn matches(&self, other: &SyntaxElement) -> bool {
        other.as_node().map(|n| n == self).unwrap_or(false)
    }

    fn siblings_with_tokens(
        &self,
        direction: Direction,
    ) -> Box<dyn Iterator<Item = SyntaxElement>> {
        Box::new(self.siblings_with_tokens(direction))
    }
}

impl SyntaxExt for SyntaxToken {
    fn matches(&self, other: &SyntaxElement) -> bool {
        other.as_token().map(|n| n == self).unwrap_or(false)
    }

    fn siblings_with_tokens(
        &self,
        direction: Direction,
    ) -> Box<dyn Iterator<Item = SyntaxElement>> {
        Box::new(self.siblings_with_tokens(direction))
    }
}

impl SyntaxExt for SyntaxElement {
    fn matches(&self, other: &SyntaxElement) -> bool {
        self == other
    }

    fn siblings_with_tokens(
        &self,
        direction: Direction,
    ) -> Box<dyn Iterator<Item = SyntaxElement>> {
        match self {
            SyntaxElement::Node(node) => Box::new(node.siblings_with_tokens(direction)),
            SyntaxElement::Token(token) => Box::new(token.siblings_with_tokens(direction)),
        }
    }
}

/// An extension trait for [`SyntaxToken`]s.
pub trait SyntaxTokenExt {
    /// Gets all of the substantial preceding trivia for an element.
    fn preceding_trivia(&self) -> Box<[SyntaxToken]>
    where
        Self: Sized,
        Self: SyntaxExt;

    /// Gets all of the substantial succeeding trivia for an element.
    fn succeeding_trivia(&self) -> Box<[SyntaxToken]>
    where
        Self: Sized,
        Self: SyntaxExt;

    /// Get any inline comment directly following an element on the
    /// same line.
    fn inline_comment(&self) -> Option<SyntaxToken>
    where
        Self: Sized,
        Self: SyntaxExt;
}

impl SyntaxTokenExt for SyntaxToken {
    fn preceding_trivia(&self) -> Box<[SyntaxToken]>
    where
        Self: Sized,
        Self: SyntaxExt,
    {
        gather_substantial_trivia(self, Direction::Prev, false)
    }

    fn succeeding_trivia(&self) -> Box<[SyntaxToken]>
    where
        Self: Sized,
        Self: SyntaxExt,
    {
        gather_substantial_trivia(self, Direction::Next, false)
    }

    fn inline_comment(&self) -> Option<SyntaxToken>
    where
        Self: Sized,
        Self: SyntaxExt,
    {
        gather_substantial_trivia(self, Direction::Next, true)
            // NOTE: at most, there can be one contiguous comment on a line.
            .first()
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SyntaxTree;

    #[test]
    fn preceding_comments() {
        let (tree, diagnostics) = SyntaxTree::parse(
            "version 1.2

# This comment should not be included
task foo {} # This comment should not be included

# Some
# comments
# are
# long
    
# Others are short

#     and, yet    another
workflow foo {} # This should not be collected.

# This comment should not be included either.",
        );

        assert!(diagnostics.is_empty());

        let workflow = tree.root().last_child().unwrap();
        assert_eq!(workflow.kind(), SyntaxKind::WorkflowDefinitionNode);
        let trivia = workflow.first_token().unwrap().preceding_trivia();
        let mut trivia_iter = trivia.iter();
        assert_eq!(trivia_iter.next().unwrap().text(), "\n\n");
        assert_eq!(trivia_iter.next().unwrap().text(), "# Some");
        assert_eq!(trivia_iter.next().unwrap().text(), "# comments");
        assert_eq!(trivia_iter.next().unwrap().text(), "# are");
        assert_eq!(trivia_iter.next().unwrap().text(), "# long");
        assert_eq!(trivia_iter.next().unwrap().text(), "\n    \n");
        assert_eq!(trivia_iter.next().unwrap().text(), "# Others are short");
        assert_eq!(trivia_iter.next().unwrap().text(), "\n\n");
        assert_eq!(
            trivia_iter.next().unwrap().text(),
            "#     and, yet    another"
        );
        assert!(trivia_iter.next().is_none());
    }

    #[test]
    fn succeeding_comments() {
        let (tree, diagnostics) = SyntaxTree::parse(
            "version 1.2

# This comment should not be included
task foo {}

# This should not be collected.
workflow foo {} # Here is a comment that should be collected.

# This comment should be included too.",
        );

        assert!(diagnostics.is_empty());

        let workflow = tree.root().last_child().unwrap();
        assert_eq!(workflow.kind(), SyntaxKind::WorkflowDefinitionNode);
        let trivia = workflow.last_token().unwrap().succeeding_trivia();
        let mut trivia_iter = trivia.iter();
        assert_eq!(
            trivia_iter.next().unwrap().text(),
            "# Here is a comment that should be collected."
        );
        assert_eq!(trivia_iter.next().unwrap().text(), "\n\n");
        assert_eq!(
            trivia_iter.next().unwrap().text(),
            "# This comment should be included too."
        );
        assert!(trivia_iter.next().is_none());
    }

    #[test]
    fn inline_comment() {
        let (tree, diagnostics) = SyntaxTree::parse(
            "version 1.2

# This comment should not be included
task foo {}

# This should not be collected.
workflow foo {} # Here is a comment that should be collected.

# This comment should not be included either.",
        );

        assert!(diagnostics.is_empty());

        let workflow = tree.root().last_child().unwrap();
        assert_eq!(workflow.kind(), SyntaxKind::WorkflowDefinitionNode);
        let comment = workflow.last_token().unwrap().inline_comment().unwrap();
        assert_eq!(
            comment.text(),
            "# Here is a comment that should be collected."
        );
    }
}
