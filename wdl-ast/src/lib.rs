//! An abstract syntax tree for Workflow Description Language (WDL) documents.
//!
//! The AST implementation is effectively a facade over the concrete syntax tree
//! (CST) implemented by [SyntaxTree] from `wdl-grammar`.
//!
//! An AST is cheap to construct and may be cheaply cloned at any level.
//!
//! However, an AST (and the underlying CST) are immutable; updating the tree
//! requires replacing a node in the tree to produce a new tree. The unaffected
//! nodes of the replacement are reused from the old tree to the new tree.
//!
//! # Examples
//!
//! An example of parsing a WDL document into an AST and validating it:
//!
//! ```rust
//! # let source = "version 1.1\nworkflow test {}";
//! use wdl_ast::Document;
//! use wdl_ast::Validator;
//!
//! let (document, diagnostics) = Document::parse(source);
//! if !diagnostics.is_empty() {
//!     // Handle the failure to parse
//! }
//!
//! let mut validator = Validator::default();
//! if let Err(diagnostics) = validator.validate(&document) {
//!     // Handle the failure to validate
//! }
//! ```

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(missing_debug_implementations)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(rustdoc::broken_intra_doc_links)]

use std::collections::HashSet;
use std::fmt;

pub use rowan::Direction;
use rowan::NodeOrToken;
use v1::CloseBrace;
use v1::CloseHeredoc;
use v1::OpenBrace;
use v1::OpenHeredoc;
pub use wdl_grammar::Diagnostic;
pub use wdl_grammar::Label;
pub use wdl_grammar::Severity;
pub use wdl_grammar::Span;
pub use wdl_grammar::SupportedVersion;
pub use wdl_grammar::SyntaxElement;
pub use wdl_grammar::SyntaxExt;
pub use wdl_grammar::SyntaxKind;
pub use wdl_grammar::SyntaxNode;
pub use wdl_grammar::SyntaxToken;
pub use wdl_grammar::SyntaxTokenExt;
pub use wdl_grammar::SyntaxTree;
pub use wdl_grammar::WorkflowDescriptionLanguage;
pub use wdl_grammar::version;

pub mod v1;

mod element;
mod validation;
mod visitor;

pub use element::*;
pub use validation::*;
pub use visitor::*;

/// A trait that abstracts the underlying representation of a syntax tree node.
///
/// The default node type is `SyntaxNode` for all AST nodes.
pub trait TreeNode: Clone + fmt::Debug + PartialEq + Eq + std::hash::Hash {
    /// The associated token type for the tree node.
    type Token: TreeToken;

    /// Gets the parent node of the node.
    ///
    /// Returns `None` if the node is a root.
    fn parent(&self) -> Option<Self>;

    /// Gets the syntax kind of the node.
    fn kind(&self) -> SyntaxKind;

    /// Gets the text of the node.
    ///
    /// Node text is not contiguous, so the returned value implements `Display`.
    fn text(&self) -> impl fmt::Display;

    /// Gets the span of the node.
    fn span(&self) -> Span;

    /// Gets the children nodes of the node.
    fn children(&self) -> impl Iterator<Item = Self>;

    /// Gets all the children of the node, including tokens.
    fn children_with_tokens(&self) -> impl Iterator<Item = NodeOrToken<Self, Self::Token>>;

    /// Gets the last token of the node.
    fn last_token(&self) -> Option<Self::Token>;

    /// Gets the node descendants of the node.
    fn descendants(&self) -> impl Iterator<Item = Self>;

    /// Gets the ancestors of the node.
    fn ancestors(&self) -> impl Iterator<Item = Self>;

    /// Determines if a given rule id is excepted for the node.
    fn is_rule_excepted(&self, id: &str) -> bool;
}

/// A trait that abstracts the underlying representation of a syntax token.
pub trait TreeToken: Clone + fmt::Debug + PartialEq + Eq + std::hash::Hash {
    /// The associated node type for the token.
    type Node: TreeNode;

    /// Gets the parent node of the token.
    fn parent(&self) -> Self::Node;

    /// Gets the syntax kind for the token.
    fn kind(&self) -> SyntaxKind;

    /// Gets the text of the token.
    fn text(&self) -> &str;

    /// Gets the span of the token.
    fn span(&self) -> Span;
}

