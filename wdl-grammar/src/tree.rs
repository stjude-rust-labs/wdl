//! Module for the concrete syntax tree (CST) representation.

use std::fmt;

use rowan::GreenNodeBuilder;

use super::grammar;
use super::lexer::Lexer;
use super::parser::Event;
use super::Diagnostic;
use crate::parser::Parser;

/// Represents the kind of syntax element (node or token) in a WDL concrete
/// syntax tree (CST).
///
/// Nodes have at least one token child and represent a syntactic construct.
///
/// Tokens are terminal and represent any span of the source.
///
/// This enumeration is a union of all supported WDL tokens and nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    /// The reserved `Directory` type keyword token.
    DirectoryTypeKeyword,
    /// The reserved `hints` keyword token.
    HintsKeyword,
    /// The reserved `requirements` keyword token.
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

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        rowan::SyntaxKind(kind as u16)
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
        let (events, errors) = grammar::document(source, parser);
        Self::build(source, events, errors)
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
