//! V1 AST representation for expressions.

use rowan::ast::support::child;
use rowan::ast::support::children;
use rowan::ast::AstChildren;
use rowan::ast::AstNode;
use rowan::NodeOrToken;
use wdl_grammar::experimental::tree::SyntaxKind;
use wdl_grammar::experimental::tree::SyntaxNode;
use wdl_grammar::experimental::tree::SyntaxToken;
use wdl_grammar::experimental::tree::WorkflowDescriptionLanguage;

use crate::experimental::token;
use crate::experimental::AstToken;
use crate::experimental::Ident;

/// Represents an expression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    /// The expression is a literal.
    Literal(LiteralExpr),
    /// The expression is a name reference.
    Name(NameRef),
    /// The expression is a parenthesized expression.
    Parenthesized(ParenthesizedExpr),
    /// The expression is an `if` expression.
    If(IfExpr),
    /// The expression is a "logical not" expression.
    LogicalNot(LogicalNotExpr),
    /// The expression is a negation expression.
    Negation(NegationExpr),
    /// The expression is a "logical or" expression.
    LogicalOr(LogicalOrExpr),
    /// The expression is a "logical and" expression.
    LogicalAnd(LogicalAndExpr),
    /// The expression is an equality expression.
    Equality(EqualityExpr),
    /// The expression is an inequality expression.
    Inequality(InequalityExpr),
    /// The expression is a "less than" expression.
    Less(LessExpr),
    /// The expression is a "less than or equal to" expression.
    LessEqual(LessEqualExpr),
    /// The expression is a "greater" expression.
    Greater(GreaterExpr),
    /// The expression is a "greater than or equal to" expression.
    GreaterEqual(GreaterEqualExpr),
    /// The expression is an addition expression.
    Addition(AdditionExpr),
    /// The expression is a subtraction expression.
    Subtraction(SubtractionExpr),
    /// The expression is a multiplication expression.
    Multiplication(MultiplicationExpr),
    /// The expression is a division expression.
    Division(DivisionExpr),
    /// The expression is a modulo expression.
    Modulo(ModuloExpr),
    /// The expression is a call expression.
    Call(CallExpr),
    /// The expression is an index expression.
    Index(IndexExpr),
    /// The expression is a member access expression.
    Access(AccessExpr),
}

impl Expr {
    /// Unwraps the expression into a literal expression.
    ///
    /// # Panics
    ///
    /// Panics if the expression is not a literal expression.
    pub fn unwrap_literal(self) -> LiteralExpr {
        match self {
            Self::Literal(expr) => expr,
            _ => panic!("not a literal expression"),
        }
    }

    /// Unwraps the expression into a name reference.
    ///
    /// # Panics
    ///
    /// Panics if the expression is not a name reference.
    pub fn unwrap_name_ref(self) -> NameRef {
        match self {
            Self::Name(expr) => expr,
            _ => panic!("not a name reference"),
        }
    }

    /// Unwraps the expression into a parenthesized expression.
    ///
    /// # Panics
    ///
    /// Panics if the expression is not a parenthesized expression.
    pub fn unwrap_parenthesized(self) -> ParenthesizedExpr {
        match self {
            Self::Parenthesized(expr) => expr,
            _ => panic!("not a parenthesized expression"),
        }
    }

    /// Unwraps the expression into an `if` expression.
    ///
    /// # Panics
    ///
    /// Panics if the expression is not an `if` expression.
    pub fn unwrap_if(self) -> IfExpr {
        match self {
            Self::If(expr) => expr,
            _ => panic!("not an `if` expression"),
        }
    }

    /// Unwraps the expression into a logical `not` expression.
    ///
    /// # Panics
    ///
    /// Panics if the expression is not a logical `not` expression.
    pub fn unwrap_logical_not(self) -> LogicalNotExpr {
        match self {
            Self::LogicalNot(expr) => expr,
            _ => panic!("not a logical `not` expression"),
        }
    }

    /// Unwraps the expression into a negation expression.
    ///
    /// # Panics
    ///
    /// Panics if the expression is not a negation expression.
    pub fn unwrap_negation(self) -> NegationExpr {
        match self {
            Self::Negation(expr) => expr,
            _ => panic!("not a negation expression"),
        }
    }

    /// Unwraps the expression into a logical `or` expression.
    ///
    /// # Panics
    ///
    /// Panics if the expression is not a logical `or` expression.
    pub fn unwrap_logical_or(self) -> LogicalOrExpr {
        match self {
            Self::LogicalOr(expr) => expr,
            _ => panic!("not a logical `or` expression"),
        }
    }

    /// Unwraps the expression into a logical `and` expression.
    ///
    /// # Panics
    ///
    /// Panics if the expression is not a logical `and` expression.
    pub fn unwrap_logical_and(self) -> LogicalAndExpr {
        match self {
            Self::LogicalAnd(expr) => expr,
            _ => panic!("not a logical `and` expression"),
        }
    }

    /// Unwraps the expression into an equality expression.
    ///
    /// # Panics
    ///
    /// Panics if the expression is not an equality expression.
    pub fn unwrap_equality(self) -> EqualityExpr {
        match self {
            Self::Equality(expr) => expr,
            _ => panic!("not an equality expression"),
        }
    }

    /// Unwraps the expression into an inequality expression.
    ///
    /// # Panics
    ///
    /// Panics if the expression is not an inequality expression.
    pub fn unwrap_inequality(self) -> InequalityExpr {
        match self {
            Self::Inequality(expr) => expr,
            _ => panic!("not an inequality expression"),
        }
    }

    /// Unwraps the expression into a "less than" expression.
    ///
    /// # Panics
    ///
    /// Panics if the expression is not a "less than" expression.
    pub fn unwrap_less(self) -> LessExpr {
        match self {
            Self::Less(expr) => expr,
            _ => panic!("not a \"less than\" expression"),
        }
    }

    /// Unwraps the expression into a "less than or equal to" expression.
    ///
    /// # Panics
    ///
    /// Panics if the expression is not a "less than or equal to" expression.
    pub fn unwrap_less_equal(self) -> LessEqualExpr {
        match self {
            Self::LessEqual(expr) => expr,
            _ => panic!("not a \"less than or equal to\" expression"),
        }
    }

    /// Unwraps the expression into a "greater than" expression.
    ///
    /// # Panics
    ///
    /// Panics if the expression is not a "greater than" expression.
    pub fn unwrap_greater(self) -> GreaterExpr {
        match self {
            Self::Greater(expr) => expr,
            _ => panic!("not a \"greater than\" expression"),
        }
    }

    /// Unwraps the expression into a "greater than or equal to" expression.
    ///
    /// # Panics
    ///
    /// Panics if the expression is not a "greater than or equal to" expression.
    pub fn unwrap_greater_equal(self) -> GreaterEqualExpr {
        match self {
            Self::GreaterEqual(expr) => expr,
            _ => panic!("not a \"greater than or equal to\" expression"),
        }
    }

    /// Unwraps the expression into an addition expression.
    ///
    /// # Panics
    ///
    /// Panics if the expression is not an addition expression.
    pub fn unwrap_addition(self) -> AdditionExpr {
        match self {
            Self::Addition(expr) => expr,
            _ => panic!("not an addition expression"),
        }
    }

    /// Unwraps the expression into a subtraction expression.
    ///
    /// # Panics
    ///
    /// Panics if the expression is not a subtraction expression.
    pub fn unwrap_subtraction(self) -> SubtractionExpr {
        match self {
            Self::Subtraction(expr) => expr,
            _ => panic!("not a subtraction expression"),
        }
    }

    /// Unwraps the expression into a multiplication expression.
    ///
    /// # Panics
    ///
    /// Panics if the expression is not a multiplication expression.
    pub fn unwrap_multiplication(self) -> MultiplicationExpr {
        match self {
            Self::Multiplication(expr) => expr,
            _ => panic!("not a multiplication expression"),
        }
    }

    /// Unwraps the expression into a division expression.
    ///
    /// # Panics
    ///
    /// Panics if the expression is not a division expression.
    pub fn unwrap_division(self) -> DivisionExpr {
        match self {
            Self::Division(expr) => expr,
            _ => panic!("not a division expression"),
        }
    }

    /// Unwraps the expression into a modulo expression.
    ///
    /// # Panics
    ///
    /// Panics if the expression is not a modulo expression.
    pub fn unwrap_modulo(self) -> ModuloExpr {
        match self {
            Self::Modulo(expr) => expr,
            _ => panic!("not a modulo expression"),
        }
    }

    /// Unwraps the expression into a call expression.
    ///
    /// # Panics
    ///
    /// Panics if the expression is not a call expression.
    pub fn unwrap_call(self) -> CallExpr {
        match self {
            Self::Call(expr) => expr,
            _ => panic!("not a call expression"),
        }
    }

    /// Unwraps the expression into an index expression.
    ///
    /// # Panics
    ///
    /// Panics if the expression is not an index expression.
    pub fn unwrap_index(self) -> IndexExpr {
        match self {
            Self::Index(expr) => expr,
            _ => panic!("not an index expression"),
        }
    }

    /// Unwraps the expression into an access expression.
    ///
    /// # Panics
    ///
    /// Panics if the expression is not an access expression.
    pub fn unwrap_access(self) -> AccessExpr {
        match self {
            Self::Access(expr) => expr,
            _ => panic!("not an access expression"),
        }
    }
}

impl AstNode for Expr {
    type Language = WorkflowDescriptionLanguage;

    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized,
    {
        if LiteralExpr::can_cast(kind) {
            return true;
        }

        matches!(
            kind,
            SyntaxKind::NameRefNode
                | SyntaxKind::ParenthesizedExprNode
                | SyntaxKind::IfExprNode
                | SyntaxKind::LogicalNotExprNode
                | SyntaxKind::NegationExprNode
                | SyntaxKind::LogicalOrExprNode
                | SyntaxKind::LogicalAndExprNode
                | SyntaxKind::EqualityExprNode
                | SyntaxKind::InequalityExprNode
                | SyntaxKind::LessExprNode
                | SyntaxKind::LessEqualExprNode
                | SyntaxKind::GreaterExprNode
                | SyntaxKind::GreaterEqualExprNode
                | SyntaxKind::AdditionExprNode
                | SyntaxKind::SubtractionExprNode
                | SyntaxKind::MultiplicationExprNode
                | SyntaxKind::DivisionExprNode
                | SyntaxKind::ModuloExprNode
                | SyntaxKind::CallExprNode
                | SyntaxKind::IndexExprNode
                | SyntaxKind::AccessExprNode
        )
    }

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        if LiteralExpr::can_cast(syntax.kind()) {
            return LiteralExpr::cast(syntax).map(Self::Literal);
        }

        match syntax.kind() {
            SyntaxKind::NameRefNode => Some(Self::Name(NameRef(syntax))),
            SyntaxKind::ParenthesizedExprNode => {
                Some(Self::Parenthesized(ParenthesizedExpr(syntax)))
            }
            SyntaxKind::IfExprNode => Some(Self::If(IfExpr(syntax))),
            SyntaxKind::LogicalNotExprNode => Some(Self::LogicalNot(LogicalNotExpr(syntax))),
            SyntaxKind::NegationExprNode => Some(Self::Negation(NegationExpr(syntax))),
            SyntaxKind::LogicalOrExprNode => Some(Self::LogicalOr(LogicalOrExpr(syntax))),
            SyntaxKind::LogicalAndExprNode => Some(Self::LogicalAnd(LogicalAndExpr(syntax))),
            SyntaxKind::EqualityExprNode => Some(Self::Equality(EqualityExpr(syntax))),
            SyntaxKind::InequalityExprNode => Some(Self::Inequality(InequalityExpr(syntax))),
            SyntaxKind::LessExprNode => Some(Self::Less(LessExpr(syntax))),
            SyntaxKind::LessEqualExprNode => Some(Self::LessEqual(LessEqualExpr(syntax))),
            SyntaxKind::GreaterExprNode => Some(Self::Greater(GreaterExpr(syntax))),
            SyntaxKind::GreaterEqualExprNode => Some(Self::GreaterEqual(GreaterEqualExpr(syntax))),
            SyntaxKind::AdditionExprNode => Some(Self::Addition(AdditionExpr(syntax))),
            SyntaxKind::SubtractionExprNode => Some(Self::Subtraction(SubtractionExpr(syntax))),
            SyntaxKind::MultiplicationExprNode => {
                Some(Self::Multiplication(MultiplicationExpr(syntax)))
            }
            SyntaxKind::DivisionExprNode => Some(Self::Division(DivisionExpr(syntax))),
            SyntaxKind::ModuloExprNode => Some(Self::Modulo(ModuloExpr(syntax))),
            SyntaxKind::CallExprNode => Some(Self::Call(CallExpr(syntax))),
            SyntaxKind::IndexExprNode => Some(Self::Index(IndexExpr(syntax))),
            SyntaxKind::AccessExprNode => Some(Self::Access(AccessExpr(syntax))),
            _ => None,
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        match self {
            Self::Literal(l) => l.syntax(),
            Self::Name(n) => &n.0,
            Self::Parenthesized(p) => &p.0,
            Self::If(i) => &i.0,
            Self::LogicalNot(n) => &n.0,
            Self::Negation(n) => &n.0,
            Self::LogicalOr(o) => &o.0,
            Self::LogicalAnd(a) => &a.0,
            Self::Equality(e) => &e.0,
            Self::Inequality(i) => &i.0,
            Self::Less(l) => &l.0,
            Self::LessEqual(l) => &l.0,
            Self::Greater(g) => &g.0,
            Self::GreaterEqual(g) => &g.0,
            Self::Addition(a) => &a.0,
            Self::Subtraction(s) => &s.0,
            Self::Multiplication(m) => &m.0,
            Self::Division(d) => &d.0,
            Self::Modulo(m) => &m.0,
            Self::Call(c) => &c.0,
            Self::Index(i) => &i.0,
            Self::Access(a) => &a.0,
        }
    }
}

/// Represents a literal expression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LiteralExpr {
    /// The literal is a `Boolean`.
    Boolean(LiteralBoolean),
    /// The literal is an `Int`.
    Integer(LiteralInteger),
    /// The literal is a `Float`.
    Float(LiteralFloat),
    /// The literal is a `String`.
    String(LiteralString),
    /// The literal is an `Array`.
    Array(LiteralArray),
    /// The literal is a `Pair`.
    Pair(LiteralPair),
    /// The literal is a `Map`.
    Map(LiteralMap),
    /// The literal is an `Object`.
    Object(LiteralObject),
    /// The literal is a struct.
    Struct(LiteralStruct),
    /// The literal is a `None`.
    None(LiteralNone),
}

impl LiteralExpr {
    /// Unwraps the expression into a literal boolean.
    ///
    /// # Panics
    ///
    /// Panics if the expression is not a literal boolean.
    pub fn unwrap_boolean(self) -> LiteralBoolean {
        match self {
            Self::Boolean(literal) => literal,
            _ => panic!("not a literal boolean"),
        }
    }