/// A trait implemented by AST nodes.
pub trait AstNode<N: TreeNode>: Sized {
    /// Determines if the kind can be cast to this representation.
    fn can_cast(kind: SyntaxKind) -> bool;

    /// Casts the given inner type to the this representation.
    fn cast(inner: N) -> Option<Self>;

    /// Gets the inner type from this representation.
    fn inner(&self) -> &N;

    /// Gets the syntax kind of the node.
    fn kind(&self) -> SyntaxKind {
        self.inner().kind()
    }

    /// Gets the text of the node.
    ///
    /// As node text is not contiguous, this returns a type that implements
    /// `Display`.
    fn text<'a>(&'a self) -> impl fmt::Display
    where
        N: 'a,
    {
        self.inner().text()
    }

    /// Gets the span of the node.
    fn span(&self) -> Span {
        self.inner().span()
    }

    /// Gets the first token child that can cast to an expected type.
    fn token<C>(&self) -> Option<C>
    where
        C: AstToken<N::Token>,
    {
        self.inner()
            .children_with_tokens()
            .filter_map(|e| e.into_token())
            .find_map(|t| C::cast(t))
    }

    /// Gets all the token children that can cast to an expected type.
    fn tokens<'a, C>(&'a self) -> impl Iterator<Item = C>
    where
        C: AstToken<N::Token>,
        N: 'a,
    {
        self.inner()
            .children_with_tokens()
            .filter_map(|e| e.into_token().and_then(C::cast))
    }

    /// Gets the last token of the node and attempts to cast it to an expected
    /// type.
    ///
    /// Returns `None` if there is no last token or if it cannot be casted to
    /// the expected type.
    fn last_token<C>(&self) -> Option<C>
    where
        C: AstToken<N::Token>,
    {
        self.inner().last_token().and_then(C::cast)
    }

    /// Gets the first node child that can cast to an expected type.
    fn child<C>(&self) -> Option<C>
    where
        C: AstNode<N>,
    {
        self.inner().children().find_map(C::cast)
    }

    /// Gets all node children that can cast to an expected type.
    fn children<'a, C>(&'a self) -> impl Iterator<Item = C>
    where
        C: AstNode<N>,
        N: 'a,
    {
        self.inner().children().filter_map(C::cast)
    }

    /// Gets the parent of the node if the underlying tree node has a parent.
    ///
    /// Returns `None` if the node has no parent or if the parent node is not of
    /// the expected type.
    fn parent<'a, P>(&self) -> Option<P>
    where
        P: AstNode<N>,
        N: 'a,
    {
        P::cast(self.inner().parent()?)
    }

    /// Calculates the span of a scope given the node where the scope is
    /// visible.
    ///
    /// Returns `None` if the node does not contain the open and close tokens as
    /// children.
    fn scope_span<O, C>(&self) -> Option<Span>
    where
        O: AstToken<N::Token>,
        C: AstToken<N::Token>,
    {
        let open = self.token::<O>()?.span();
        let close = self.last_token::<C>()?.span();

        // The span starts after the opening brace and before the closing brace
        Some(Span::new(open.end(), close.start() - open.end()))
    }

    /// Gets the interior span of child opening and closing brace tokens for the
    /// node.
    ///
    /// The span starts from immediately after the opening brace token and ends
    /// immediately before the closing brace token.
    ///
    /// Returns `None` if the node does not contain child brace tokens.
    fn braced_scope_span(&self) -> Option<Span> {
        self.scope_span::<OpenBrace<N::Token>, CloseBrace<N::Token>>()
    }

    /// Gets the interior span of child opening and closing heredoc tokens for
    /// the node.
    ///
    /// The span starts from immediately after the opening heredoc token and
    /// ends immediately before the closing heredoc token.
    ///
    /// Returns `None` if the node does not contain child heredoc tokens.
    fn heredoc_scope_span(&self) -> Option<Span> {
        self.scope_span::<OpenHeredoc<N::Token>, CloseHeredoc<N::Token>>()
    }

    /// Gets the node descendants (including self) from this node that can be
    /// cast to the expected type.
    fn descendants<'a, D>(&'a self) -> impl Iterator<Item = D>
    where
        D: AstNode<N>,
        N: 'a,
    {
        self.inner().descendants().filter_map(|d| D::cast(d))
    }
}

/// A trait implemented by AST tokens.
pub trait AstToken<T: TreeToken>: Sized {
    /// Determines if the kind can be cast to this representation.
    fn can_cast(kind: SyntaxKind) -> bool;

    /// Casts the given inner type to the this representation.
    fn cast(inner: T) -> Option<Self>;

    /// Gets the inner type from this representation.
    fn inner(&self) -> &T;

    /// Gets the syntax kind of the token.
    fn kind(&self) -> SyntaxKind {
        self.inner().kind()
    }

    /// Gets the text of the token.
    fn text<'a>(&'a self) -> &'a str
    where
        T: 'a,
    {
        self.inner().text()
    }

    /// Gets the span of the token.
    fn span(&self) -> Span {
        self.inner().span()
    }

    /// Gets the parent of the token.
    ///
    /// Returns `None` if the parent node cannot be cast to the expected type.
    fn parent<'a, P>(&self) -> Option<P>
    where
        P: AstNode<T::Node>,
        T: 'a,
    {
        P::cast(self.inner().parent())
    }
}

/// Implemented by nodes that can create a new root from a different tree node
/// type.
pub trait NewRoot<N: TreeNode>: Sized {
    /// Constructs a new root node from the give root node of a different tree
    /// node type.
    fn new_root(root: N) -> Self;
}

impl TreeNode for SyntaxNode {
    type Token = SyntaxToken;

    fn parent(&self) -> Option<SyntaxNode> {
        self.parent()
    }

    fn kind(&self) -> SyntaxKind {
        self.kind()
    }

    fn children(&self) -> impl Iterator<Item = Self> {
        self.children()
    }

    fn children_with_tokens(&self) -> impl Iterator<Item = NodeOrToken<Self, Self::Token>> {
        self.children_with_tokens()
    }

    fn text(&self) -> impl fmt::Display {
        self.text()
    }

    fn span(&self) -> Span {
        let range = self.text_range();
        let start = usize::from(range.start());
        Span::new(start, usize::from(range.end()) - start)
    }

    fn last_token(&self) -> Option<Self::Token> {
        self.last_token()
    }

    fn descendants(&self) -> impl Iterator<Item = Self> {
        self.descendants()
    }

    fn ancestors(&self) -> impl Iterator<Item = Self> {
        self.ancestors()
    }

    fn is_rule_excepted(&self, id: &str) -> bool {
        <Self as SyntaxNodeExt>::is_rule_excepted(self, id)
    }
}

impl TreeToken for SyntaxToken {
    type Node = SyntaxNode;

    fn parent(&self) -> SyntaxNode {
        self.parent().expect("token should have a parent")
    }

    fn kind(&self) -> SyntaxKind {
        self.kind()
    }

    fn text(&self) -> &str {
        self.text()
    }

    fn span(&self) -> Span {
        let range = self.text_range();
        let start = usize::from(range.start());
        Span::new(start, usize::from(range.end()) - start)
    }
}

/// Represents the reason an AST node has been visited.
///
/// Each node is visited exactly once, but the visitor will receive
/// a call for entering the node and a call for exiting the node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VisitReason {
    /// The visit has entered the node.
    Enter,
    /// The visit has exited the node.
    Exit,
}

/// An extension trait for syntax nodes.
pub trait SyntaxNodeExt {
    /// Gets an iterator over the `@except` comments for a syntax node.
    fn except_comments(&self) -> impl Iterator<Item = SyntaxToken> + '_;

    /// Gets the AST node's rule exceptions set.
    ///
    /// The set is the comma-delimited list of rule identifiers that follows a
    /// `#@ except:` comment.
    fn rule_exceptions(&self) -> HashSet<String>;

    /// Determines if a given rule id is excepted for the syntax node.
    fn is_rule_excepted(&self, id: &str) -> bool;
}