    /// Unwraps the expression into a literal integer.
    ///
    /// # Panics
    ///
    /// Panics if the expression is not a literal integer.
    pub fn unwrap_integer(self) -> LiteralInteger {
        match self {
            Self::Integer(literal) => literal,
            _ => panic!("not a literal integer"),
        }
    }

    /// Unwraps the expression into a literal float.
    ///
    /// # Panics
    ///
    /// Panics if the expression is not a literal float.
    pub fn unwrap_float(self) -> LiteralFloat {
        match self {
            Self::Float(literal) => literal,
            _ => panic!("not a literal float"),
        }
    }

    /// Unwraps the expression into a literal string.
    ///
    /// # Panics
    ///
    /// Panics if the expression is not a literal string.
    pub fn unwrap_string(self) -> LiteralString {
        match self {
            Self::String(literal) => literal,
            _ => panic!("not a literal string"),
        }
    }

    /// Unwraps the expression into a literal array.
    ///
    /// # Panics
    ///
    /// Panics if the expression is not a literal array.
    pub fn unwrap_array(self) -> LiteralArray {
        match self {
            Self::Array(literal) => literal,
            _ => panic!("not a literal array"),
        }
    }

    /// Unwraps the expression into a literal pair.
    ///
    /// # Panics
    ///
    /// Panics if the expression is not a literal pair.
    pub fn unwrap_pair(self) -> LiteralPair {
        match self {
            Self::Pair(literal) => literal,
            _ => panic!("not a literal pair"),
        }
    }

    /// Unwraps the expression into a literal map.
    ///
    /// # Panics
    ///
    /// Panics if the expression is not a literal map.
    pub fn unwrap_map(self) -> LiteralMap {
        match self {
            Self::Map(literal) => literal,
            _ => panic!("not a literal map"),
        }
    }

    /// Unwraps the expression into a literal object.
    ///
    /// # Panics
    ///
    /// Panics if the expression is not a literal object.
    pub fn unwrap_object(self) -> LiteralObject {
        match self {
            Self::Object(literal) => literal,
            _ => panic!("not a literal object"),
        }
    }

    /// Unwraps the expression into a literal struct.
    ///
    /// # Panics
    ///
    /// Panics if the expression is not a literal struct.
    pub fn unwrap_struct(self) -> LiteralStruct {
        match self {
            Self::Struct(literal) => literal,
            _ => panic!("not a literal struct"),
        }
    }

    /// Unwraps the expression into a literal `None`.
    ///
    /// # Panics
    ///
    /// Panics if the expression is not a literal `None`.
    pub fn unwrap_none(self) -> LiteralNone {
        match self {
            Self::None(literal) => literal,
            _ => panic!("not a literal `None`"),
        }
    }
}

impl AstNode for LiteralExpr {
    type Language = WorkflowDescriptionLanguage;

    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized,
    {
        matches!(
            kind,
            SyntaxKind::LiteralBooleanNode
                | SyntaxKind::LiteralIntegerNode
                | SyntaxKind::LiteralFloatNode
                | SyntaxKind::LiteralStringNode
                | SyntaxKind::LiteralArrayNode
                | SyntaxKind::LiteralPairNode
                | SyntaxKind::LiteralMapNode
                | SyntaxKind::LiteralObjectNode
                | SyntaxKind::LiteralStructNode
                | SyntaxKind::LiteralNoneNode
        )
    }

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        match syntax.kind() {
            SyntaxKind::LiteralBooleanNode => Some(Self::Boolean(LiteralBoolean(syntax))),
            SyntaxKind::LiteralIntegerNode => Some(Self::Integer(LiteralInteger(syntax))),
            SyntaxKind::LiteralFloatNode => Some(Self::Float(LiteralFloat(syntax))),
            SyntaxKind::LiteralStringNode => Some(Self::String(LiteralString(syntax))),
            SyntaxKind::LiteralArrayNode => Some(Self::Array(LiteralArray(syntax))),
            SyntaxKind::LiteralPairNode => Some(Self::Pair(LiteralPair(syntax))),
            SyntaxKind::LiteralMapNode => Some(Self::Map(LiteralMap(syntax))),
            SyntaxKind::LiteralObjectNode => Some(Self::Object(LiteralObject(syntax))),
            SyntaxKind::LiteralStructNode => Some(Self::Struct(LiteralStruct(syntax))),
            SyntaxKind::LiteralNoneNode => Some(Self::None(LiteralNone(syntax))),
            _ => None,
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        match self {
            Self::Boolean(b) => &b.0,
            Self::Integer(i) => &i.0,
            Self::Float(f) => &f.0,
            Self::String(s) => &s.0,
            Self::Array(a) => &a.0,
            Self::Pair(p) => &p.0,
            Self::Map(m) => &m.0,
            Self::Object(o) => &o.0,
            Self::Struct(s) => &s.0,
            Self::None(n) => &n.0,
        }
    }
}

/// Represents a literal boolean.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiteralBoolean(pub(super) SyntaxNode);

impl LiteralBoolean {
    /// Gets the value of the literal boolean.
    pub fn value(&self) -> bool {
        self.0
            .children_with_tokens()
            .find_map(|c| match c.kind() {
                SyntaxKind::TrueKeyword => Some(true),
                SyntaxKind::FalseKeyword => Some(false),
                _ => None,
            })
            .expect("`true` or `false` keyword should be present")
    }
}