impl SyntaxNodeExt for SyntaxNode {
    fn except_comments(&self) -> impl Iterator<Item = SyntaxToken> + '_ {
        self.siblings_with_tokens(Direction::Prev)
            .skip(1)
            .map_while(|s| {
                if s.kind() == SyntaxKind::Whitespace || s.kind() == SyntaxKind::Comment {
                    s.into_token()
                } else {
                    None
                }
            })
            .filter(|t| t.kind() == SyntaxKind::Comment)
    }

    fn rule_exceptions(&self) -> HashSet<String> {
        let mut set = HashSet::default();
        for comment in self.except_comments() {
            if let Some(ids) = comment.text().strip_prefix(EXCEPT_COMMENT_PREFIX) {
                for id in ids.split(',') {
                    let id = id.trim();
                    set.insert(id.to_string());
                }
            }
        }

        set
    }

    fn is_rule_excepted(&self, id: &str) -> bool {
        for comment in self.except_comments() {
            if let Some(ids) = comment.text().strip_prefix(EXCEPT_COMMENT_PREFIX) {
                if ids.split(',').any(|i| i.trim() == id) {
                    return true;
                }
            }
        }

        false
    }
}

/// Represents the AST of a [Document].
///
/// See [Document::ast].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ast<N: TreeNode = SyntaxNode> {
    /// The WDL document specifies an unsupported version.
    Unsupported,
    /// The WDL document is V1.
    V1(v1::Ast<N>),
}

impl<N: TreeNode> Ast<N> {
    /// Gets the AST as a V1 AST.
    ///
    /// Returns `None` if the AST is not a V1 AST.
    pub fn as_v1(&self) -> Option<&v1::Ast<N>> {
        match self {
            Self::V1(ast) => Some(ast),
            _ => None,
        }
    }

    /// Consumes `self` and attempts to return the V1 AST.
    pub fn into_v1(self) -> Option<v1::Ast<N>> {
        match self {
            Self::V1(ast) => Some(ast),
            _ => None,
        }
    }

    /// Consumes `self` and attempts to return the V1 AST.
    ///
    /// # Panics
    ///
    /// Panics if the AST is not a V1 AST.
    pub fn unwrap_v1(self) -> v1::Ast<N> {
        self.into_v1().expect("the AST is not a V1 AST")
    }
}

/// Represents a single WDL document.
///
/// See [Document::ast] for getting a version-specific Abstract
/// Syntax Tree.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Document<N: TreeNode = SyntaxNode>(N);

impl<N: TreeNode> AstNode<N> for Document<N> {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::RootNode
    }

    fn cast(inner: N) -> Option<Self> {
        if Self::can_cast(inner.kind()) {
            Some(Self(inner))
        } else {
            None
        }
    }

    fn inner(&self) -> &N {
        &self.0
    }
}

impl Document {
    /// Parses a document from the given source.
    ///
    /// A document and its AST elements are trivially cloned.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use wdl_ast::{Document, AstToken, Ast};
    /// let (document, diagnostics) = Document::parse("version 1.1");
    /// assert!(diagnostics.is_empty());
    ///
    /// assert_eq!(
    ///     document
    ///         .version_statement()
    ///         .expect("should have version statement")
    ///         .version()
    ///         .text(),
    ///     "1.1"
    /// );
    ///
    /// match document.ast() {
    ///     Ast::V1(ast) => {
    ///         assert_eq!(ast.items().count(), 0);
    ///     }
    ///     Ast::Unsupported => panic!("should be a V1 AST"),
    /// }
    /// ```
    pub fn parse(source: &str) -> (Self, Vec<Diagnostic>) {
        let (tree, diagnostics) = SyntaxTree::parse(source);
        (
            Document::cast(tree.into_syntax()).expect("document should cast"),
            diagnostics,
        )
    }
}

impl<N: TreeNode> Document<N> {
    /// Gets the version statement of the document.
    ///
    /// This can be used to determine the version of the document that was
    /// parsed.
    ///
    /// A return value of `None` signifies a missing version statement.
    pub fn version_statement(&self) -> Option<VersionStatement<N>> {
        self.child()
    }

    /// Gets the AST representation of the document.
    pub fn ast(&self) -> Ast<N> {
        self.version_statement()
            .as_ref()
            .and_then(|s| s.version().text().parse::<SupportedVersion>().ok())
            .map(|v| match v {
                SupportedVersion::V1(_) => Ast::V1(v1::Ast(self.0.clone())),
                _ => Ast::Unsupported,
            })
            .unwrap_or(Ast::Unsupported)
    }

    /// Morphs a document of one node type to a document of a different node
    /// type.
    pub fn morph<U: TreeNode + NewRoot<N>>(self) -> Document<U> {
        Document(U::new_root(self.0))
    }
}

impl Document<SyntaxNode> {
    /// Visits the document with a pre-order traversal using the provided
    /// visitor to visit each element in the document.
    pub fn visit<V: Visitor>(&self, state: &mut V::State, visitor: &mut V) {
        visit(&self.0, state, visitor)
    }
}

impl fmt::Debug for Document {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Represents a whitespace token in the AST.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Whitespace<T: TreeToken = SyntaxToken>(T);

impl<T: TreeToken> AstToken<T> for Whitespace<T> {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::Whitespace
    }

    fn cast(inner: T) -> Option<Self> {
        match inner.kind() {
            SyntaxKind::Whitespace => Some(Self(inner)),
            _ => None,
        }
    }

    fn inner(&self) -> &T {
        &self.0
    }
}

/// Represents a comment token in the AST.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Comment<T: TreeToken = SyntaxToken>(T);

impl<T: TreeToken> AstToken<T> for Comment<T> {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::Comment
    }

    fn cast(inner: T) -> Option<Self> {
        match inner.kind() {
            SyntaxKind::Comment => Some(Self(inner)),
            _ => None,
        }
    }

    fn inner(&self) -> &T {
        &self.0
    }
}

/// Represents a version statement in a WDL AST.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VersionStatement<N: TreeNode = SyntaxNode>(N);

impl<N: TreeNode> VersionStatement<N> {
    /// Gets the version of the version statement.
    pub fn version(&self) -> Version<N::Token> {
        self.token()
            .expect("version statement must have a version token")
    }

    /// Gets the version keyword of the version statement.
    pub fn keyword(&self) -> v1::VersionKeyword<N::Token> {
        self.token()
            .expect("version statement must have a version keyword")
    }
}

impl<N: TreeNode> AstNode<N> for VersionStatement<N> {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::VersionStatementNode
    }

    fn cast(inner: N) -> Option<Self> {
        match inner.kind() {
            SyntaxKind::VersionStatementNode => Some(Self(inner)),
            _ => None,
        }
    }

    fn inner(&self) -> &N {
        &self.0
    }
}

/// Represents a version in the AST.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Version<T: TreeToken = SyntaxToken>(T);

impl<T: TreeToken> AstToken<T> for Version<T> {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::Version
    }

    fn cast(inner: T) -> Option<Self> {
        match inner.kind() {
            SyntaxKind::Version => Some(Self(inner)),
            _ => None,
        }
    }

    fn inner(&self) -> &T {
        &self.0
    }
}

/// Represents an identifier token.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident<T: TreeToken = SyntaxToken>(T);

impl<T: TreeToken> Ident<T> {
    /// Gets a hashable representation of the identifier.
    pub fn hashable(&self) -> TokenText<T> {
        TokenText(self.0.clone())
    }
}

impl<T: TreeToken> AstToken<T> for Ident<T> {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::Ident
    }

    fn cast(inner: T) -> Option<Self> {
        match inner.kind() {
            SyntaxKind::Ident => Some(Self(inner)),
            _ => None,
        }
    }

    fn inner(&self) -> &T {
        &self.0
    }
}

/// Helper for hashing tokens by their text.
///
/// Normally a token's equality and hash implementation work by comparing
/// the token's element in the tree; thus, two tokens with the same text
/// but different positions in the tree will compare and hash differently.
///
/// With this hash implementation, two tokens compare and hash identically if
/// their text is identical.
#[derive(Debug, Clone)]
pub struct TokenText<T: TreeToken = SyntaxToken>(T);

impl TokenText {
    /// Gets the text of the underlying token.
    pub fn text(&self) -> &str {
        self.0.text()
    }

    /// Gets the span of the underlying token.
    pub fn span(&self) -> Span {
        self.0.span()
    }
}

impl<T: TreeToken> PartialEq for TokenText<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.text() == other.0.text()
    }
}

impl<T: TreeToken> Eq for TokenText<T> {}

impl<T: TreeToken> std::hash::Hash for TokenText<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.text().hash(state);
    }
}

impl<T: TreeToken> std::borrow::Borrow<str> for TokenText<T> {
    fn borrow(&self) -> &str {
        self.0.text()
    }
}