impl AstNode for LiteralBoolean {
    type Language = WorkflowDescriptionLanguage;

    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == SyntaxKind::LiteralBooleanNode
    }

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        match syntax.kind() {
            SyntaxKind::LiteralBooleanNode => Some(Self(syntax)),
            _ => None,
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

/// Represents a literal integer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiteralInteger(pub(super) SyntaxNode);

impl LiteralInteger {
    /// Gets the value of the literal integer.
    ///
    /// A literal integer in WDL is a *signed* 64-bit integer.
    ///
    /// However, as the minimum signed 64-bit integer is
    /// `-9223372036854775808` and a literal of `9223372036854775808`
    /// exceeds the maximum signed 64-bit integer, this returns an
    /// unsigned 64-bit integer as the negation is not part of the
    /// literal.
    ///
    /// Validation will check to make sure that the value does not
    /// exceed the range of a signed 64-bit integer, taking into account
    /// a prefix negation expression, if any.
    ///
    /// Returns `None` if the literal value is out of range (excluding
    /// `9223372036854775808`, which will only be accepted as an
    /// operand to a negation expression).
    pub fn value(&self) -> Option<u64> {
        let token = self
            .0
            .children_with_tokens()
            .filter_map(NodeOrToken::into_token)
            .find(|t| t.kind() == SyntaxKind::Integer)
            .expect("integer token should be present");

        let text = token.text();
        let i = if text == "0" {
            0
        } else if text.starts_with("0x") || text.starts_with("0X") {
            u64::from_str_radix(&text[2..], 16).ok()?
        } else if text.starts_with('0') {
            u64::from_str_radix(text, 8).ok()?
        } else {
            text.parse::<u64>().ok()?
        };

        if i > (i64::MAX as u64) + 1 {
            None
        } else {
            Some(i)
        }
    }
}

impl AstNode for LiteralInteger {
    type Language = WorkflowDescriptionLanguage;

    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == SyntaxKind::LiteralIntegerNode
    }

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        match syntax.kind() {
            SyntaxKind::LiteralIntegerNode => Some(Self(syntax)),
            _ => None,
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

/// Represents a literal float.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiteralFloat(pub(crate) SyntaxNode);

impl LiteralFloat {
    /// Gets the value of the literal float.
    ///
    /// Returns `None` if the literal value is not in range.
    pub fn value(&self) -> Option<f64> {
        let token = self
            .0
            .children_with_tokens()
            .filter_map(NodeOrToken::into_token)
            .find(|t| t.kind() == SyntaxKind::Float)
            .expect("integer token should be present");

        token
            .text()
            .parse()
            .ok()
            .and_then(|f: f64| if f.is_infinite() { None } else { Some(f) })
    }
}

impl AstNode for LiteralFloat {
    type Language = WorkflowDescriptionLanguage;

    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == SyntaxKind::LiteralFloatNode
    }

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        match syntax.kind() {
            SyntaxKind::LiteralFloatNode => Some(Self(syntax)),
            _ => None,
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

/// Represents a literal string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiteralString(pub(super) SyntaxNode);

impl LiteralString {
    /// Gets the quote character of the literal string.
    pub fn quote(&self) -> char {
        self.0
            .children_with_tokens()
            .find_map(|c| match c.kind() {
                SyntaxKind::SingleQuote => Some('\''),
                SyntaxKind::DoubleQuote => Some('"'),
                _ => None,
            })
            .expect("string is missing quote tokens")
    }

    /// Gets the parts of the string.
    ///
    /// A part may be literal text or an interpolated expression.
    pub fn parts(&self) -> impl Iterator<Item = StringPart> {
        self.0.children_with_tokens().filter_map(StringPart::cast)
    }

    /// Gets the string text if the string is not interpolated (i.e.
    /// has no placeholders).
    ///
    /// Returns `None` if the string is interpolated, as
    /// interpolated strings cannot be represented as a single
    /// span of text.
    pub fn text(&self) -> Option<StringText> {
        let mut parts = self.parts();
        if let Some(StringPart::Text(text)) = parts.next() {
            if parts.next().is_none() {
                return Some(text);
            }
        }

        None
    }
}

impl AstNode for LiteralString {
    type Language = WorkflowDescriptionLanguage;

    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == SyntaxKind::LiteralStringNode
    }

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        match syntax.kind() {
            SyntaxKind::LiteralStringNode => Some(Self(syntax)),
            _ => None,
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

/// Represents a part of a string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StringPart {
    /// A textual part of the string.
    Text(StringText),
    /// A placeholder encountered in the string.
    Placeholder(Placeholder),
}

impl StringPart {
    /// Unwraps the string part into text.
    ///
    /// # Panics
    ///
    /// Panics if the string part is not text.
    pub fn unwrap_text(self) -> StringText {
        match self {
            Self::Text(text) => text,
            _ => panic!("not string text"),
        }
    }

    /// Unwraps the string part into a placeholder.
    ///
    /// # Panics
    ///
    /// Panics if the string part is not a placeholder.
    pub fn unwrap_placeholder(self) -> Placeholder {
        match self {
            Self::Placeholder(p) => p,
            _ => panic!("not a placeholder"),
        }
    }

    /// Casts the given syntax node or token to a string part.
    fn cast(syntax: NodeOrToken<SyntaxNode, SyntaxToken>) -> Option<Self> {
        match syntax {
            NodeOrToken::Node(n) => Some(Self::Placeholder(Placeholder::cast(n)?)),
            NodeOrToken::Token(t) => Some(Self::Text(StringText::cast(t)?)),
        }
    }
}

/// Represents a textual part of a string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StringText(SyntaxToken);

impl AstToken for StringText {
    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == SyntaxKind::LiteralStringText
    }

    fn cast(syntax: SyntaxToken) -> Option<Self>
    where
        Self: Sized,
    {
        match syntax.kind() {
            SyntaxKind::LiteralStringText => Some(Self(syntax)),
            _ => None,
        }
    }

    fn syntax(&self) -> &SyntaxToken {
        &self.0
    }
}

/// Represents a placeholder in a string or command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Placeholder(pub(super) SyntaxNode);

impl Placeholder {
    /// Gets the options for the placeholder.
    pub fn options(&self) -> AstChildren<PlaceholderOption> {
        children(&self.0)
    }

    /// Gets the placeholder expression.
    pub fn expr(&self) -> Expr {
        child(&self.0).expect("placeholder should have an expression")
    }
}

impl AstNode for Placeholder {
    type Language = WorkflowDescriptionLanguage;

    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == SyntaxKind::PlaceholderNode
    }

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        match syntax.kind() {
            SyntaxKind::PlaceholderNode => Some(Self(syntax)),
            _ => None,
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

/// Represents a placeholder option.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlaceholderOption {
    /// A `sep` option for specifying a delimiter for formatting arrays.
    Sep(SepOption),
    /// A `default` option for substituting a default value for an undefined
    /// expression.
    Default(DefaultOption),
    /// A `true/false` option for substituting a value depending on whether a
    /// boolean expression is true or false.
    TrueFalse(TrueFalseOption),
}

impl PlaceholderOption {
    /// Unwraps the option into a separator option.
    ///
    /// # Panics
    ///
    /// Panics if the option is not a separator option.
    pub fn unwrap_sep(self) -> SepOption {
        match self {
            Self::Sep(opt) => opt,
            _ => panic!("not a separator option"),
        }
    }

    /// Unwraps the option into a default option.
    ///
    /// # Panics
    ///
    /// Panics if the option is not a default option.
    pub fn unwrap_default(self) -> DefaultOption {
        match self {
            Self::Default(opt) => opt,
            _ => panic!("not a default option"),
        }
    }

    /// Unwraps the option into a true/false option.
    ///
    /// # Panics
    ///
    /// Panics if the option is not a true/false option.
    pub fn unwrap_true_false(self) -> TrueFalseOption {
        match self {
            Self::TrueFalse(opt) => opt,
            _ => panic!("not a true/false option"),
        }
    }
}

impl AstNode for PlaceholderOption {
    type Language = WorkflowDescriptionLanguage;

    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized,
    {
        matches!(
            kind,
            SyntaxKind::PlaceholderSepOptionNode
                | SyntaxKind::PlaceholderDefaultOptionNode
                | SyntaxKind::PlaceholderTrueFalseOptionNode
        )
    }

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        match syntax.kind() {
            SyntaxKind::PlaceholderSepOptionNode => Some(Self::Sep(SepOption(syntax))),
            SyntaxKind::PlaceholderDefaultOptionNode => Some(Self::Default(DefaultOption(syntax))),
            SyntaxKind::PlaceholderTrueFalseOptionNode => {
                Some(Self::TrueFalse(TrueFalseOption(syntax)))
            }
            _ => None,
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        match self {
            Self::Sep(s) => &s.0,
            Self::Default(d) => &d.0,
            Self::TrueFalse(tf) => &tf.0,
        }
    }
}

/// Represents a `sep` option for a placeholder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SepOption(SyntaxNode);

impl SepOption {
    /// Gets the separator to use for formatting an array.
    pub fn separator(&self) -> LiteralString {
        child(&self.0).expect("sep option should have a string literal")
    }
}

impl AstNode for SepOption {
    type Language = WorkflowDescriptionLanguage;

    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == SyntaxKind::PlaceholderSepOptionNode
    }

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        match syntax.kind() {
            SyntaxKind::PlaceholderSepOptionNode => Some(Self(syntax)),
            _ => None,
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

/// Represents a `default` option for a placeholder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DefaultOption(SyntaxNode);

impl DefaultOption {
    /// Gets the value to use for an undefined expression.
    pub fn value(&self) -> LiteralString {
        child(&self.0).expect("default option should have a string literal")
    }
}

impl AstNode for DefaultOption {
    type Language = WorkflowDescriptionLanguage;

    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == SyntaxKind::PlaceholderDefaultOptionNode
    }

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        match syntax.kind() {
            SyntaxKind::PlaceholderDefaultOptionNode => Some(Self(syntax)),
            _ => None,
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

/// Represents a `true/false` option for a placeholder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrueFalseOption(SyntaxNode);

impl TrueFalseOption {
    /// Gets the `true` and `false`` values to use for a placeholder
    /// expression that evaluates to a boolean.
    ///
    /// The first value returned is the `true` value and the second
    /// value is the `false` value.
    pub fn values(&self) -> (LiteralString, LiteralString) {
        let mut true_value = None;
        let mut false_value = None;
        let mut found = None;
        let mut children = self.0.children_with_tokens();
        for child in children.by_ref() {
            match child.kind() {
                SyntaxKind::TrueKeyword => {
                    found = Some(true);
                }
                SyntaxKind::FalseKeyword => {
                    found = Some(false);
                }
                k if LiteralString::can_cast(k) => {
                    let child = child.into_node().expect("should be a node");
                    if found.expect("should have found true or false") {
                        assert!(true_value.is_none(), "multiple true values present");
                        true_value = Some(LiteralString(child));
                    } else {
                        assert!(false_value.is_none(), "multiple false values present");
                        false_value = Some(LiteralString(child));
                    }

                    if true_value.is_some() && false_value.is_some() {
                        break;
                    }
                }
                _ => continue,
            }
        }

        (
            true_value.expect("expected a true value to be present"),
            false_value.expect("expected a false value to be present`"),
        )
    }
}

impl AstNode for TrueFalseOption {
    type Language = WorkflowDescriptionLanguage;

    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == SyntaxKind::PlaceholderTrueFalseOptionNode
    }

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        match syntax.kind() {
            SyntaxKind::PlaceholderTrueFalseOptionNode => Some(Self(syntax)),
            _ => None,
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

/// Represents a literal array.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiteralArray(pub(super) SyntaxNode);

impl LiteralArray {
    /// Gets the elements of the literal array.
    pub fn elements(&self) -> AstChildren<Expr> {
        children(&self.0)
    }
}

impl AstNode for LiteralArray {
    type Language = WorkflowDescriptionLanguage;

    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == SyntaxKind::LiteralArrayNode
    }

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        match syntax.kind() {
            SyntaxKind::LiteralArrayNode => Some(Self(syntax)),
            _ => None,
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

/// Represents a literal pair.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiteralPair(pub(super) SyntaxNode);

impl LiteralPair {
    /// Gets the first and second expressions in the literal pair.
    pub fn exprs(&self) -> (Expr, Expr) {
        let mut children = self.0.children().filter_map(Expr::cast);
        let first = children
            .next()
            .expect("pair should have a first expression");
        let second = children
            .next()
            .expect("pair should have a second expression");
        (first, second)
    }
}

impl AstNode for LiteralPair {
    type Language = WorkflowDescriptionLanguage;

    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == SyntaxKind::LiteralPairNode
    }

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        match syntax.kind() {
            SyntaxKind::LiteralPairNode => Some(Self(syntax)),
            _ => None,
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

/// Represents a literal map.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiteralMap(pub(super) SyntaxNode);

impl LiteralMap {
    /// Gets the items of the literal map.
    pub fn items(&self) -> AstChildren<LiteralMapItem> {
        children(&self.0)
    }
}

impl AstNode for LiteralMap {
    type Language = WorkflowDescriptionLanguage;

    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == SyntaxKind::LiteralMapNode
    }

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        match syntax.kind() {
            SyntaxKind::LiteralMapNode => Some(Self(syntax)),
            _ => None,
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

/// Represents a literal map item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiteralMapItem(pub(super) SyntaxNode);

impl LiteralMapItem {
    /// Gets the key and the value of the item.
    pub fn key_value(&self) -> (Expr, Expr) {
        let mut children = self.0.children().filter_map(Expr::cast);
        let key = children.next().expect("expected a key expression");
        let value = children.next().expect("expected a value expression");
        (key, value)
    }
}

impl AstNode for LiteralMapItem {
    type Language = WorkflowDescriptionLanguage;

    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == SyntaxKind::LiteralMapItemNode
    }

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        match syntax.kind() {
            SyntaxKind::LiteralMapItemNode => Some(Self(syntax)),
            _ => None,
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

/// Represents a literal object.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiteralObject(pub(super) SyntaxNode);

impl LiteralObject {
    /// Gets the items of the literal object.
    pub fn items(&self) -> AstChildren<LiteralObjectItem> {
        children(&self.0)
    }
}

impl AstNode for LiteralObject {
    type Language = WorkflowDescriptionLanguage;

    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == SyntaxKind::LiteralObjectNode
    }

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        match syntax.kind() {
            SyntaxKind::LiteralObjectNode => Some(Self(syntax)),
            _ => None,
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

/// Gets the name and value of a object or struct literal item.
fn name_value(parent: &SyntaxNode) -> (Ident, Expr) {
    let mut children = parent
        .children_with_tokens()
        .filter(|c| Ident::can_cast(c.kind()) || Expr::can_cast(c.kind()));
    let key = Ident::cast(
        children
            .next()
            .expect("expected a key token")
            .into_token()
            .expect("key should be a token"),
    )
    .expect("token should cast to ident");
    let value = Expr::cast(
        children
            .next()
            .expect("there should be a value expression")
            .into_node()
            .expect("value should be a node"),
    )
    .expect("node should cast to an expression");
    (key, value)
}

/// Represents a literal object item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiteralObjectItem(pub(super) SyntaxNode);

impl LiteralObjectItem {
    /// Gets the name and the value of the item.
    pub fn name_value(&self) -> (Ident, Expr) {
        name_value(&self.0)
    }
}

impl AstNode for LiteralObjectItem {
    type Language = WorkflowDescriptionLanguage;

    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == SyntaxKind::LiteralObjectItemNode
    }

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        match syntax.kind() {
            SyntaxKind::LiteralObjectItemNode => Some(Self(syntax)),
            _ => None,
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

/// Represents a literal struct.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiteralStruct(pub(super) SyntaxNode);

impl LiteralStruct {
    /// Gets the name of the struct.
    pub fn name(&self) -> Ident {
        token(&self.0).expect("expected the struct to have a name")
    }

    /// Gets the items of the literal struct.
    pub fn items(&self) -> AstChildren<LiteralStructItem> {
        children(&self.0)
    }
}

impl AstNode for LiteralStruct {
    type Language = WorkflowDescriptionLanguage;

    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == SyntaxKind::LiteralStructNode
    }

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        match syntax.kind() {
            SyntaxKind::LiteralStructNode => Some(Self(syntax)),
            _ => None,
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

/// Represents a literal struct item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiteralStructItem(pub(super) SyntaxNode);

impl LiteralStructItem {
    /// Gets the name and the value of the item.
    pub fn name_value(&self) -> (Ident, Expr) {
        name_value(&self.0)
    }
}

impl AstNode for LiteralStructItem {
    type Language = WorkflowDescriptionLanguage;

    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == SyntaxKind::LiteralStructItemNode
    }

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        match syntax.kind() {
            SyntaxKind::LiteralStructItemNode => Some(Self(syntax)),
            _ => None,
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

/// Represents a literal `None`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiteralNone(pub(super) SyntaxNode);

impl AstNode for LiteralNone {
    type Language = WorkflowDescriptionLanguage;

    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == SyntaxKind::LiteralNoneNode
    }

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        match syntax.kind() {
            SyntaxKind::LiteralNoneNode => Some(Self(syntax)),
            _ => None,
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

/// Represents a reference to a name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NameRef(pub(super) SyntaxNode);

impl NameRef {
    /// Gets the name being referenced.
    pub fn name(&self) -> Ident {
        token(&self.0).expect("expected a name")
    }
}

impl AstNode for NameRef {
    type Language = WorkflowDescriptionLanguage;

    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == SyntaxKind::NameRefNode
    }

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        match syntax.kind() {
            SyntaxKind::NameRefNode => Some(Self(syntax)),
            _ => None,
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

/// Represents a parenthesized expression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParenthesizedExpr(pub(super) SyntaxNode);

impl ParenthesizedExpr {
    /// Gets the inner expression.
    pub fn inner(&self) -> Expr {
        child(&self.0).expect("expected an inner expression")
    }
}

impl AstNode for ParenthesizedExpr {
    type Language = WorkflowDescriptionLanguage;

    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == SyntaxKind::ParenthesizedExprNode
    }

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        match syntax.kind() {
            SyntaxKind::ParenthesizedExprNode => Some(Self(syntax)),
            _ => None,
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

/// Represents an `if` expression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IfExpr(pub(super) SyntaxNode);

impl IfExpr {
    /// Gets the three expressions of the `if` expression
    ///
    /// The first expression is the conditional.
    /// The second expression is the `true` expression.
    /// The third expression is the `false` expression.
    pub fn exprs(&self) -> (Expr, Expr, Expr) {
        let mut children = self.0.children().filter_map(Expr::cast);
        let conditional = children
            .next()
            .expect("should have a conditional expression");
        let true_expr = children.next().expect("should have a `true` expression");
        let false_expr = children.next().expect("should have a `false` expression");
        (conditional, true_expr, false_expr)
    }
}

impl AstNode for IfExpr {
    type Language = WorkflowDescriptionLanguage;

    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == SyntaxKind::IfExprNode
    }

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        match syntax.kind() {
            SyntaxKind::IfExprNode => Some(Self(syntax)),
            _ => None,
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

/// Used to declare a prefix expression.
macro_rules! prefix_expression {
    ($name:ident, $kind:ident, $desc:literal) => {
        #[doc = concat!("Represents a ", $desc, " expression.")]
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct $name(pub(super) SyntaxNode);

        impl $name {
            /// Gets the operand expression.
            pub fn operand(&self) -> Expr {
                child(&self.0).expect("expected an operand expression")
            }
        }

        impl AstNode for $name {
            type Language = WorkflowDescriptionLanguage;

            fn can_cast(kind: SyntaxKind) -> bool
            where
                Self: Sized,
            {
                kind == SyntaxKind::$kind
            }

            fn cast(syntax: SyntaxNode) -> Option<Self>
            where
                Self: Sized,
            {
                match syntax.kind() {
                    SyntaxKind::$kind => Some(Self(syntax)),
                    _ => None,
                }
            }

            fn syntax(&self) -> &SyntaxNode {
                &self.0
            }
        }
    };
}

/// Used to declare an infix expression.
macro_rules! infix_expression {
    ($name:ident, $kind:ident, $desc:literal) => {
        #[doc = concat!("Represents a ", $desc, " expression.")]
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct $name(pub(super) SyntaxNode);

        impl $name {
            /// Gets the operands of the expression.
            pub fn operands(&self) -> (Expr, Expr) {
                let mut children = self.0.children().filter_map(Expr::cast);
                let lhs = children.next().expect("expected a lhs expression");
                let rhs = children.next().expect("expected a rhs expression");
                (lhs, rhs)
            }
        }

        impl AstNode for $name {
            type Language = WorkflowDescriptionLanguage;

            fn can_cast(kind: SyntaxKind) -> bool
            where
                Self: Sized,
            {
                kind == SyntaxKind::$kind
            }

            fn cast(syntax: SyntaxNode) -> Option<Self>
            where
                Self: Sized,
            {
                match syntax.kind() {
                    SyntaxKind::$kind => Some(Self(syntax)),
                    _ => None,
                }
            }

            fn syntax(&self) -> &SyntaxNode {
                &self.0
            }
        }
    };
}

prefix_expression!(LogicalNotExpr, LogicalNotExprNode, "logical `not`");
prefix_expression!(NegationExpr, NegationExprNode, "negation");
infix_expression!(LogicalOrExpr, LogicalOrExprNode, "logical `or`");
infix_expression!(LogicalAndExpr, LogicalAndExprNode, "logical `and`");
infix_expression!(EqualityExpr, EqualityExprNode, "equality");
infix_expression!(InequalityExpr, InequalityExprNode, "inequality");
infix_expression!(LessExpr, LessExprNode, "less than");
infix_expression!(LessEqualExpr, LessEqualExprNode, "less than or equal to");
infix_expression!(GreaterExpr, GreaterExprNode, "greater than");
infix_expression!(
    GreaterEqualExpr,
    GreaterEqualExprNode,
    "greater than or equal to"
);
infix_expression!(AdditionExpr, AdditionExprNode, "addition");
infix_expression!(SubtractionExpr, SubtractionExprNode, "substitution");
infix_expression!(MultiplicationExpr, MultiplicationExprNode, "multiplication");
infix_expression!(DivisionExpr, DivisionExprNode, "division");
infix_expression!(ModuloExpr, ModuloExprNode, "modulo");

/// Represents a call expression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallExpr(pub(super) SyntaxNode);

impl CallExpr {
    /// Gets the call target expression.
    pub fn target(&self) -> Expr {
        child(&self.0).expect("expected a target expression")
    }

    /// Gets the call arguments.
    pub fn arguments(&self) -> impl Iterator<Item = Expr> {
        children(&self.0).skip(1)
    }
}

impl AstNode for CallExpr {
    type Language = WorkflowDescriptionLanguage;

    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == SyntaxKind::CallExprNode
    }

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        match syntax.kind() {
            SyntaxKind::CallExprNode => Some(Self(syntax)),
            _ => None,
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

/// Represents an index expression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexExpr(pub(super) SyntaxNode);

impl IndexExpr {
    /// Gets the operand and the index expressions.
    ///
    /// The first is the operand expression.
    /// The second is the index expression.
    pub fn operands(&self) -> (Expr, Expr) {
        let mut children = self.0.children().filter_map(Expr::cast);
        let operand = children.next().expect("expected an operand expression");
        let index = children.next().expect("expected an index expression");
        (operand, index)
    }
}

impl AstNode for IndexExpr {
    type Language = WorkflowDescriptionLanguage;

    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == SyntaxKind::IndexExprNode
    }

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        match syntax.kind() {
            SyntaxKind::IndexExprNode => Some(Self(syntax)),
            _ => None,
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

/// Represents an access expression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccessExpr(pub(super) SyntaxNode);

impl AccessExpr {
    /// Gets the operand and the name of the access.
    ///
    /// The first is the operand expression.
    /// The second is the member name.
    pub fn operands(&self) -> (Expr, Ident) {
        let operand = child(&self.0).expect("expected an operand expression");
        let name = Ident::cast(self.0.last_token().expect("expected a last token"))
            .expect("expected an ident token");
        (operand, name)
    }
}

impl AstNode for AccessExpr {
    type Language = WorkflowDescriptionLanguage;

    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == SyntaxKind::AccessExprNode
    }

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        match syntax.kind() {
            SyntaxKind::AccessExprNode => Some(Self(syntax)),
            _ => None,
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::fmt::Write;

    use approx::assert_relative_eq;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::experimental::v1::VisitReason;
    use crate::experimental::v1::Visitor;
    use crate::experimental::Document;

    #[test]
    fn literal_booleans() {
        let parse = Document::parse(
            r#"
version 1.1

task test {
    Boolean a = true
    Boolean b = false
}
"#,
        );

        let document = parse.into_result().expect("there should be no errors");
        let ast = document.ast();
        let ast = ast.as_v1().expect("should be a V1 AST");
        let tasks: Vec<_> = ast.tasks().collect();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name().as_str(), "test");

        // Task declarations
        let decls: Vec<_> = tasks[0].declarations().collect();
        assert_eq!(decls.len(), 2);

        // First declaration
        assert_eq!(decls[0].ty().to_string(), "Boolean");
        assert_eq!(decls[0].name().as_str(), "a");
        assert!(decls[0].expr().unwrap_literal().unwrap_boolean().value());

        // Second declaration
        assert_eq!(decls[1].ty().to_string(), "Boolean");
        assert_eq!(decls[1].name().as_str(), "b");
        assert!(!decls[1].expr().unwrap_literal().unwrap_boolean().value());

        // Visit the literal boolean values in the tree
        struct MyVisitor(Vec<bool>);

        impl Visitor for MyVisitor {
            type State = ();

            fn expr(&mut self, _: &mut Self::State, reason: VisitReason, expr: &Expr) {
                if reason != VisitReason::Enter {
                    return;
                }

                if let Expr::Literal(LiteralExpr::Boolean(b)) = expr {
                    self.0.push(b.value());
                }
            }
        }

        let mut visitor = MyVisitor(Vec::new());
        ast.visit(&mut (), &mut visitor);
        assert_eq!(visitor.0, [true, false]);
    }

    #[test]
    fn literal_integer() {
        let parse = Document::parse(
            r#"
version 1.1

task test {
    Int a = 0
    Int b = 1234
    Int c = 01234
    Int d = 0x1234
    Int e = 0XF
    Int f = 9223372036854775807
    Int g = 9223372036854775808
    Int h = 9223372036854775809
}
"#,
        );

        let document = parse.into_result().expect("there should be no errors");
        let ast = document.ast();
        let ast = ast.as_v1().expect("should be a V1 AST");
        let tasks: Vec<_> = ast.tasks().collect();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name().as_str(), "test");

        // Task declarations
        let decls: Vec<_> = tasks[0].declarations().collect();
        assert_eq!(decls.len(), 8);

        // First declaration
        assert_eq!(decls[0].ty().to_string(), "Int");
        assert_eq!(decls[0].name().as_str(), "a");
        assert_eq!(
            decls[0]
                .expr()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            0
        );

        // Second declaration
        assert_eq!(decls[1].ty().to_string(), "Int");
        assert_eq!(decls[1].name().as_str(), "b");
        assert_eq!(
            decls[1]
                .expr()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            1234
        );

        // Third declaration
        assert_eq!(decls[2].ty().to_string(), "Int");
        assert_eq!(decls[2].name().as_str(), "c");
        assert_eq!(
            decls[2]
                .expr()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            668
        );

        // Fourth declaration
        assert_eq!(decls[3].ty().to_string(), "Int");
        assert_eq!(decls[3].name().as_str(), "d");
        assert_eq!(
            decls[3]
                .expr()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            4660
        );

        // Fifth declaration
        assert_eq!(decls[4].ty().to_string(), "Int");
        assert_eq!(decls[4].name().as_str(), "e");
        assert_eq!(
            decls[4]
                .expr()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            15
        );

        // Sixth declaration
        assert_eq!(decls[5].ty().to_string(), "Int");
        assert_eq!(decls[5].name().as_str(), "f");
        assert_eq!(
            decls[5]
                .expr()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            9223372036854775807
        );

        // Seventh declaration
        assert_eq!(decls[6].ty().to_string(), "Int");
        assert_eq!(decls[6].name().as_str(), "g");
        assert_eq!(
            decls[6]
                .expr()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            9223372036854775808
        );

        // Eighth declaration
        assert_eq!(decls[7].ty().to_string(), "Int");
        assert_eq!(decls[7].name().as_str(), "h");
        assert!(
            decls[7]
                .expr()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .is_none()
        );

        // Use a visitor to visit the literal integers in the tree
        struct MyVisitor(Vec<Option<u64>>);

        impl Visitor for MyVisitor {
            type State = ();

            fn expr(&mut self, _: &mut Self::State, reason: VisitReason, expr: &Expr) {
                if reason != VisitReason::Enter {
                    return;
                }

                if let Expr::Literal(LiteralExpr::Integer(i)) = expr {
                    self.0.push(i.value());
                }
            }
        }

        let mut visitor = MyVisitor(Vec::new());
        ast.visit(&mut (), &mut visitor);
        assert_eq!(
            visitor.0,
            [
                Some(0),
                Some(1234),
                Some(668),
                Some(4660),
                Some(15),
                Some(9223372036854775807),
                Some(9223372036854775808),
                None,
            ]
        );
    }

    #[test]
    fn literal_float() {
        let parse = Document::parse(
            r#"
version 1.1

task test {
    Float a = 0.
    Float b = 0.0
    Float c = 1234.1234
    Float d = 123e123
    Float e = 0.1234
    Float f = 10.
    Float g = .2
    Float h = 1234.1234e1234
}
"#,
        );

        let document = parse.into_result().expect("there should be no errors");
        let ast = document.ast();
        let ast = ast.as_v1().expect("should be a V1 AST");
        let tasks: Vec<_> = ast.tasks().collect();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name().as_str(), "test");

        // Task declarations
        let decls: Vec<_> = tasks[0].declarations().collect();
        assert_eq!(decls.len(), 8);

        // First declaration
        assert_eq!(decls[0].ty().to_string(), "Float");
        assert_eq!(decls[0].name().as_str(), "a");
        assert_relative_eq!(
            decls[0]
                .expr()
                .unwrap_literal()
                .unwrap_float()
                .value()
                .unwrap(),
            0.0
        );

        // Second declaration
        assert_eq!(decls[1].ty().to_string(), "Float");
        assert_eq!(decls[1].name().as_str(), "b");
        assert_relative_eq!(
            decls[1]
                .expr()
                .unwrap_literal()
                .unwrap_float()
                .value()
                .unwrap(),
            0.0
        );

        // Third declaration
        assert_eq!(decls[2].ty().to_string(), "Float");
        assert_eq!(decls[2].name().as_str(), "c");
        assert_relative_eq!(
            decls[2]
                .expr()
                .unwrap_literal()
                .unwrap_float()
                .value()
                .unwrap(),
            1234.1234
        );

        // Fourth declaration
        assert_eq!(decls[3].ty().to_string(), "Float");
        assert_eq!(decls[3].name().as_str(), "d");
        assert_relative_eq!(
            decls[3]
                .expr()
                .unwrap_literal()
                .unwrap_float()
                .value()
                .unwrap(),
            123e+123
        );

        // Fifth declaration
        assert_eq!(decls[4].ty().to_string(), "Float");
        assert_eq!(decls[4].name().as_str(), "e");
        assert_relative_eq!(
            decls[4]
                .expr()
                .unwrap_literal()
                .unwrap_float()
                .value()
                .unwrap(),
            0.1234
        );

        // Sixth declaration
        assert_eq!(decls[5].ty().to_string(), "Float");
        assert_eq!(decls[5].name().as_str(), "f");
        assert_relative_eq!(
            decls[5]
                .expr()
                .unwrap_literal()
                .unwrap_float()
                .value()
                .unwrap(),
            10.0
        );

        // Seventh declaration
        assert_eq!(decls[6].ty().to_string(), "Float");
        assert_eq!(decls[6].name().as_str(), "g");
        assert_relative_eq!(
            decls[6]
                .expr()
                .unwrap_literal()
                .unwrap_float()
                .value()
                .unwrap(),
            0.2
        );

        // Eighth declaration
        assert_eq!(decls[7].ty().to_string(), "Float");
        assert_eq!(decls[7].name().as_str(), "h");
        assert!(
            decls[7]
                .expr()
                .unwrap_literal()
                .unwrap_float()
                .value()
                .is_none()
        );

        // Use a visitor to visit all the literal floats in the tree
        struct MyVisitor(Vec<f64>);

        impl Visitor for MyVisitor {
            type State = ();

            fn expr(&mut self, _: &mut Self::State, reason: VisitReason, expr: &Expr) {
                if reason != VisitReason::Enter {
                    return;
                }

                if let Expr::Literal(LiteralExpr::Float(f)) = expr {
                    if let Some(f) = f.value() {
                        self.0.push(f);
                    }
                }
            }
        }

        let mut visitor = MyVisitor(Vec::new());
        ast.visit(&mut (), &mut visitor);
        assert_relative_eq!(
            visitor.0.as_slice(),
            [0.0, 0.0, 1234.1234, 123e123, 0.1234, 10.0, 0.2].as_slice()
        );
    }

    #[test]
    fn literal_string() {
        let parse = Document::parse(
            r#"
version 1.1

task test {
    String a = "hello"
    String b = 'world'
    String c = "Hello, ${name}!"
    String d = 'String~{'ception'}!'
}
"#,
        );

        let document = parse.into_result().expect("there should be no errors");
        let ast = document.ast();
        let ast = ast.as_v1().expect("should be a V1 AST");
        let tasks: Vec<_> = ast.tasks().collect();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name().as_str(), "test");

        // Task declarations
        let decls: Vec<_> = tasks[0].declarations().collect();
        assert_eq!(decls.len(), 4);

        // First declaration
        assert_eq!(decls[0].ty().to_string(), "String");
        assert_eq!(decls[0].name().as_str(), "a");
        let s = decls[0].expr().unwrap_literal().unwrap_string();
        assert_eq!(s.quote(), '"');
        assert_eq!(s.text().unwrap().as_str(), "hello");

        // Second declaration
        assert_eq!(decls[1].ty().to_string(), "String");
        assert_eq!(decls[1].name().as_str(), "b");
        let s = decls[1].expr().unwrap_literal().unwrap_string();
        assert_eq!(s.quote(), '\'');
        assert_eq!(s.text().unwrap().as_str(), "world");

        // Third declaration
        assert_eq!(decls[2].ty().to_string(), "String");
        assert_eq!(decls[2].name().as_str(), "c");
        let s = decls[2].expr().unwrap_literal().unwrap_string();
        assert_eq!(s.quote(), '"');
        let parts: Vec<_> = s.parts().collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0].clone().unwrap_text().as_str(), "Hello, ");
        assert_eq!(
            parts[1]
                .clone()
                .unwrap_placeholder()
                .expr()
                .unwrap_name_ref()
                .name()
                .as_str(),
            "name"
        );
        assert_eq!(parts[2].clone().unwrap_text().as_str(), "!");

        // Fourth declaration
        assert_eq!(decls[3].ty().to_string(), "String");
        assert_eq!(decls[3].name().as_str(), "d");
        let s = decls[3].expr().unwrap_literal().unwrap_string();
        assert_eq!(s.quote(), '\'');
        let parts: Vec<_> = s.parts().collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0].clone().unwrap_text().as_str(), "String");
        assert_eq!(
            parts[1]
                .clone()
                .unwrap_placeholder()
                .expr()
                .unwrap_literal()
                .unwrap_string()
                .text()
                .unwrap()
                .as_str(),
            "ception"
        );
        assert_eq!(parts[2].clone().unwrap_text().as_str(), "!");

        // Use a visitor to visit all the string literals without placeholders
        struct MyVisitor(Vec<String>);

        impl Visitor for MyVisitor {
            type State = ();

            fn expr(&mut self, _: &mut Self::State, reason: VisitReason, expr: &Expr) {
                if reason != VisitReason::Enter {
                    return;
                }

                // Collect only the non-interpolated strings in the source
                if let Expr::Literal(LiteralExpr::String(s)) = expr {
                    if let Some(s) = s.text() {
                        self.0.push(s.as_str().to_string());
                    }
                }
            }
        }

        let mut visitor = MyVisitor(Vec::new());
        ast.visit(&mut (), &mut visitor);
        assert_eq!(visitor.0, ["hello", "world", "ception"]);
    }

    #[test]
    fn literal_array() {
        let parse = Document::parse(
            r#"
version 1.1

task test {
    Array[Int] a = [1, 2, 3]
    Array[String] b = ["hello", "world", "!"]
    Array[Array[Int]] c = [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
}
"#,
        );

        let document = parse.into_result().expect("there should be no errors");
        let ast = document.ast();
        let ast = ast.as_v1().expect("should be a V1 AST");
        let tasks: Vec<_> = ast.tasks().collect();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name().as_str(), "test");

        // Task declarations
        let decls: Vec<_> = tasks[0].declarations().collect();
        assert_eq!(decls.len(), 3);

        // First declaration
        assert_eq!(decls[0].ty().to_string(), "Array[Int]");
        assert_eq!(decls[0].name().as_str(), "a");
        let a = decls[0].expr().unwrap_literal().unwrap_array();
        let elements: Vec<_> = a.elements().collect();
        assert_eq!(elements.len(), 3);
        assert_eq!(
            elements[0]
                .clone()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            1
        );
        assert_eq!(
            elements[1]
                .clone()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            2
        );
        assert_eq!(
            elements[2]
                .clone()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            3
        );

        // Second declaration
        assert_eq!(decls[1].ty().to_string(), "Array[String]");
        assert_eq!(decls[1].name().as_str(), "b");
        let a = decls[1].expr().unwrap_literal().unwrap_array();
        let elements: Vec<_> = a.elements().collect();
        assert_eq!(elements.len(), 3);
        assert_eq!(
            elements[0]
                .clone()
                .unwrap_literal()
                .unwrap_string()
                .text()
                .unwrap()
                .as_str(),
            "hello"
        );
        assert_eq!(
            elements[1]
                .clone()
                .unwrap_literal()
                .unwrap_string()
                .text()
                .unwrap()
                .as_str(),
            "world"
        );
        assert_eq!(
            elements[2]
                .clone()
                .unwrap_literal()
                .unwrap_string()
                .text()
                .unwrap()
                .as_str(),
            "!"
        );

        // Third declaration
        assert_eq!(decls[2].ty().to_string(), "Array[Array[Int]]");
        assert_eq!(decls[2].name().as_str(), "c");
        let a = decls[2].expr().unwrap_literal().unwrap_array();
        let elements: Vec<_> = a.elements().collect();
        assert_eq!(elements.len(), 3);
        let sub: Vec<_> = elements[0]
            .clone()
            .unwrap_literal()
            .unwrap_array()
            .elements()
            .collect();
        assert_eq!(sub.len(), 3);
        assert_eq!(
            sub[0]
                .clone()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            1
        );
        assert_eq!(
            sub[1]
                .clone()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            2
        );
        assert_eq!(
            sub[2]
                .clone()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            3
        );
        let sub: Vec<_> = elements[1]
            .clone()
            .unwrap_literal()
            .unwrap_array()
            .elements()
            .collect();
        assert_eq!(sub.len(), 3);
        assert_eq!(
            sub[0]
                .clone()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            4
        );
        assert_eq!(
            sub[1]
                .clone()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            5
        );
        assert_eq!(
            sub[2]
                .clone()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            6
        );
        let sub: Vec<_> = elements[2]
            .clone()
            .unwrap_literal()
            .unwrap_array()
            .elements()
            .collect();
        assert_eq!(sub.len(), 3);
        assert_eq!(
            sub[0]
                .clone()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            7
        );
        assert_eq!(
            sub[1]
                .clone()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            8
        );
        assert_eq!(
            sub[2]
                .clone()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            9
        );

        // Use a visitor to visit all the literal arrays in the tree,
        // flattening as needed
        struct MyVisitor(Vec<Vec<String>>);

        impl Visitor for MyVisitor {
            type State = ();

            fn expr(&mut self, _: &mut Self::State, reason: VisitReason, expr: &Expr) {
                if reason != VisitReason::Enter {
                    return;
                }

                if let Expr::Literal(LiteralExpr::Array(a)) = expr {
                    let mut elements = Vec::new();
                    for element in a.elements() {
                        match element {
                            Expr::Literal(LiteralExpr::Integer(i)) => {
                                elements.push(i.value().unwrap().to_string())
                            }
                            Expr::Literal(LiteralExpr::String(s)) => {
                                elements.push(s.text().unwrap().as_str().to_string())
                            }
                            Expr::Literal(LiteralExpr::Array(a)) => {
                                for element in a.elements().map(|e| {
                                    e.unwrap_literal()
                                        .unwrap_integer()
                                        .value()
                                        .unwrap()
                                        .to_string()
                                }) {
                                    elements.push(element);
                                }
                            }
                            _ => panic!("unexpected element"),
                        }
                    }

                    self.0.push(elements);
                }
            }
        }

        let mut visitor = MyVisitor(Vec::new());
        ast.visit(&mut (), &mut visitor);
        assert_eq!(visitor.0.len(), 6);
        assert_eq!(visitor.0[0], ["1", "2", "3"]);
        assert_eq!(visitor.0[1], ["hello", "world", "!"]);
        assert_eq!(visitor.0[2], ["1", "2", "3", "4", "5", "6", "7", "8", "9"]); // flattened
        assert_eq!(visitor.0[3], ["1", "2", "3"]); // first inner
        assert_eq!(visitor.0[4], ["4", "5", "6"]); // second inner
        assert_eq!(visitor.0[5], ["7", "8", "9"]); // third inner
    }

    #[test]
    fn literal_pair() {
        let parse = Document::parse(
            r#"
version 1.1

task test {
    Pair[Int, Int] a = (1000, 0x1000)
    Pair[String, Int] b = ("0x1000", 1000)
    Array[Pair[Int, String]] c = [(1, "hello"), (2, 'world'), (3, "!")]
}
"#,
        );

        let document = parse.into_result().expect("there should be no errors");
        let ast = document.ast();
        let ast = ast.as_v1().expect("should be a V1 AST");
        let tasks: Vec<_> = ast.tasks().collect();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name().as_str(), "test");

        // Task declarations
        let decls: Vec<_> = tasks[0].declarations().collect();
        assert_eq!(decls.len(), 3);

        // First declaration
        assert_eq!(decls[0].ty().to_string(), "Pair[Int, Int]");
        assert_eq!(decls[0].name().as_str(), "a");
        let p = decls[0].expr().unwrap_literal().unwrap_pair();
        let (first, second) = p.exprs();
        assert_eq!(
            first
                .clone()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            1000
        );
        assert_eq!(
            second
                .clone()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            0x1000
        );

        // Second declaration
        assert_eq!(decls[1].ty().to_string(), "Pair[String, Int]");
        assert_eq!(decls[1].name().as_str(), "b");
        let p = decls[1].expr().unwrap_literal().unwrap_pair();
        let (first, second) = p.exprs();
        assert_eq!(
            first
                .clone()
                .unwrap_literal()
                .unwrap_string()
                .text()
                .unwrap()
                .as_str(),
            "0x1000"
        );
        assert_eq!(
            second
                .clone()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            1000
        );

        // Third declaration
        assert_eq!(decls[2].ty().to_string(), "Array[Pair[Int, String]]");
        assert_eq!(decls[2].name().as_str(), "c");
        let a = decls[2].expr().unwrap_literal().unwrap_array();
        let elements: Vec<_> = a.elements().collect();
        assert_eq!(elements.len(), 3);
        let p = elements[0].clone().unwrap_literal().unwrap_pair();
        let (first, second) = p.exprs();
        assert_eq!(
            first
                .clone()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            1
        );
        assert_eq!(
            second
                .clone()
                .unwrap_literal()
                .unwrap_string()
                .text()
                .unwrap()
                .as_str(),
            "hello"
        );
        let p = elements[1].clone().unwrap_literal().unwrap_pair();
        let (first, second) = p.exprs();
        assert_eq!(
            first
                .clone()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            2
        );
        assert_eq!(
            second
                .clone()
                .unwrap_literal()
                .unwrap_string()
                .text()
                .unwrap()
                .as_str(),
            "world"
        );
        let p = elements[2].clone().unwrap_literal().unwrap_pair();
        let (first, second) = p.exprs();
        assert_eq!(
            first
                .clone()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            3
        );
        assert_eq!(
            second
                .clone()
                .unwrap_literal()
                .unwrap_string()
                .text()
                .unwrap()
                .as_str(),
            "!"
        );

        // Use a visitor to visit all the literal pairs in the tree
        struct MyVisitor(Vec<(String, String)>);

        impl Visitor for MyVisitor {
            type State = ();

            fn expr(&mut self, _: &mut Self::State, reason: VisitReason, expr: &Expr) {
                if reason != VisitReason::Enter {
                    return;
                }

                if let Expr::Literal(LiteralExpr::Pair(p)) = expr {
                    let (first, second) = p.exprs();

                    let first = match first {
                        Expr::Literal(LiteralExpr::String(s)) => {
                            s.text().unwrap().as_str().to_string()
                        }
                        Expr::Literal(LiteralExpr::Integer(i)) => i.value().unwrap().to_string(),
                        _ => panic!("expected a string or integer"),
                    };

                    let second = match second {
                        Expr::Literal(LiteralExpr::String(s)) => {
                            s.text().unwrap().as_str().to_string()
                        }
                        Expr::Literal(LiteralExpr::Integer(i)) => i.value().unwrap().to_string(),
                        _ => panic!("expected a string or integer"),
                    };

                    self.0.push((first, second));
                }
            }
        }

        let mut visitor = MyVisitor(Vec::new());
        ast.visit(&mut (), &mut visitor);
        assert_eq!(
            visitor
                .0
                .iter()
                .map(|(f, s)| (f.as_str(), s.as_str()))
                .collect::<Vec<_>>(),
            [
                ("1000", "4096"),
                ("0x1000", "1000"),
                ("1", "hello"),
                ("2", "world"),
                ("3", "!")
            ]
        );
    }

    #[test]
    fn literal_map() {
        let parse = Document::parse(
            r#"
version 1.1

task test {
    Map[Int, Int] a = {}
    Map[String, String] b = { "foo": "bar", "bar": "baz" }
}
"#,
        );

        let document = parse.into_result().expect("there should be no errors");
        let ast = document.ast();
        let ast = ast.as_v1().expect("should be a V1 AST");
        let tasks: Vec<_> = ast.tasks().collect();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name().as_str(), "test");

        // Task declarations
        let decls: Vec<_> = tasks[0].declarations().collect();
        assert_eq!(decls.len(), 2);

        // First declaration
        assert_eq!(decls[0].ty().to_string(), "Map[Int, Int]");
        assert_eq!(decls[0].name().as_str(), "a");
        let m = decls[0].expr().unwrap_literal().unwrap_map();
        let items: Vec<_> = m.items().collect();
        assert_eq!(items.len(), 0);

        // Second declaration
        assert_eq!(decls[1].ty().to_string(), "Map[String, String]");
        assert_eq!(decls[1].name().as_str(), "b");
        let m = decls[1].expr().unwrap_literal().unwrap_map();
        let items: Vec<_> = m.items().collect();
        assert_eq!(items.len(), 2);
        let (key, value) = items[0].key_value();
        assert_eq!(
            key.unwrap_literal()
                .unwrap_string()
                .text()
                .unwrap()
                .as_str(),
            "foo"
        );
        assert_eq!(
            value
                .unwrap_literal()
                .unwrap_string()
                .text()
                .unwrap()
                .as_str(),
            "bar"
        );
        let (key, value) = items[1].key_value();
        assert_eq!(
            key.unwrap_literal()
                .unwrap_string()
                .text()
                .unwrap()
                .as_str(),
            "bar"
        );
        assert_eq!(
            value
                .unwrap_literal()
                .unwrap_string()
                .text()
                .unwrap()
                .as_str(),
            "baz"
        );

        // Use a visitor to visit every literal map in the tree
        struct MyVisitor(Vec<HashMap<String, String>>);

        impl Visitor for MyVisitor {
            type State = ();

            fn expr(&mut self, _: &mut Self::State, reason: VisitReason, expr: &Expr) {
                if reason != VisitReason::Enter {
                    return;
                }

                if let Expr::Literal(LiteralExpr::Map(m)) = expr {
                    let mut items = HashMap::new();
                    for item in m.items() {
                        let (key, value) = item.key_value();
                        items.insert(
                            key.unwrap_literal()
                                .unwrap_string()
                                .text()
                                .unwrap()
                                .as_str()
                                .to_string(),
                            value
                                .unwrap_literal()
                                .unwrap_string()
                                .text()
                                .unwrap()
                                .as_str()
                                .to_string(),
                        );
                    }

                    self.0.push(items);
                }
            }
        }

        let mut visitor = MyVisitor(Vec::new());
        ast.visit(&mut (), &mut visitor);
        assert_eq!(visitor.0.len(), 2);
        assert_eq!(visitor.0[0].len(), 0);
        assert_eq!(visitor.0[1].len(), 2);
        assert_eq!(visitor.0[1]["foo"], "bar");
        assert_eq!(visitor.0[1]["bar"], "baz");
    }

    #[test]
    fn literal_object() {
        let parse = Document::parse(
            r#"
version 1.1

task test {
    Object a = object {}
    Object b = object { foo: "bar", bar: 1, baz: [1, 2, 3] }
}
"#,
        );

        let document = parse.into_result().expect("there should be no errors");
        let ast = document.ast();
        let ast = ast.as_v1().expect("should be a V1 AST");
        let tasks: Vec<_> = ast.tasks().collect();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name().as_str(), "test");

        // Task declarations
        let decls: Vec<_> = tasks[0].declarations().collect();
        assert_eq!(decls.len(), 2);

        // First declaration
        assert_eq!(decls[0].ty().to_string(), "Object");
        assert_eq!(decls[0].name().as_str(), "a");
        let o = decls[0].expr().unwrap_literal().unwrap_object();
        let items: Vec<_> = o.items().collect();
        assert_eq!(items.len(), 0);

        // Second declaration
        assert_eq!(decls[1].ty().to_string(), "Object");
        assert_eq!(decls[1].name().as_str(), "b");
        let o = decls[1].expr().unwrap_literal().unwrap_object();
        let items: Vec<_> = o.items().collect();
        assert_eq!(items.len(), 3);
        let (name, value) = items[0].name_value();
        assert_eq!(name.as_str(), "foo");
        assert_eq!(
            value
                .unwrap_literal()
                .unwrap_string()
                .text()
                .unwrap()
                .as_str(),
            "bar"
        );
        let (name, value) = items[1].name_value();
        assert_eq!(name.as_str(), "bar");
        assert_eq!(value.unwrap_literal().unwrap_integer().value().unwrap(), 1);
        let (name, value) = items[2].name_value();
        assert_eq!(name.as_str(), "baz");
        let elements: Vec<_> = value.unwrap_literal().unwrap_array().elements().collect();
        assert_eq!(elements.len(), 3);
        assert_eq!(
            elements[0]
                .clone()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            1
        );
        assert_eq!(
            elements[1]
                .clone()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            2
        );
        assert_eq!(
            elements[2]
                .clone()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            3
        );

        // Use a visitor to visit every literal object in the tree
        struct MyVisitor(Vec<HashMap<String, String>>);

        impl Visitor for MyVisitor {
            type State = ();

            fn expr(&mut self, _: &mut Self::State, reason: VisitReason, expr: &Expr) {
                if reason != VisitReason::Enter {
                    return;
                }

                if let Expr::Literal(LiteralExpr::Object(o)) = expr {
                    let mut items = HashMap::new();
                    for item in o.items() {
                        let (name, value) = item.name_value();
                        match value {
                            Expr::Literal(LiteralExpr::Integer(i)) => {
                                items.insert(
                                    name.as_str().to_string(),
                                    i.value().unwrap().to_string(),
                                );
                            }
                            Expr::Literal(LiteralExpr::String(s)) => {
                                items.insert(
                                    name.as_str().to_string(),
                                    s.text().unwrap().as_str().to_string(),
                                );
                            }
                            Expr::Literal(LiteralExpr::Array(a)) => {
                                items.insert(
                                    name.as_str().to_string(),
                                    a.elements()
                                        .map(|e| {
                                            e.unwrap_literal().unwrap_integer().value().unwrap()
                                        })
                                        .fold(String::new(), |mut v, i| {
                                            if !v.is_empty() {
                                                v.push_str(", ");
                                            }
                                            write!(&mut v, "{i}").unwrap();
                                            v
                                        }),
                                );
                            }
                            _ => panic!("unexpected element"),
                        }
                    }

                    self.0.push(items);
                }
            }
        }

        let mut visitor = MyVisitor(Vec::new());
        ast.visit(&mut (), &mut visitor);
        assert_eq!(visitor.0.len(), 2);
        assert_eq!(visitor.0[0].len(), 0);
        assert_eq!(visitor.0[1].len(), 3);
        assert_eq!(visitor.0[1]["foo"], "bar");
        assert_eq!(visitor.0[1]["bar"], "1");
        assert_eq!(visitor.0[1]["baz"], "1, 2, 3");
    }

    #[test]
    fn literal_struct() {
        let parse = Document::parse(
            r#"
version 1.1

task test {
    Foo a = Foo { foo: "bar" }
    Bar b = Bar { bar: 1, baz: [1, 2, 3] }
}
"#,
        );

        let document = parse.into_result().expect("there should be no errors");
        let ast = document.ast();
        let ast = ast.as_v1().expect("should be a V1 AST");
        let tasks: Vec<_> = ast.tasks().collect();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name().as_str(), "test");

        // Task declarations
        let decls: Vec<_> = tasks[0].declarations().collect();
        assert_eq!(decls.len(), 2);

        // First declaration
        assert_eq!(decls[0].ty().to_string(), "Foo");
        assert_eq!(decls[0].name().as_str(), "a");
        let s = decls[0].expr().unwrap_literal().unwrap_struct();
        assert_eq!(s.name().as_str(), "Foo");
        let items: Vec<_> = s.items().collect();
        assert_eq!(items.len(), 1);
        let (name, value) = items[0].name_value();
        assert_eq!(name.as_str(), "foo");
        assert_eq!(
            value
                .unwrap_literal()
                .unwrap_string()
                .text()
                .unwrap()
                .as_str(),
            "bar"
        );

        // Second declaration
        assert_eq!(decls[1].ty().to_string(), "Bar");
        assert_eq!(decls[1].name().as_str(), "b");
        let s = decls[1].expr().unwrap_literal().unwrap_struct();
        assert_eq!(s.name().as_str(), "Bar");
        let items: Vec<_> = s.items().collect();
        assert_eq!(items.len(), 2);
        let (name, value) = items[0].name_value();
        assert_eq!(name.as_str(), "bar");
        assert_eq!(value.unwrap_literal().unwrap_integer().value().unwrap(), 1);
        let (name, value) = items[1].name_value();
        assert_eq!(name.as_str(), "baz");
        let elements: Vec<_> = value.unwrap_literal().unwrap_array().elements().collect();
        assert_eq!(elements.len(), 3);
        assert_eq!(
            elements[0]
                .clone()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            1
        );
        assert_eq!(
            elements[1]
                .clone()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            2
        );
        assert_eq!(
            elements[2]
                .clone()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            3
        );

        // Use a visitor to visit every literal struct in the tree
        struct MyVisitor(Vec<HashMap<String, String>>);

        impl Visitor for MyVisitor {
            type State = ();

            fn expr(&mut self, _: &mut Self::State, reason: VisitReason, expr: &Expr) {
                if reason != VisitReason::Enter {
                    return;
                }

                if let Expr::Literal(LiteralExpr::Struct(s)) = expr {
                    let mut items = HashMap::new();
                    for item in s.items() {
                        let (name, value) = item.name_value();
                        match value {
                            Expr::Literal(LiteralExpr::Integer(i)) => {
                                items.insert(
                                    name.as_str().to_string(),
                                    i.value().unwrap().to_string(),
                                );
                            }
                            Expr::Literal(LiteralExpr::String(s)) => {
                                items.insert(
                                    name.as_str().to_string(),
                                    s.text().unwrap().as_str().to_string(),
                                );
                            }
                            Expr::Literal(LiteralExpr::Array(a)) => {
                                items.insert(
                                    name.as_str().to_string(),
                                    a.elements()
                                        .map(|e| {
                                            e.unwrap_literal().unwrap_integer().value().unwrap()
                                        })
                                        .fold(String::new(), |mut v, i| {
                                            if !v.is_empty() {
                                                v.push_str(", ");
                                            }
                                            write!(&mut v, "{i}").unwrap();
                                            v
                                        }),
                                );
                            }
                            _ => panic!("unexpected element"),
                        }
                    }

                    self.0.push(items);
                }
            }
        }

        let mut visitor = MyVisitor(Vec::new());
        ast.visit(&mut (), &mut visitor);
        assert_eq!(visitor.0.len(), 2);
        assert_eq!(visitor.0[0].len(), 1);
        assert_eq!(visitor.0[0]["foo"], "bar");
        assert_eq!(visitor.0[1].len(), 2);
        assert_eq!(visitor.0[1]["bar"], "1");
        assert_eq!(visitor.0[1]["baz"], "1, 2, 3");
    }

    #[test]
    fn literal_none() {
        let parse = Document::parse(
            r#"
version 1.1

task test {
    Int? a = None
    Boolean b = a == None
}
"#,
        );

        let document = parse.into_result().expect("there should be no errors");
        let ast = document.ast();
        let ast = ast.as_v1().expect("should be a V1 AST");
        let tasks: Vec<_> = ast.tasks().collect();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name().as_str(), "test");

        // Task declarations
        let decls: Vec<_> = tasks[0].declarations().collect();
        assert_eq!(decls.len(), 2);

        // First declaration
        assert_eq!(decls[0].ty().to_string(), "Int?");
        assert_eq!(decls[0].name().as_str(), "a");
        decls[0].expr().unwrap_literal().unwrap_none();

        // Second declaration
        assert_eq!(decls[1].ty().to_string(), "Boolean");
        assert_eq!(decls[1].name().as_str(), "b");
        let (lhs, rhs) = decls[1].expr().unwrap_equality().operands();
        assert_eq!(lhs.unwrap_name_ref().name().as_str(), "a");
        rhs.unwrap_literal().unwrap_none();

        // Use a visitor to count the number of literal `None` in the tree
        struct MyVisitor(usize);

        impl Visitor for MyVisitor {
            type State = ();

            fn expr(&mut self, _: &mut Self::State, reason: VisitReason, expr: &Expr) {
                if reason == VisitReason::Enter {
                    if let Expr::Literal(LiteralExpr::None(_)) = expr {
                        self.0 += 1;
                    }
                }
            }
        }

        let mut visitor = MyVisitor(0);
        ast.visit(&mut (), &mut visitor);
        assert_eq!(visitor.0, 2);
    }

    #[test]
    fn name_ref() {
        let parse = Document::parse(
            r#"
version 1.1

task test {
    Int a = 0
    Int b = a
}
"#,
        );

        let document = parse.into_result().expect("there should be no errors");
        let ast = document.ast();
        let ast = ast.as_v1().expect("should be a V1 AST");
        let tasks: Vec<_> = ast.tasks().collect();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name().as_str(), "test");

        // Task declarations
        let decls: Vec<_> = tasks[0].declarations().collect();
        assert_eq!(decls.len(), 2);

        // First declaration
        assert_eq!(decls[0].ty().to_string(), "Int");
        assert_eq!(decls[0].name().as_str(), "a");
        assert_eq!(
            decls[0]
                .expr()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            0
        );

        // Second declaration
        assert_eq!(decls[1].ty().to_string(), "Int");
        assert_eq!(decls[1].name().as_str(), "b");
        assert_eq!(decls[1].expr().unwrap_name_ref().name().as_str(), "a");

        // Use a visitor to visit every name reference in the tree
        struct MyVisitor(Vec<String>);

        impl Visitor for MyVisitor {
            type State = ();

            fn expr(&mut self, _: &mut Self::State, reason: VisitReason, expr: &Expr) {
                if reason != VisitReason::Enter {
                    return;
                }

                if let Expr::Name(n) = expr {
                    self.0.push(n.name().as_str().to_string());
                }
            }
        }

        let mut visitor = MyVisitor(Vec::new());
        ast.visit(&mut (), &mut visitor);
        assert_eq!(visitor.0, ["a"]);
    }

    #[test]
    fn parenthesized() {
        let parse = Document::parse(
            r#"
version 1.1

task test {
    Int a = (0)
    Int b = (10 - (5 + 5))
}
"#,
        );

        let document = parse.into_result().expect("there should be no errors");
        let ast = document.ast();
        let ast = ast.as_v1().expect("should be a V1 AST");
        let tasks: Vec<_> = ast.tasks().collect();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name().as_str(), "test");

        // Task declarations
        let decls: Vec<_> = tasks[0].declarations().collect();
        assert_eq!(decls.len(), 2);

        // First declaration
        assert_eq!(decls[0].ty().to_string(), "Int");
        assert_eq!(decls[0].name().as_str(), "a");
        assert_eq!(
            decls[0]
                .expr()
                .unwrap_parenthesized()
                .inner()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            0
        );

        // Second declaration
        assert_eq!(decls[1].ty().to_string(), "Int");
        assert_eq!(decls[1].name().as_str(), "b");
        let (lhs, rhs) = decls[1]
            .expr()
            .unwrap_parenthesized()
            .inner()
            .unwrap_subtraction()
            .operands();
        assert_eq!(lhs.unwrap_literal().unwrap_integer().value().unwrap(), 10);
        let (lhs, rhs) = rhs
            .unwrap_parenthesized()
            .inner()
            .unwrap_addition()
            .operands();
        assert_eq!(lhs.unwrap_literal().unwrap_integer().value().unwrap(), 5);
        assert_eq!(rhs.unwrap_literal().unwrap_integer().value().unwrap(), 5);

        // Use a visitor to count the number of parenthesized expressions in the tree
        struct MyVisitor(usize);

        impl Visitor for MyVisitor {
            type State = ();

            fn expr(&mut self, _: &mut Self::State, reason: VisitReason, expr: &Expr) {
                if reason == VisitReason::Enter {
                    if let Expr::Parenthesized(_) = expr {
                        self.0 += 1;
                    }
                }
            }
        }

        let mut visitor = MyVisitor(0);
        ast.visit(&mut (), &mut visitor);
        assert_eq!(visitor.0, 3);
    }

    #[test]
    fn if_expr() {
        let parse = Document::parse(
            r#"
version 1.1

task test {
    Int a = if true then 1 else 0
    String b = if a > 0 then "yes" else "no"
}
"#,
        );

        let document = parse.into_result().expect("there should be no errors");
        let ast = document.ast();
        let ast = ast.as_v1().expect("should be a V1 AST");
        let tasks: Vec<_> = ast.tasks().collect();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name().as_str(), "test");

        // Task declarations
        let decls: Vec<_> = tasks[0].declarations().collect();
        assert_eq!(decls.len(), 2);

        // First declaration
        assert_eq!(decls[0].ty().to_string(), "Int");
        assert_eq!(decls[0].name().as_str(), "a");
        let (c, t, f) = decls[0].expr().unwrap_if().exprs();
        assert!(c.unwrap_literal().unwrap_boolean().value());
        assert_eq!(t.unwrap_literal().unwrap_integer().value().unwrap(), 1);
        assert_eq!(f.unwrap_literal().unwrap_integer().value().unwrap(), 0);

        // Second declaration
        assert_eq!(decls[1].ty().to_string(), "String");
        assert_eq!(decls[1].name().as_str(), "b");
        let (c, t, f) = decls[1].expr().unwrap_if().exprs();
        let (lhs, rhs) = c.unwrap_greater().operands();
        assert_eq!(lhs.unwrap_name_ref().name().as_str(), "a");
        assert_eq!(rhs.unwrap_literal().unwrap_integer().value().unwrap(), 0);
        assert_eq!(
            t.unwrap_literal().unwrap_string().text().unwrap().as_str(),
            "yes"
        );
        assert_eq!(
            f.unwrap_literal().unwrap_string().text().unwrap().as_str(),
            "no"
        );

        // Use a visitor to count the number of `if` expressions in the tree
        struct MyVisitor(usize);

        impl Visitor for MyVisitor {
            type State = ();

            fn expr(&mut self, _: &mut Self::State, reason: VisitReason, expr: &Expr) {
                if reason == VisitReason::Enter {
                    if let Expr::If(_) = expr {
                        self.0 += 1;
                    }
                }
            }
        }

        let mut visitor = MyVisitor(0);
        ast.visit(&mut (), &mut visitor);
        assert_eq!(visitor.0, 2);
    }

    #[test]
    fn logical_not() {
        let parse = Document::parse(
            r#"
version 1.1

task test {
    Boolean a = !true
    Boolean b = !!!a
}
"#,
        );

        let document = parse.into_result().expect("there should be no errors");
        let ast = document.ast();
        let ast = ast.as_v1().expect("should be a V1 AST");
        let tasks: Vec<_> = ast.tasks().collect();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name().as_str(), "test");

        // Task declarations
        let decls: Vec<_> = tasks[0].declarations().collect();
        assert_eq!(decls.len(), 2);

        // First declaration
        assert_eq!(decls[0].ty().to_string(), "Boolean");
        assert_eq!(decls[0].name().as_str(), "a");
        assert!(
            decls[0]
                .expr()
                .unwrap_logical_not()
                .operand()
                .unwrap_literal()
                .unwrap_boolean()
                .value()
        );

        // Second declaration
        assert_eq!(decls[1].ty().to_string(), "Boolean");
        assert_eq!(decls[1].name().as_str(), "b");
        assert_eq!(
            decls[1]
                .expr()
                .unwrap_logical_not()
                .operand()
                .unwrap_logical_not()
                .operand()
                .unwrap_logical_not()
                .operand()
                .unwrap_name_ref()
                .name()
                .as_str(),
            "a"
        );

        // Use a visitor to count the number of logical not expressions in the tree
        struct MyVisitor(usize);

        impl Visitor for MyVisitor {
            type State = ();

            fn expr(&mut self, _: &mut Self::State, reason: VisitReason, expr: &Expr) {
                if reason == VisitReason::Enter {
                    if let Expr::LogicalNot(_) = expr {
                        self.0 += 1;
                    }
                }
            }
        }

        let mut visitor = MyVisitor(0);
        ast.visit(&mut (), &mut visitor);
        assert_eq!(visitor.0, 4);
    }

    #[test]
    fn negation() {
        let parse = Document::parse(
            r#"
version 1.1

task test {
    Int a = -1
    Int b = ---a
}
"#,
        );

        let document = parse.into_result().expect("there should be no errors");
        let ast = document.ast();
        let ast = ast.as_v1().expect("should be a V1 AST");
        let tasks: Vec<_> = ast.tasks().collect();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name().as_str(), "test");

        // Task declarations
        let decls: Vec<_> = tasks[0].declarations().collect();
        assert_eq!(decls.len(), 2);

        // First declaration
        assert_eq!(decls[0].ty().to_string(), "Int");
        assert_eq!(decls[0].name().as_str(), "a");
        assert_eq!(
            decls[0]
                .expr()
                .unwrap_negation()
                .operand()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            1
        );

        // Second declaration
        assert_eq!(decls[1].ty().to_string(), "Int");
        assert_eq!(decls[1].name().as_str(), "b");
        assert_eq!(
            decls[1]
                .expr()
                .unwrap_negation()
                .operand()
                .unwrap_negation()
                .operand()
                .unwrap_negation()
                .operand()
                .unwrap_name_ref()
                .name()
                .as_str(),
            "a"
        );

        // Use a visitor to count the number of negation expressions in the tree
        struct MyVisitor(usize);

        impl Visitor for MyVisitor {
            type State = ();

            fn expr(&mut self, _: &mut Self::State, reason: VisitReason, expr: &Expr) {
                if reason == VisitReason::Enter {
                    if let Expr::Negation(_) = expr {
                        self.0 += 1;
                    }
                }
            }
        }

        let mut visitor = MyVisitor(0);
        ast.visit(&mut (), &mut visitor);
        assert_eq!(visitor.0, 4);
    }

    #[test]
    fn logical_or() {
        let parse = Document::parse(
            r#"
version 1.1

task test {
    Boolean a = false
    Boolean b = true
    Boolean c = a || b
}
"#,
        );

        let document = parse.into_result().expect("there should be no errors");
        let ast = document.ast();
        let ast = ast.as_v1().expect("should be a V1 AST");
        let tasks: Vec<_> = ast.tasks().collect();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name().as_str(), "test");

        // Task declarations
        let decls: Vec<_> = tasks[0].declarations().collect();
        assert_eq!(decls.len(), 3);

        // First declaration
        assert_eq!(decls[0].ty().to_string(), "Boolean");
        assert_eq!(decls[0].name().as_str(), "a");
        assert!(!decls[0].expr().unwrap_literal().unwrap_boolean().value());

        // Second declaration
        assert_eq!(decls[1].ty().to_string(), "Boolean");
        assert_eq!(decls[1].name().as_str(), "b");
        assert!(decls[1].expr().unwrap_literal().unwrap_boolean().value());

        // Third declaration
        assert_eq!(decls[2].ty().to_string(), "Boolean");
        assert_eq!(decls[2].name().as_str(), "c");
        let (lhs, rhs) = decls[2].expr().unwrap_logical_or().operands();
        assert_eq!(lhs.unwrap_name_ref().name().as_str(), "a");
        assert_eq!(rhs.unwrap_name_ref().name().as_str(), "b");

        // Use a visitor to count the number of logical `or` expressions in the tree
        struct MyVisitor(usize);

        impl Visitor for MyVisitor {
            type State = ();

            fn expr(&mut self, _: &mut Self::State, reason: VisitReason, expr: &Expr) {
                if reason == VisitReason::Enter {
                    if let Expr::LogicalOr(_) = expr {
                        self.0 += 1;
                    }
                }
            }
        }

        let mut visitor = MyVisitor(0);
        ast.visit(&mut (), &mut visitor);
        assert_eq!(visitor.0, 1);
    }

    #[test]
    fn logical_and() {
        let parse = Document::parse(
            r#"
version 1.1

task test {
    Boolean a = true
    Boolean b = true
    Boolean c = a && b
}
"#,
        );

        let document = parse.into_result().expect("there should be no errors");
        let ast = document.ast();
        let ast = ast.as_v1().expect("should be a V1 AST");
        let tasks: Vec<_> = ast.tasks().collect();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name().as_str(), "test");

        // Task declarations
        let decls: Vec<_> = tasks[0].declarations().collect();
        assert_eq!(decls.len(), 3);

        // First declaration
        assert_eq!(decls[0].ty().to_string(), "Boolean");
        assert_eq!(decls[0].name().as_str(), "a");
        assert!(decls[0].expr().unwrap_literal().unwrap_boolean().value());

        // Second declaration
        assert_eq!(decls[1].ty().to_string(), "Boolean");
        assert_eq!(decls[1].name().as_str(), "b");
        assert!(decls[1].expr().unwrap_literal().unwrap_boolean().value());

        // Third declaration
        assert_eq!(decls[2].ty().to_string(), "Boolean");
        assert_eq!(decls[2].name().as_str(), "c");
        let (lhs, rhs) = decls[2].expr().unwrap_logical_and().operands();
        assert_eq!(lhs.unwrap_name_ref().name().as_str(), "a");
        assert_eq!(rhs.unwrap_name_ref().name().as_str(), "b");

        // Use a visitor to count the number of logical `and` expressions in the tree
        struct MyVisitor(usize);

        impl Visitor for MyVisitor {
            type State = ();

            fn expr(&mut self, _: &mut Self::State, reason: VisitReason, expr: &Expr) {
                if reason == VisitReason::Enter {
                    if let Expr::LogicalAnd(_) = expr {
                        self.0 += 1;
                    }
                }
            }
        }

        let mut visitor = MyVisitor(0);
        ast.visit(&mut (), &mut visitor);
        assert_eq!(visitor.0, 1);
    }

    #[test]
    fn equality() {
        let parse = Document::parse(
            r#"
version 1.1

task test {
    Boolean a = true
    Boolean b = false
    Boolean c = a == b
}
"#,
        );

        let document = parse.into_result().expect("there should be no errors");
        let ast = document.ast();
        let ast = ast.as_v1().expect("should be a V1 AST");
        let tasks: Vec<_> = ast.tasks().collect();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name().as_str(), "test");

        // Task declarations
        let decls: Vec<_> = tasks[0].declarations().collect();
        assert_eq!(decls.len(), 3);

        // First declaration
        assert_eq!(decls[0].ty().to_string(), "Boolean");
        assert_eq!(decls[0].name().as_str(), "a");
        assert!(decls[0].expr().unwrap_literal().unwrap_boolean().value());

        // Second declaration
        assert_eq!(decls[1].ty().to_string(), "Boolean");
        assert_eq!(decls[1].name().as_str(), "b");
        assert!(!decls[1].expr().unwrap_literal().unwrap_boolean().value());

        // Third declaration
        assert_eq!(decls[2].ty().to_string(), "Boolean");
        assert_eq!(decls[2].name().as_str(), "c");
        let (lhs, rhs) = decls[2].expr().unwrap_equality().operands();
        assert_eq!(lhs.unwrap_name_ref().name().as_str(), "a");
        assert_eq!(rhs.unwrap_name_ref().name().as_str(), "b");

        // Use a visitor to count the number of equality expressions in the tree
        struct MyVisitor(usize);

        impl Visitor for MyVisitor {
            type State = ();

            fn expr(&mut self, _: &mut Self::State, reason: VisitReason, expr: &Expr) {
                if reason == VisitReason::Enter {
                    if let Expr::Equality(_) = expr {
                        self.0 += 1;
                    }
                }
            }
        }

        let mut visitor = MyVisitor(0);
        ast.visit(&mut (), &mut visitor);
        assert_eq!(visitor.0, 1);
    }

    #[test]
    fn inequality() {
        let parse = Document::parse(
            r#"
version 1.1

task test {
    Boolean a = true
    Boolean b = false
    Boolean c = a != b
}
"#,
        );

        let document = parse.into_result().expect("there should be no errors");
        let ast = document.ast();
        let ast = ast.as_v1().expect("should be a V1 AST");
        let tasks: Vec<_> = ast.tasks().collect();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name().as_str(), "test");

        // Task declarations
        let decls: Vec<_> = tasks[0].declarations().collect();
        assert_eq!(decls.len(), 3);

        // First declaration
        assert_eq!(decls[0].ty().to_string(), "Boolean");
        assert_eq!(decls[0].name().as_str(), "a");
        assert!(decls[0].expr().unwrap_literal().unwrap_boolean().value());

        // Second declaration
        assert_eq!(decls[1].ty().to_string(), "Boolean");
        assert_eq!(decls[1].name().as_str(), "b");
        assert!(!decls[1].expr().unwrap_literal().unwrap_boolean().value());

        // Third declaration
        assert_eq!(decls[2].ty().to_string(), "Boolean");
        assert_eq!(decls[2].name().as_str(), "c");
        let (lhs, rhs) = decls[2].expr().unwrap_inequality().operands();
        assert_eq!(lhs.unwrap_name_ref().name().as_str(), "a");
        assert_eq!(rhs.unwrap_name_ref().name().as_str(), "b");

        // Use a visitor to count the number of inequality expressions in the tree.
        struct MyVisitor(usize);

        impl Visitor for MyVisitor {
            type State = ();

            fn expr(&mut self, _: &mut Self::State, reason: VisitReason, expr: &Expr) {
                if reason == VisitReason::Enter {
                    if let Expr::Inequality(_) = expr {
                        self.0 += 1;
                    }
                }
            }
        }

        let mut visitor = MyVisitor(0);
        ast.visit(&mut (), &mut visitor);
        assert_eq!(visitor.0, 1);
    }

    #[test]
    fn less() {
        let parse = Document::parse(
            r#"
version 1.1

task test {
    Int a = 1
    Int b = 2
    Boolean c = a < b
}
"#,
        );

        let document = parse.into_result().expect("there should be no errors");
        let ast = document.ast();
        let ast = ast.as_v1().expect("should be a V1 AST");
        let tasks: Vec<_> = ast.tasks().collect();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name().as_str(), "test");

        // Task declarations
        let decls: Vec<_> = tasks[0].declarations().collect();
        assert_eq!(decls.len(), 3);

        // First declaration
        assert_eq!(decls[0].ty().to_string(), "Int");
        assert_eq!(decls[0].name().as_str(), "a");
        assert_eq!(
            decls[0]
                .expr()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            1
        );

        // Second declaration
        assert_eq!(decls[1].ty().to_string(), "Int");
        assert_eq!(decls[1].name().as_str(), "b");
        assert_eq!(
            decls[1]
                .expr()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            2
        );

        // Third declaration
        assert_eq!(decls[2].ty().to_string(), "Boolean");
        assert_eq!(decls[2].name().as_str(), "c");
        let (lhs, rhs) = decls[2].expr().unwrap_less().operands();
        assert_eq!(lhs.unwrap_name_ref().name().as_str(), "a");
        assert_eq!(rhs.unwrap_name_ref().name().as_str(), "b");

        // Use a visitor to visit the number of `<` expressions in the tree.
        struct MyVisitor(usize);

        impl Visitor for MyVisitor {
            type State = ();

            fn expr(&mut self, _: &mut Self::State, reason: VisitReason, expr: &Expr) {
                if reason == VisitReason::Enter {
                    if let Expr::Less(_) = expr {
                        self.0 += 1;
                    }
                }
            }
        }

        let mut visitor = MyVisitor(0);
        ast.visit(&mut (), &mut visitor);
        assert_eq!(visitor.0, 1);
    }

    #[test]
    fn less_equal() {
        let parse = Document::parse(
            r#"
version 1.1

task test {
    Int a = 1
    Int b = 2
    Boolean c = a <= b
}
"#,
        );

        let document = parse.into_result().expect("there should be no errors");
        let ast = document.ast();
        let ast = ast.as_v1().expect("should be a V1 AST");
        let tasks: Vec<_> = ast.tasks().collect();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name().as_str(), "test");

        // Task declarations
        let decls: Vec<_> = tasks[0].declarations().collect();
        assert_eq!(decls.len(), 3);

        // First declaration
        assert_eq!(decls[0].ty().to_string(), "Int");
        assert_eq!(decls[0].name().as_str(), "a");
        assert_eq!(
            decls[0]
                .expr()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            1
        );

        // Second declaration
        assert_eq!(decls[1].ty().to_string(), "Int");
        assert_eq!(decls[1].name().as_str(), "b");
        assert_eq!(
            decls[1]
                .expr()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            2
        );

        // Third declaration
        assert_eq!(decls[2].ty().to_string(), "Boolean");
        assert_eq!(decls[2].name().as_str(), "c");
        let (lhs, rhs) = decls[2].expr().unwrap_less_equal().operands();
        assert_eq!(lhs.unwrap_name_ref().name().as_str(), "a");
        assert_eq!(rhs.unwrap_name_ref().name().as_str(), "b");

        // Use a visitor to count the number of `<=` expressions in the tree.
        struct MyVisitor(usize);

        impl Visitor for MyVisitor {
            type State = ();

            fn expr(&mut self, _: &mut Self::State, reason: VisitReason, expr: &Expr) {
                if reason == VisitReason::Enter {
                    if let Expr::LessEqual(_) = expr {
                        self.0 += 1;
                    }
                }
            }
        }

        let mut visitor = MyVisitor(0);
        ast.visit(&mut (), &mut visitor);
        assert_eq!(visitor.0, 1);
    }

    #[test]
    fn greater() {
        let parse = Document::parse(
            r#"
version 1.1

task test {
    Int a = 1
    Int b = 2
    Boolean c = a > b
}
"#,
        );

        let document = parse.into_result().expect("there should be no errors");
        let ast = document.ast();
        let ast = ast.as_v1().expect("should be a V1 AST");
        let tasks: Vec<_> = ast.tasks().collect();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name().as_str(), "test");

        // Task declarations
        let decls: Vec<_> = tasks[0].declarations().collect();
        assert_eq!(decls.len(), 3);

        // First declaration
        assert_eq!(decls[0].ty().to_string(), "Int");
        assert_eq!(decls[0].name().as_str(), "a");
        assert_eq!(
            decls[0]
                .expr()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            1
        );

        // Second declaration
        assert_eq!(decls[1].ty().to_string(), "Int");
        assert_eq!(decls[1].name().as_str(), "b");
        assert_eq!(
            decls[1]
                .expr()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            2
        );

        // Third declaration
        assert_eq!(decls[2].ty().to_string(), "Boolean");
        assert_eq!(decls[2].name().as_str(), "c");
        let (lhs, rhs) = decls[2].expr().unwrap_greater().operands();
        assert_eq!(lhs.unwrap_name_ref().name().as_str(), "a");
        assert_eq!(rhs.unwrap_name_ref().name().as_str(), "b");

        // Use a visitor to count the number of `>` expressions in the tree
        struct MyVisitor(usize);

        impl Visitor for MyVisitor {
            type State = ();

            fn expr(&mut self, _: &mut Self::State, reason: VisitReason, expr: &Expr) {
                if reason == VisitReason::Enter {
                    if let Expr::Greater(_) = expr {
                        self.0 += 1;
                    }
                }
            }
        }

        let mut visitor = MyVisitor(0);
        ast.visit(&mut (), &mut visitor);
        assert_eq!(visitor.0, 1);
    }

    #[test]
    fn greater_equal() {
        let parse = Document::parse(
            r#"
version 1.1

task test {
    Int a = 1
    Int b = 2
    Boolean c = a >= b
}
"#,
        );

        let document = parse.into_result().expect("there should be no errors");
        let ast = document.ast();
        let ast = ast.as_v1().expect("should be a V1 AST");
        let tasks: Vec<_> = ast.tasks().collect();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name().as_str(), "test");

        // Task declarations
        let decls: Vec<_> = tasks[0].declarations().collect();
        assert_eq!(decls.len(), 3);

        // First declaration
        assert_eq!(decls[0].ty().to_string(), "Int");
        assert_eq!(decls[0].name().as_str(), "a");
        assert_eq!(
            decls[0]
                .expr()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            1
        );

        // Second declaration
        assert_eq!(decls[1].ty().to_string(), "Int");
        assert_eq!(decls[1].name().as_str(), "b");
        assert_eq!(
            decls[1]
                .expr()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            2
        );

        // Third declaration
        assert_eq!(decls[2].ty().to_string(), "Boolean");
        assert_eq!(decls[2].name().as_str(), "c");
        let (lhs, rhs) = decls[2].expr().unwrap_greater_equal().operands();
        assert_eq!(lhs.unwrap_name_ref().name().as_str(), "a");
        assert_eq!(rhs.unwrap_name_ref().name().as_str(), "b");

        // Use a visitor to count the number of `>=` expressions in the tree.
        struct MyVisitor(usize);

        impl Visitor for MyVisitor {
            type State = ();

            fn expr(&mut self, _: &mut Self::State, reason: VisitReason, expr: &Expr) {
                if reason == VisitReason::Enter {
                    if let Expr::GreaterEqual(_) = expr {
                        self.0 += 1;
                    }
                }
            }
        }

        let mut visitor = MyVisitor(0);
        ast.visit(&mut (), &mut visitor);
        assert_eq!(visitor.0, 1);
    }

    #[test]
    fn addition() {
        let parse = Document::parse(
            r#"
version 1.1

task test {
    Int a = 1
    Int b = 2
    Int c = a + b
}
"#,
        );

        let document = parse.into_result().expect("there should be no errors");
        let ast = document.ast();
        let ast = ast.as_v1().expect("should be a V1 AST");
        let tasks: Vec<_> = ast.tasks().collect();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name().as_str(), "test");

        // Task declarations
        let decls: Vec<_> = tasks[0].declarations().collect();
        assert_eq!(decls.len(), 3);

        // First declaration
        assert_eq!(decls[0].ty().to_string(), "Int");
        assert_eq!(decls[0].name().as_str(), "a");
        assert_eq!(
            decls[0]
                .expr()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            1
        );

        // Second declaration
        assert_eq!(decls[1].ty().to_string(), "Int");
        assert_eq!(decls[1].name().as_str(), "b");
        assert_eq!(
            decls[1]
                .expr()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            2
        );

        // Third declaration
        assert_eq!(decls[2].ty().to_string(), "Int");
        assert_eq!(decls[2].name().as_str(), "c");
        let (lhs, rhs) = decls[2].expr().unwrap_addition().operands();
        assert_eq!(lhs.unwrap_name_ref().name().as_str(), "a");
        assert_eq!(rhs.unwrap_name_ref().name().as_str(), "b");

        // Use a visitor to count the number of addition expressions in the tree
        struct MyVisitor(usize);

        impl Visitor for MyVisitor {
            type State = ();

            fn expr(&mut self, _: &mut Self::State, reason: VisitReason, expr: &Expr) {
                if reason == VisitReason::Enter {
                    if let Expr::Addition(_) = expr {
                        self.0 += 1;
                    }
                }
            }
        }

        let mut visitor = MyVisitor(0);
        ast.visit(&mut (), &mut visitor);
        assert_eq!(visitor.0, 1);
    }

    #[test]
    fn subtraction() {
        let parse = Document::parse(
            r#"
version 1.1

task test {
    Int a = 1
    Int b = 2
    Int c = a - b
}
"#,
        );

        let document = parse.into_result().expect("there should be no errors");
        let ast = document.ast();
        let ast = ast.as_v1().expect("should be a V1 AST");
        let tasks: Vec<_> = ast.tasks().collect();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name().as_str(), "test");

        // Task declarations
        let decls: Vec<_> = tasks[0].declarations().collect();
        assert_eq!(decls.len(), 3);

        // First declaration
        assert_eq!(decls[0].ty().to_string(), "Int");
        assert_eq!(decls[0].name().as_str(), "a");
        assert_eq!(
            decls[0]
                .expr()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            1
        );

        // Second declaration
        assert_eq!(decls[1].ty().to_string(), "Int");
        assert_eq!(decls[1].name().as_str(), "b");
        assert_eq!(
            decls[1]
                .expr()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            2
        );

        // Third declaration
        assert_eq!(decls[2].ty().to_string(), "Int");
        assert_eq!(decls[2].name().as_str(), "c");
        let (lhs, rhs) = decls[2].expr().unwrap_subtraction().operands();
        assert_eq!(lhs.unwrap_name_ref().name().as_str(), "a");
        assert_eq!(rhs.unwrap_name_ref().name().as_str(), "b");

        // Use a visitor to count the number of subtraction expressions in the tree
        struct MyVisitor(usize);

        impl Visitor for MyVisitor {
            type State = ();

            fn expr(&mut self, _: &mut Self::State, reason: VisitReason, expr: &Expr) {
                if reason == VisitReason::Enter {
                    if let Expr::Subtraction(_) = expr {
                        self.0 += 1;
                    }
                }
            }
        }

        let mut visitor = MyVisitor(0);
        ast.visit(&mut (), &mut visitor);
        assert_eq!(visitor.0, 1);
    }

    #[test]
    fn multiplication() {
        let parse = Document::parse(
            r#"
version 1.1

task test {
    Int a = 1
    Int b = 2
    Int c = a * b
}
"#,
        );

        let document = parse.into_result().expect("there should be no errors");
        let ast = document.ast();
        let ast = ast.as_v1().expect("should be a V1 AST");
        let tasks: Vec<_> = ast.tasks().collect();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name().as_str(), "test");

        // Task declarations
        let decls: Vec<_> = tasks[0].declarations().collect();
        assert_eq!(decls.len(), 3);

        // First declaration
        assert_eq!(decls[0].ty().to_string(), "Int");
        assert_eq!(decls[0].name().as_str(), "a");
        assert_eq!(
            decls[0]
                .expr()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            1
        );

        // Second declaration
        assert_eq!(decls[1].ty().to_string(), "Int");
        assert_eq!(decls[1].name().as_str(), "b");
        assert_eq!(
            decls[1]
                .expr()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            2
        );

        // Third declaration
        assert_eq!(decls[2].ty().to_string(), "Int");
        assert_eq!(decls[2].name().as_str(), "c");
        let (lhs, rhs) = decls[2].expr().unwrap_multiplication().operands();
        assert_eq!(lhs.unwrap_name_ref().name().as_str(), "a");
        assert_eq!(rhs.unwrap_name_ref().name().as_str(), "b");

        // Use a visitor to count the number of multiplication expressions in the tree
        struct MyVisitor(usize);

        impl Visitor for MyVisitor {
            type State = ();

            fn expr(&mut self, _: &mut Self::State, reason: VisitReason, expr: &Expr) {
                if reason == VisitReason::Enter {
                    if let Expr::Multiplication(_) = expr {
                        self.0 += 1;
                    }
                }
            }
        }

        let mut visitor = MyVisitor(0);
        ast.visit(&mut (), &mut visitor);
        assert_eq!(visitor.0, 1);
    }

    #[test]
    fn division() {
        let parse = Document::parse(
            r#"
version 1.1

task test {
    Int a = 1
    Int b = 2
    Int c = a / b
}
"#,
        );

        let document = parse.into_result().expect("there should be no errors");
        let ast = document.ast();
        let ast = ast.as_v1().expect("should be a V1 AST");
        let tasks: Vec<_> = ast.tasks().collect();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name().as_str(), "test");

        // Task declarations
        let decls: Vec<_> = tasks[0].declarations().collect();
        assert_eq!(decls.len(), 3);

        // First declaration
        assert_eq!(decls[0].ty().to_string(), "Int");
        assert_eq!(decls[0].name().as_str(), "a");
        assert_eq!(
            decls[0]
                .expr()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            1
        );

        // Second declaration
        assert_eq!(decls[1].ty().to_string(), "Int");
        assert_eq!(decls[1].name().as_str(), "b");
        assert_eq!(
            decls[1]
                .expr()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            2
        );

        // Third declaration
        assert_eq!(decls[2].ty().to_string(), "Int");
        assert_eq!(decls[2].name().as_str(), "c");
        let (lhs, rhs) = decls[2].expr().unwrap_division().operands();
        assert_eq!(lhs.unwrap_name_ref().name().as_str(), "a");
        assert_eq!(rhs.unwrap_name_ref().name().as_str(), "b");

        // Use a visitor to count the number of division expressions in the tree
        struct MyVisitor(usize);

        impl Visitor for MyVisitor {
            type State = ();

            fn expr(&mut self, _: &mut Self::State, reason: VisitReason, expr: &Expr) {
                if reason == VisitReason::Enter {
                    if let Expr::Division(_) = expr {
                        self.0 += 1;
                    }
                }
            }
        }

        let mut visitor = MyVisitor(0);
        ast.visit(&mut (), &mut visitor);
        assert_eq!(visitor.0, 1);
    }

    #[test]
    fn modulo() {
        let parse = Document::parse(
            r#"
version 1.1

task test {
    Int a = 1
    Int b = 2
    Int c = a % b
}
"#,
        );

        let document = parse.into_result().expect("there should be no errors");
        let ast = document.ast();
        let ast = ast.as_v1().expect("should be a V1 AST");
        let tasks: Vec<_> = ast.tasks().collect();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name().as_str(), "test");

        // Task declarations
        let decls: Vec<_> = tasks[0].declarations().collect();
        assert_eq!(decls.len(), 3);

        // First declaration
        assert_eq!(decls[0].ty().to_string(), "Int");
        assert_eq!(decls[0].name().as_str(), "a");
        assert_eq!(
            decls[0]
                .expr()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            1
        );

        // Second declaration
        assert_eq!(decls[1].ty().to_string(), "Int");
        assert_eq!(decls[1].name().as_str(), "b");
        assert_eq!(
            decls[1]
                .expr()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            2
        );

        // Third declaration
        assert_eq!(decls[2].ty().to_string(), "Int");
        assert_eq!(decls[2].name().as_str(), "c");
        let (lhs, rhs) = decls[2].expr().unwrap_modulo().operands();
        assert_eq!(lhs.unwrap_name_ref().name().as_str(), "a");
        assert_eq!(rhs.unwrap_name_ref().name().as_str(), "b");

        // Use a visitor to count the number of modulo expressions in the tree
        struct MyVisitor(usize);

        impl Visitor for MyVisitor {
            type State = ();

            fn expr(&mut self, _: &mut Self::State, reason: VisitReason, expr: &Expr) {
                if reason == VisitReason::Enter {
                    if let Expr::Modulo(_) = expr {
                        self.0 += 1;
                    }
                }
            }
        }

        let mut visitor = MyVisitor(0);
        ast.visit(&mut (), &mut visitor);
        assert_eq!(visitor.0, 1);
    }

    #[test]
    fn call() {
        let parse = Document::parse(
            r#"
version 1.1

task test {
    Array[Int] a = [1, 2, 3]
    String b = sep(" ", a)
}
"#,
        );

        let document = parse.into_result().expect("there should be no errors");
        let ast = document.ast();
        let ast = ast.as_v1().expect("should be a V1 AST");
        let tasks: Vec<_> = ast.tasks().collect();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name().as_str(), "test");

        // Task declarations
        let decls: Vec<_> = tasks[0].declarations().collect();
        assert_eq!(decls.len(), 2);

        // First declaration
        assert_eq!(decls[0].ty().to_string(), "Array[Int]");
        assert_eq!(decls[0].name().as_str(), "a");
        let elements: Vec<_> = decls[0]
            .expr()
            .unwrap_literal()
            .unwrap_array()
            .elements()
            .collect();
        assert_eq!(elements.len(), 3);
        assert_eq!(
            elements[0]
                .clone()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            1
        );
        assert_eq!(
            elements[1]
                .clone()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            2
        );
        assert_eq!(
            elements[2]
                .clone()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            3
        );

        // Second declaration
        assert_eq!(decls[1].ty().to_string(), "String");
        assert_eq!(decls[1].name().as_str(), "b");
        let call = decls[1].expr().unwrap_call();
        assert_eq!(call.target().unwrap_name_ref().name().as_str(), "sep");
        let args: Vec<_> = call.arguments().collect();
        assert_eq!(args.len(), 2);
        assert_eq!(
            args[0]
                .clone()
                .unwrap_literal()
                .unwrap_string()
                .text()
                .unwrap()
                .as_str(),
            " "
        );
        assert_eq!(args[1].clone().unwrap_name_ref().name().as_str(), "a");

        // Use a visitor to count the number of call expressions in the tree
        struct MyVisitor(usize);

        impl Visitor for MyVisitor {
            type State = ();

            fn expr(&mut self, _: &mut Self::State, reason: VisitReason, expr: &Expr) {
                if reason == VisitReason::Enter {
                    if let Expr::Call(_) = expr {
                        self.0 += 1;
                    }
                }
            }
        }

        let mut visitor = MyVisitor(0);
        ast.visit(&mut (), &mut visitor);
        assert_eq!(visitor.0, 1);
    }

    #[test]
    fn index() {
        let parse = Document::parse(
            r#"
version 1.1

task test {
    Array[Int] a = [1, 2, 3]
    Int b = a[1]
}
"#,
        );

        let document = parse.into_result().expect("there should be no errors");
        let ast = document.ast();
        let ast = ast.as_v1().expect("should be a V1 AST");
        let tasks: Vec<_> = ast.tasks().collect();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name().as_str(), "test");

        // Task declarations
        let decls: Vec<_> = tasks[0].declarations().collect();
        assert_eq!(decls.len(), 2);

        // First declaration
        assert_eq!(decls[0].ty().to_string(), "Array[Int]");
        assert_eq!(decls[0].name().as_str(), "a");
        let elements: Vec<_> = decls[0]
            .expr()
            .unwrap_literal()
            .unwrap_array()
            .elements()
            .collect();
        assert_eq!(elements.len(), 3);
        assert_eq!(
            elements[0]
                .clone()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            1
        );
        assert_eq!(
            elements[1]
                .clone()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            2
        );
        assert_eq!(
            elements[2]
                .clone()
                .unwrap_literal()
                .unwrap_integer()
                .value()
                .unwrap(),
            3
        );

        // Second declaration
        assert_eq!(decls[1].ty().to_string(), "Int");
        assert_eq!(decls[1].name().as_str(), "b");
        let (expr, index) = decls[1].expr().unwrap_index().operands();
        assert_eq!(expr.unwrap_name_ref().name().as_str(), "a");
        assert_eq!(index.unwrap_literal().unwrap_integer().value().unwrap(), 1);

        // Use a visitor to count the number of index expressions in the tree
        struct MyVisitor(usize);

        impl Visitor for MyVisitor {
            type State = ();

            fn expr(&mut self, _: &mut Self::State, reason: VisitReason, expr: &Expr) {
                if reason == VisitReason::Enter {
                    if let Expr::Index(_) = expr {
                        self.0 += 1;
                    }
                }
            }
        }

        let mut visitor = MyVisitor(0);
        ast.visit(&mut (), &mut visitor);
        assert_eq!(visitor.0, 1);
    }

    #[test]
    fn access() {
        let parse = Document::parse(
            r#"
version 1.1

task test {
    Object a = object { foo: "bar" }
    String b = a.foo
}
"#,
        );

        let document = parse.into_result().expect("there should be no errors");
        let ast = document.ast();
        let ast = ast.as_v1().expect("should be a V1 AST");
        let tasks: Vec<_> = ast.tasks().collect();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name().as_str(), "test");

        // Task declarations
        let decls: Vec<_> = tasks[0].declarations().collect();
        assert_eq!(decls.len(), 2);

        // First declaration
        assert_eq!(decls[0].ty().to_string(), "Object");
        assert_eq!(decls[0].name().as_str(), "a");
        let items: Vec<_> = decls[0]
            .expr()
            .unwrap_literal()
            .unwrap_object()
            .items()
            .collect();
        assert_eq!(items.len(), 1);
        let (name, value) = items[0].name_value();
        assert_eq!(name.as_str(), "foo");
        assert_eq!(
            value
                .unwrap_literal()
                .unwrap_string()
                .text()
                .unwrap()
                .as_str(),
            "bar"
        );

        // Second declaration
        assert_eq!(decls[1].ty().to_string(), "String");
        assert_eq!(decls[1].name().as_str(), "b");
        let (expr, index) = decls[1].expr().unwrap_access().operands();
        assert_eq!(expr.unwrap_name_ref().name().as_str(), "a");
        assert_eq!(index.as_str(), "foo");

        // Use a visitor to count the number of access expressions in the tree
        struct MyVisitor(usize);

        impl Visitor for MyVisitor {
            type State = ();

            fn expr(&mut self, _: &mut Self::State, reason: VisitReason, expr: &Expr) {
                if reason == VisitReason::Enter {
                    if let Expr::Access(_) = expr {
                        self.0 += 1;
                    }
                }
            }
        }

        let mut visitor = MyVisitor(0);
        ast.visit(&mut (), &mut visitor);
        assert_eq!(visitor.0, 1);
    }
}
