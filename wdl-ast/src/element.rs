//! An element representation for every node/token in the AST.

use crate::v1::*;
use crate::AstNode;
use crate::AstToken;
use crate::Comment;
use crate::Ident;
use crate::SyntaxElement;
use crate::SyntaxKind;
use crate::Version;
use crate::VersionStatement;
use crate::Whitespace;

/// An abstract syntax element.
///
/// A member is included for every [`SyntaxNode`](crate::SyntaxNode) and
/// [`SyntaxToken`](crate::SyntaxToken).
#[derive(Debug)]
pub enum AstElement {
    /// An access expression.
    AccessExpr(AccessExpr),
    /// An addition expression.
    AdditionExpr(AdditionExpr),
    /// The `after` keyword.
    AfterKeyword(AfterKeyword),
    /// The `alias` keyword.
    AliasKeyword(AliasKeyword),
    /// An array type.
    ArrayType(ArrayType),
    /// The `Array` type keyword.
    ArrayTypeKeyword(ArrayTypeKeyword),
    /// The `as` keyword.
    AsKeyword(AsKeyword),
    /// The `=` symbol.
    Assignment(Assignment),
    /// A V1 abstract syntax tree.
    Ast(Ast),
    /// The `*` symbol.
    Asterisk(Asterisk),
    /// The `Boolean` type keyword.
    BooleanTypeKeyword(BooleanTypeKeyword),
    /// A bound declaration.
    BoundDecl(BoundDecl),
    /// An after clause in a call statement.
    CallAfter(CallAfter),
    /// An alias clause in a call statement.
    CallAlias(CallAlias),
    /// A call expression.
    CallExpr(CallExpr),
    /// A call input item.
    CallInputItem(CallInputItem),
    /// The `call` keyword.
    CallKeyword(CallKeyword),
    /// A call statement.
    CallStatement(CallStatement),
    /// A target within a call statement.
    CallTarget(CallTarget),
    /// The `}` symbol.
    CloseBrace(CloseBrace),
    /// The `]` symbol.
    CloseBracket(CloseBracket),
    /// The `>>>` symbol.
    CloseHeredoc(CloseHeredoc),
    /// The `)` symbol.
    CloseParen(CloseParen),
    /// The `:` symbol.
    Colon(Colon),
    /// The `,` symbol.
    Comma(Comma),
    /// The `command` keyword.
    CommandKeyword(CommandKeyword),
    /// A command section.
    CommandSection(CommandSection),
    /// The text within a command section.
    CommandText(CommandText),
    /// A comment.
    Comment(Comment),
    /// A conditional statement.
    ConditionalStatement(ConditionalStatement),
    /// The `default` placeholder option.
    DefaultOption(DefaultOption),
    /// The `Directory` type keyword.
    DirectoryTypeKeyword(DirectoryTypeKeyword),
    /// A division expression.
    DivisionExpr(DivisionExpr),
    /// The `.` symbol.
    Dot(Dot),
    /// The `"` symbol.
    DoubleQuote(DoubleQuote),
    /// The `else` keyword.
    ElseKeyword(ElseKeyword),
    /// The `==` symbol.
    Equal(Equal),
    /// An equality expression.
    EqualityExpr(EqualityExpr),
    /// The `!` symbol.
    Exclamation(Exclamation),
    /// The `**` symbol.
    Exponentiation(Exponentiation),
    /// An exponentiation expression.
    ExponentiationExpr(ExponentiationExpr),
    /// The `false` keyword.
    FalseKeyword(FalseKeyword),
    /// The `File` type keyword.
    FileTypeKeyword(FileTypeKeyword),
    /// A float.
    Float(Float),
    /// The `Float` type keyword.
    FloatTypeKeyword(FloatTypeKeyword),
    /// The `>` symbol.
    Greater(Greater),
    /// The `>=` symbol.
    GreaterEqual(GreaterEqual),
    /// A greater than or equal to expression.
    GreaterEqualExpr(GreaterEqualExpr),
    /// A greater than expression.
    GreaterExpr(GreaterExpr),
    /// An item within a hints section.
    HintsItem(HintsItem),
    /// The `hints` keyword.
    HintsKeyword(HintsKeyword),
    /// A hints section.
    HintsSection(HintsSection),
    /// An identity.
    Ident(Ident),
    /// An if expression.
    IfExpr(IfExpr),
    /// The `if` keyword.
    IfKeyword(IfKeyword),
    /// An import alias.
    ImportAlias(ImportAlias),
    /// The `import` keyword.
    ImportKeyword(ImportKeyword),
    /// An import statement.
    ImportStatement(ImportStatement),
    /// An index expression.
    IndexExpr(IndexExpr),
    /// An inequality expression.
    InequalityExpr(InequalityExpr),
    /// The `in` keyword.
    InKeyword(InKeyword),
    /// The `input` keyword.
    InputKeyword(InputKeyword),
    /// An input section.
    InputSection(InputSection),
    /// An integer.
    Integer(Integer),
    /// The `Int` type keyword.
    IntTypeKeyword(IntTypeKeyword),
    /// The `<` symbol.
    Less(Less),
    /// The `<=` symbol.
    LessEqual(LessEqual),
    /// A less than or equal to expression.
    LessEqualExpr(LessEqualExpr),
    /// A less than expression.
    LessExpr(LessExpr),
    /// A literal array.
    LiteralArray(LiteralArray),
    /// A literal boolean.
    LiteralBoolean(LiteralBoolean),
    /// A literal float.
    LiteralFloat(LiteralFloat),
    /// A literal hints.
    LiteralHints(LiteralHints),
    /// A literal hints item.
    LiteralHintsItem(LiteralHintsItem),
    /// A literal input.
    LiteralInput(LiteralInput),
    /// A literal input item.
    LiteralInputItem(LiteralInputItem),
    /// A literal integer.
    LiteralInteger(LiteralInteger),
    /// A literal map.
    LiteralMap(LiteralMap),
    /// A literal map item.
    LiteralMapItem(LiteralMapItem),
    /// A literal none.
    LiteralNone(LiteralNone),
    /// A literal null.
    LiteralNull(LiteralNull),
    /// A literal object.
    LiteralObject(LiteralObject),
    /// A literal object item.
    LiteralObjectItem(LiteralObjectItem),
    /// A literal output.
    LiteralOutput(LiteralOutput),
    /// A literal output item.
    LiteralOutputItem(LiteralOutputItem),
    /// A literal pair.
    LiteralPair(LiteralPair),
    /// A literal string.
    LiteralString(LiteralString),
    /// A literal struct.
    LiteralStruct(LiteralStruct),
    /// A literal struct item.
    LiteralStructItem(LiteralStructItem),
    /// The `&&` symbol.
    LogicalAnd(LogicalAnd),
    /// A logical and expression.
    LogicalAndExpr(LogicalAndExpr),
    /// A logical not expression.
    LogicalNotExpr(LogicalNotExpr),
    /// The `||` symbol.
    LogicalOr(LogicalOr),
    /// A logical or expression.
    LogicalOrExpr(LogicalOrExpr),
    /// A map type.
    MapType(MapType),
    /// The `Map` type keyword.
    MapTypeKeyword(MapTypeKeyword),
    /// A metadata array.
    MetadataArray(MetadataArray),
    /// A metadata object.
    MetadataObject(MetadataObject),
    /// A metadata object item.
    MetadataObjectItem(MetadataObjectItem),
    /// A metadata section.
    MetadataSection(MetadataSection),
    /// The `meta` keyword.
    MetaKeyword(MetaKeyword),
    /// The `-` symbol.
    Minus(Minus),
    /// A modulo expression.
    ModuloExpr(ModuloExpr),
    /// A multiplication expression.
    MultiplicationExpr(MultiplicationExpr),
    /// A reference to a name.
    NameRef(NameRef),
    /// A negation expression.
    NegationExpr(NegationExpr),
    /// The `None` keyword.
    NoneKeyword(NoneKeyword),
    /// The `!=` symbol.
    NotEqual(NotEqual),
    /// The `null` keyword.
    NullKeyword(NullKeyword),
    /// The `object` keyword.
    ObjectKeyword(ObjectKeyword),
    /// An object type.
    ObjectType(ObjectType),
    /// The `Object` type keyword.
    ObjectTypeKeyword(ObjectTypeKeyword),
    /// The `{` symbol.
    OpenBrace(OpenBrace),
    /// The `[` symbol.
    OpenBracket(OpenBracket),
    /// The `<<<` symbol.
    OpenHeredoc(OpenHeredoc),
    /// The `(` symbol.
    OpenParen(OpenParen),
    /// The `output` keyword.
    OutputKeyword(OutputKeyword),
    /// An output section.
    OutputSection(OutputSection),
    /// A pair type.
    PairType(PairType),
    /// The `Pair` type keyword.
    PairTypeKeyword(PairTypeKeyword),
    /// A parameter metadata section.
    ParameterMetadataSection(ParameterMetadataSection),
    /// The `parameter_meta` keyword.
    ParameterMetaKeyword(ParameterMetaKeyword),
    /// A parenthesized expression.
    ParenthesizedExpr(ParenthesizedExpr),
    /// The `%` symbol.
    Percent(Percent),
    /// A placeholder.
    Placeholder(Placeholder),
    /// One of the placeholder open symbols.
    PlaceholderOpen(PlaceholderOpen),
    /// The `+` symbol.
    Plus(Plus),
    /// A primitive type.
    PrimitiveType(PrimitiveType),
    /// The `?` symbol.
    QuestionMark(QuestionMark),
    /// A requirements item.
    RequirementsItem(RequirementsItem),
    /// The `requirements` keyword.
    RequirementsKeyword(RequirementsKeyword),
    /// A requirements section.
    RequirementsSection(RequirementsSection),
    /// A runtime item.
    RuntimeItem(RuntimeItem),
    /// The `runtime` keyword.
    RuntimeKeyword(RuntimeKeyword),
    /// A runtime section.
    RuntimeSection(RuntimeSection),
    /// The `scatter` keyword.
    ScatterKeyword(ScatterKeyword),
    /// A scatter statement.
    ScatterStatement(ScatterStatement),
    /// The `sep` placeholder option.
    SepOption(SepOption),
    /// The `'` symbol.
    SingleQuote(SingleQuote),
    /// The `/` symbol.
    Slash(Slash),
    /// The textual part of a string.
    StringText(StringText),
    /// The `String` type keyword.
    StringTypeKeyword(StringTypeKeyword),
    /// A struct definition.
    StructDefinition(StructDefinition),
    /// The `struct` keyword.
    StructKeyword(StructKeyword),
    /// A subtraction expression.
    SubtractionExpr(SubtractionExpr),
    /// A task definition.
    TaskDefinition(TaskDefinition),
    /// The `task` keyword.
    TaskKeyword(TaskKeyword),
    /// The `then` keyword.
    ThenKeyword(ThenKeyword),
    /// A `true`/`false` placeholder option.
    TrueFalseOption(TrueFalseOption),
    /// The `true` keyword.
    TrueKeyword(TrueKeyword),
    /// A reference to a type.
    TypeRef(TypeRef),
    /// An unbound declaration.
    UnboundDecl(UnboundDecl),
    /// A version.
    Version(Version),
    /// The `version` keyword.
    VersionKeyword(VersionKeyword),
    /// A version statement.
    VersionStatement(VersionStatement),
    /// Whitespace.
    Whitespace(Whitespace),
    /// A workflow definition.
    WorkflowDefinition(WorkflowDefinition),
    /// The `workflow` keyword.
    WorkflowKeyword(WorkflowKeyword),
}

#[macropol::macropol]
macro_rules! as_into_unwrap {
    ($suffix:ident, $inner:ty, $variant:path) => {
        paste::paste! {
            /// Attempts to get a reference to the inner [`${stringify!($inner)}`].
            ///
            /// * If `self` is a [`${stringify!($variant)}`], then a reference to the
            ///   inner [`${stringify!($inner)}`] wrapped in [`Some`] is returned.
            /// * Else, [`None`] is returned.
            pub fn [<as_ $suffix>](&self) -> Option<&$inner> {
                match self {
                    $variant($suffix) => Some($suffix),
                    _ => None,
                }
            }

            /// Consumes `self` and attempts to return the inner
            /// [`${stringify!($inner)}`].
            ///
            /// * If `self` is a [`${stringify!($variant)}`], then the inner
            ///   [`${stringify!($inner)}`] wrapped in [`Some`] is returned.
            /// * Else, [`None`] is returned.
            pub fn [<into_ $suffix>](self) -> Option<$inner> {
                match self {
                    $variant($suffix) => Some($suffix),
                    _ => None,
                }
            }

            /// Consumes `self` and returns the inner [`${stringify!($inner)}`].
            ///
            /// # Panics
            ///
            /// If `self` is not a [`${stringify!($variant)}`].
            pub fn [<unwrap_ $suffix>](self) -> $inner {
                self.[<into_ $suffix>]().expect(concat!(
                    "Expected `",
                    stringify!($variant),
                    "` but got a different variant"
                ))
            }
        }
    };
}

impl AstElement {
    as_into_unwrap!(access_expr, AccessExpr, AstElement::AccessExpr);

    as_into_unwrap!(addition_expr, AdditionExpr, AstElement::AdditionExpr);

    as_into_unwrap!(after_keyword, AfterKeyword, AstElement::AfterKeyword);

    as_into_unwrap!(alias_keyword, AliasKeyword, AstElement::AliasKeyword);

    as_into_unwrap!(array_type, ArrayType, AstElement::ArrayType);

    as_into_unwrap!(
        array_type_keyword,
        ArrayTypeKeyword,
        AstElement::ArrayTypeKeyword
    );

    as_into_unwrap!(as_keyword, AsKeyword, AstElement::AsKeyword);

    as_into_unwrap!(assignment, Assignment, AstElement::Assignment);

    as_into_unwrap!(ast, Ast, AstElement::Ast);

    as_into_unwrap!(asterisk, Asterisk, AstElement::Asterisk);

    as_into_unwrap!(
        boolean_type_keyword,
        BooleanTypeKeyword,
        AstElement::BooleanTypeKeyword
    );

    as_into_unwrap!(bound_decl, BoundDecl, AstElement::BoundDecl);

    as_into_unwrap!(call_after, CallAfter, AstElement::CallAfter);

    as_into_unwrap!(call_alias, CallAlias, AstElement::CallAlias);

    as_into_unwrap!(call_expr, CallExpr, AstElement::CallExpr);

    as_into_unwrap!(call_input_item, CallInputItem, AstElement::CallInputItem);

    as_into_unwrap!(call_keyword, CallKeyword, AstElement::CallKeyword);

    as_into_unwrap!(call_statement, CallStatement, AstElement::CallStatement);

    as_into_unwrap!(call_target, CallTarget, AstElement::CallTarget);

    as_into_unwrap!(close_brace, CloseBrace, AstElement::CloseBrace);

    as_into_unwrap!(close_brack, CloseBracket, AstElement::CloseBracket);

    as_into_unwrap!(close_heredoc, CloseHeredoc, AstElement::CloseHeredoc);

    as_into_unwrap!(close_paren, CloseParen, AstElement::CloseParen);

    as_into_unwrap!(colon, Colon, AstElement::Colon);

    as_into_unwrap!(comma, Comma, AstElement::Comma);

    as_into_unwrap!(command_keyword, CommandKeyword, AstElement::CommandKeyword);

    as_into_unwrap!(command_section, CommandSection, AstElement::CommandSection);

    as_into_unwrap!(command_text, CommandText, AstElement::CommandText);

    as_into_unwrap!(comment, Comment, AstElement::Comment);

    as_into_unwrap!(
        conditional_statement,
        ConditionalStatement,
        AstElement::ConditionalStatement
    );

    as_into_unwrap!(default_option, DefaultOption, AstElement::DefaultOption);

    as_into_unwrap!(
        directory_type_keyword,
        DirectoryTypeKeyword,
        AstElement::DirectoryTypeKeyword
    );

    as_into_unwrap!(division_expr, DivisionExpr, AstElement::DivisionExpr);

    as_into_unwrap!(dot, Dot, AstElement::Dot);

    as_into_unwrap!(double_quote, DoubleQuote, AstElement::DoubleQuote);

    as_into_unwrap!(else_keyword, ElseKeyword, AstElement::ElseKeyword);

    as_into_unwrap!(equal, Equal, AstElement::Equal);

    as_into_unwrap!(equality_expr, EqualityExpr, AstElement::EqualityExpr);

    as_into_unwrap!(exclaimation, Exclamation, AstElement::Exclamation);

    as_into_unwrap!(exponentiation, Exponentiation, AstElement::Exponentiation);

    as_into_unwrap!(
        exponentiation_expr,
        ExponentiationExpr,
        AstElement::ExponentiationExpr
    );

    as_into_unwrap!(false_keyword, FalseKeyword, AstElement::FalseKeyword);

    as_into_unwrap!(
        file_type_keyword,
        FileTypeKeyword,
        AstElement::FileTypeKeyword
    );

    as_into_unwrap!(float, Float, AstElement::Float);

    as_into_unwrap!(
        float_type_keyword,
        FloatTypeKeyword,
        AstElement::FloatTypeKeyword
    );

    as_into_unwrap!(greater, Greater, AstElement::Greater);

    as_into_unwrap!(greater_equal, GreaterEqual, AstElement::GreaterEqual);

    as_into_unwrap!(
        greater_equal_expr,
        GreaterEqualExpr,
        AstElement::GreaterEqualExpr
    );

    as_into_unwrap!(greater_expr, GreaterExpr, AstElement::GreaterExpr);

    as_into_unwrap!(hints_item, HintsItem, AstElement::HintsItem);

    as_into_unwrap!(hints_keyword, HintsKeyword, AstElement::HintsKeyword);

    as_into_unwrap!(hints_section, HintsSection, AstElement::HintsSection);

    as_into_unwrap!(ident, Ident, AstElement::Ident);

    as_into_unwrap!(if_expr, IfExpr, AstElement::IfExpr);

    as_into_unwrap!(if_keyword, IfKeyword, AstElement::IfKeyword);

    as_into_unwrap!(import_alias, ImportAlias, AstElement::ImportAlias);

    as_into_unwrap!(import_keyword, ImportKeyword, AstElement::ImportKeyword);

    as_into_unwrap!(
        import_statement,
        ImportStatement,
        AstElement::ImportStatement
    );

    as_into_unwrap!(index_expr, IndexExpr, AstElement::IndexExpr);

    as_into_unwrap!(inequality_expr, InequalityExpr, AstElement::InequalityExpr);

    as_into_unwrap!(in_keyword, InKeyword, AstElement::InKeyword);

    as_into_unwrap!(input_keyword, InputKeyword, AstElement::InputKeyword);

    as_into_unwrap!(input_section, InputSection, AstElement::InputSection);

    as_into_unwrap!(integer, Integer, AstElement::Integer);

    as_into_unwrap!(int_type_keyword, IntTypeKeyword, AstElement::IntTypeKeyword);

    as_into_unwrap!(less, Less, AstElement::Less);

    as_into_unwrap!(less_equal, LessEqual, AstElement::LessEqual);

    as_into_unwrap!(less_equal_expr, LessEqualExpr, AstElement::LessEqualExpr);

    as_into_unwrap!(less_expr, LessExpr, AstElement::LessExpr);

    as_into_unwrap!(literal_array, LiteralArray, AstElement::LiteralArray);

    as_into_unwrap!(literal_boolean, LiteralBoolean, AstElement::LiteralBoolean);

    as_into_unwrap!(literal_float, LiteralFloat, AstElement::LiteralFloat);

    as_into_unwrap!(literal_hints, LiteralHints, AstElement::LiteralHints);

    as_into_unwrap!(
        literal_hints_item,
        LiteralHintsItem,
        AstElement::LiteralHintsItem
    );

    as_into_unwrap!(literal_input, LiteralInput, AstElement::LiteralInput);

    as_into_unwrap!(
        literal_input_item,
        LiteralInputItem,
        AstElement::LiteralInputItem
    );

    as_into_unwrap!(literal_integer, LiteralInteger, AstElement::LiteralInteger);

    as_into_unwrap!(literal_map, LiteralMap, AstElement::LiteralMap);

    as_into_unwrap!(literal_map_item, LiteralMapItem, AstElement::LiteralMapItem);

    as_into_unwrap!(literal_none, LiteralNone, AstElement::LiteralNone);

    as_into_unwrap!(literal_null, LiteralNull, AstElement::LiteralNull);

    as_into_unwrap!(literal_object, LiteralObject, AstElement::LiteralObject);

    as_into_unwrap!(
        literal_object_item,
        LiteralObjectItem,
        AstElement::LiteralObjectItem
    );

    as_into_unwrap!(literal_output, LiteralOutput, AstElement::LiteralOutput);

    as_into_unwrap!(
        literal_output_item,
        LiteralOutputItem,
        AstElement::LiteralOutputItem
    );

    as_into_unwrap!(literal_pair, LiteralPair, AstElement::LiteralPair);

    as_into_unwrap!(literal_string, LiteralString, AstElement::LiteralString);

    as_into_unwrap!(literal_struct, LiteralStruct, AstElement::LiteralStruct);

    as_into_unwrap!(
        literal_struct_item,
        LiteralStructItem,
        AstElement::LiteralStructItem
    );

    as_into_unwrap!(logical_and, LogicalAnd, AstElement::LogicalAnd);

    as_into_unwrap!(logical_and_expr, LogicalAndExpr, AstElement::LogicalAndExpr);

    as_into_unwrap!(logical_not_expr, LogicalNotExpr, AstElement::LogicalNotExpr);

    as_into_unwrap!(logical_or, LogicalOr, AstElement::LogicalOr);

    as_into_unwrap!(logical_or_expr, LogicalOrExpr, AstElement::LogicalOrExpr);

    as_into_unwrap!(map_type, MapType, AstElement::MapType);

    as_into_unwrap!(map_type_keyword, MapTypeKeyword, AstElement::MapTypeKeyword);

    as_into_unwrap!(metadata_array, MetadataArray, AstElement::MetadataArray);

    as_into_unwrap!(metadata_object, MetadataObject, AstElement::MetadataObject);

    as_into_unwrap!(
        metadata_object_item,
        MetadataObjectItem,
        AstElement::MetadataObjectItem
    );

    as_into_unwrap!(
        metadata_section,
        MetadataSection,
        AstElement::MetadataSection
    );

    as_into_unwrap!(meta_keyword, MetaKeyword, AstElement::MetaKeyword);

    as_into_unwrap!(minus, Minus, AstElement::Minus);

    as_into_unwrap!(modulo_expr, ModuloExpr, AstElement::ModuloExpr);

    as_into_unwrap!(
        multiplication_expr,
        MultiplicationExpr,
        AstElement::MultiplicationExpr
    );

    as_into_unwrap!(name_ref, NameRef, AstElement::NameRef);

    as_into_unwrap!(negation_expr, NegationExpr, AstElement::NegationExpr);

    as_into_unwrap!(none_keyword, NoneKeyword, AstElement::NoneKeyword);

    as_into_unwrap!(not_equal, NotEqual, AstElement::NotEqual);

    as_into_unwrap!(null_keyword, NullKeyword, AstElement::NullKeyword);

    as_into_unwrap!(object_keyword, ObjectKeyword, AstElement::ObjectKeyword);

    as_into_unwrap!(object_type, ObjectType, AstElement::ObjectType);

    as_into_unwrap!(
        object_type_keyword,
        ObjectTypeKeyword,
        AstElement::ObjectTypeKeyword
    );

    as_into_unwrap!(open_brace, OpenBrace, AstElement::OpenBrace);

    as_into_unwrap!(open_bracket, OpenBracket, AstElement::OpenBracket);

    as_into_unwrap!(open_heredoc, OpenHeredoc, AstElement::OpenHeredoc);

    as_into_unwrap!(open_paren, OpenParen, AstElement::OpenParen);

    as_into_unwrap!(output_keyword, OutputKeyword, AstElement::OutputKeyword);

    as_into_unwrap!(output_section, OutputSection, AstElement::OutputSection);

    as_into_unwrap!(pair_type, PairType, AstElement::PairType);

    as_into_unwrap!(
        pair_type_keyword,
        PairTypeKeyword,
        AstElement::PairTypeKeyword
    );

    as_into_unwrap!(
        parameter_metadata_section,
        ParameterMetadataSection,
        AstElement::ParameterMetadataSection
    );

    as_into_unwrap!(
        parameter_meta_keyword,
        ParameterMetaKeyword,
        AstElement::ParameterMetaKeyword
    );

    as_into_unwrap!(
        parenthesized_expr,
        ParenthesizedExpr,
        AstElement::ParenthesizedExpr
    );

    as_into_unwrap!(percent, Percent, AstElement::Percent);

    as_into_unwrap!(placeholder, Placeholder, AstElement::Placeholder);

    as_into_unwrap!(
        placeholder_open,
        PlaceholderOpen,
        AstElement::PlaceholderOpen
    );

    as_into_unwrap!(plus, Plus, AstElement::Plus);

    as_into_unwrap!(primitive_type, PrimitiveType, AstElement::PrimitiveType);

    as_into_unwrap!(question_mark, QuestionMark, AstElement::QuestionMark);

    as_into_unwrap!(
        requirements_item,
        RequirementsItem,
        AstElement::RequirementsItem
    );

    as_into_unwrap!(
        requirements_keyword,
        RequirementsKeyword,
        AstElement::RequirementsKeyword
    );

    as_into_unwrap!(
        requirements_section,
        RequirementsSection,
        AstElement::RequirementsSection
    );

    as_into_unwrap!(runtime_item, RuntimeItem, AstElement::RuntimeItem);

    as_into_unwrap!(runtime_keyword, RuntimeKeyword, AstElement::RuntimeKeyword);

    as_into_unwrap!(runtime_section, RuntimeSection, AstElement::RuntimeSection);

    as_into_unwrap!(scatter_keyword, ScatterKeyword, AstElement::ScatterKeyword);

    as_into_unwrap!(
        scatter_statement,
        ScatterStatement,
        AstElement::ScatterStatement
    );

    as_into_unwrap!(sep_option, SepOption, AstElement::SepOption);

    as_into_unwrap!(single_quote, SingleQuote, AstElement::SingleQuote);

    as_into_unwrap!(slash, Slash, AstElement::Slash);

    as_into_unwrap!(string_text, StringText, AstElement::StringText);

    as_into_unwrap!(
        string_type_keyword,
        StringTypeKeyword,
        AstElement::StringTypeKeyword
    );

    as_into_unwrap!(
        struct_definition,
        StructDefinition,
        AstElement::StructDefinition
    );

    as_into_unwrap!(struct_keyword, StructKeyword, AstElement::StructKeyword);

    as_into_unwrap!(
        subtraction_expr,
        SubtractionExpr,
        AstElement::SubtractionExpr
    );

    as_into_unwrap!(task_definition, TaskDefinition, AstElement::TaskDefinition);

    as_into_unwrap!(task_keyword, TaskKeyword, AstElement::TaskKeyword);

    as_into_unwrap!(then_keyword, ThenKeyword, AstElement::ThenKeyword);

    as_into_unwrap!(
        true_false_option,
        TrueFalseOption,
        AstElement::TrueFalseOption
    );

    as_into_unwrap!(true_keyword, TrueKeyword, AstElement::TrueKeyword);

    as_into_unwrap!(type_ref, TypeRef, AstElement::TypeRef);

    as_into_unwrap!(unbound_decl, UnboundDecl, AstElement::UnboundDecl);

    as_into_unwrap!(version, Version, AstElement::Version);

    as_into_unwrap!(version_keyword, VersionKeyword, AstElement::VersionKeyword);

    as_into_unwrap!(
        version_statement,
        VersionStatement,
        AstElement::VersionStatement
    );

    as_into_unwrap!(whitespace, Whitespace, AstElement::Whitespace);

    as_into_unwrap!(
        workflow_definition,
        WorkflowDefinition,
        AstElement::WorkflowDefinition
    );

    as_into_unwrap!(
        workflow_keyword,
        WorkflowKeyword,
        AstElement::WorkflowKeyword
    );

    /// Attempts to cast a [`SyntaxElement`] to its analogous [`AstElement`].
    pub fn cast(syntax: SyntaxElement) -> Option<Self> {
        match syntax.kind() {
            SyntaxKind::AccessExprNode => Some(Self::AccessExpr(
                AccessExpr::cast(
                    syntax
                        .into_node()
                        .expect("`AccessExprNode` should always cast to a node"),
                )
                .expect("`AccessExpr` should cast"),
            )),
            SyntaxKind::AdditionExprNode => Some(Self::AdditionExpr(
                AdditionExpr::cast(
                    syntax
                        .into_node()
                        .expect("`AdditionExprNode` should always cast to a node"),
                )
                .expect("`AdditionExpr` should cast"),
            )),
            SyntaxKind::AfterKeyword => Some(Self::AfterKeyword(
                AfterKeyword::cast(
                    syntax
                        .into_token()
                        .expect("`AfterKeyword` should always cast to a token"),
                )
                .expect("`AfterKeyword` should cast"),
            )),
            SyntaxKind::AliasKeyword => Some(Self::AliasKeyword(
                AliasKeyword::cast(
                    syntax
                        .into_token()
                        .expect("`AliasKeyword` should always cast to a token"),
                )
                .expect("`AliasKeyword` should cast"),
            )),
            SyntaxKind::ArrayTypeKeyword => Some(Self::ArrayTypeKeyword(
                ArrayTypeKeyword::cast(
                    syntax
                        .into_token()
                        .expect("`ArrayTypeKeyword` should always cast to a token"),
                )
                .expect("`ArrayTypeKeyword` should cast"),
            )),
            SyntaxKind::ArrayTypeNode => Some(Self::ArrayType(
                ArrayType::cast(
                    syntax
                        .into_node()
                        .expect("`ArrayTypeNode` should always cast to a node"),
                )
                .expect("`ArrayType` should cast"),
            )),
            SyntaxKind::AsKeyword => Some(Self::AsKeyword(
                AsKeyword::cast(
                    syntax
                        .into_token()
                        .expect("`AsKeyword` should always cast to a token"),
                )
                .expect("`AsKeyword` should cast"),
            )),
            SyntaxKind::Assignment => Some(Self::Assignment(
                Assignment::cast(
                    syntax
                        .into_token()
                        .expect("`Assignment` should always cast to a token"),
                )
                .expect("`Assignment` should cast"),
            )),
            SyntaxKind::Asterisk => Some(Self::Asterisk(
                Asterisk::cast(
                    syntax
                        .into_token()
                        .expect("`Asterisk` should always cast to a token"),
                )
                .expect("`Asterisk` should cast"),
            )),
            SyntaxKind::BooleanTypeKeyword => Some(Self::BooleanTypeKeyword(
                BooleanTypeKeyword::cast(
                    syntax
                        .into_token()
                        .expect("`BooleanTypeKeyword` should always cast to a token"),
                )
                .expect("`BooleanTypeKeyword` should cast"),
            )),
            SyntaxKind::BoundDeclNode => Some(Self::BoundDecl(
                BoundDecl::cast(
                    syntax
                        .into_node()
                        .expect("`BoundDeclNode` should always cast to a node"),
                )
                .expect("`BoundDecl` should cast"),
            )),
            SyntaxKind::CallAfterNode => Some(Self::CallAfter(
                CallAfter::cast(
                    syntax
                        .into_node()
                        .expect("`CallAfterNode` should always cast to a node"),
                )
                .expect("`CallAfter` should cast"),
            )),
            SyntaxKind::CallAliasNode => Some(Self::CallAlias(
                CallAlias::cast(
                    syntax
                        .into_node()
                        .expect("`CallAliasNode` should always cast to a node"),
                )
                .expect("`CallAlias` should cast"),
            )),
            SyntaxKind::CallExprNode => Some(Self::CallExpr(
                CallExpr::cast(
                    syntax
                        .into_node()
                        .expect("`CallExprNode` should always cast to a node"),
                )
                .expect("`CallExpr` should cast"),
            )),
            SyntaxKind::CallInputItemNode => Some(Self::CallInputItem(
                CallInputItem::cast(
                    syntax
                        .into_node()
                        .expect("`CallInputItemNode` should always cast to a node"),
                )
                .expect("`CallInputItem` should cast"),
            )),
            SyntaxKind::CallKeyword => Some(Self::CallKeyword(
                CallKeyword::cast(
                    syntax
                        .into_token()
                        .expect("`CallKeyword` should always cast to a token"),
                )
                .expect("`CallKeyword` should cast"),
            )),
            SyntaxKind::CallStatementNode => Some(Self::CallStatement(
                CallStatement::cast(
                    syntax
                        .into_node()
                        .expect("`CallStatementNode` should always cast to a node"),
                )
                .expect("`CallStatement` should cast"),
            )),
            SyntaxKind::CallTargetNode => Some(Self::CallTarget(
                CallTarget::cast(
                    syntax
                        .into_node()
                        .expect("`CallTargetNode` should always cast to a node"),
                )
                .expect("`CallTarget` should cast"),
            )),
            SyntaxKind::CloseBrace => Some(Self::CloseBrace(
                CloseBrace::cast(
                    syntax
                        .into_token()
                        .expect("`CloseBrace` should always cast to a token"),
                )
                .expect("`CloseBrace` should cast"),
            )),
            SyntaxKind::CloseBracket => Some(Self::CloseBracket(
                CloseBracket::cast(
                    syntax
                        .into_token()
                        .expect("`CloseBracket` should always cast to a token"),
                )
                .expect("`CloseBracket` should cast"),
            )),
            SyntaxKind::CloseHeredoc => Some(Self::CloseHeredoc(
                CloseHeredoc::cast(
                    syntax
                        .into_token()
                        .expect("`CloseHeredoc` should always cast to a token"),
                )
                .expect("`CloseHeredoc` should cast"),
            )),
            SyntaxKind::CloseParen => Some(Self::CloseParen(
                CloseParen::cast(
                    syntax
                        .into_token()
                        .expect("`CloseParen` should always cast to a token"),
                )
                .expect("`CloseParen` should cast"),
            )),
            SyntaxKind::Colon => Some(Self::Colon(
                Colon::cast(
                    syntax
                        .into_token()
                        .expect("`Colon` should always cast to a token"),
                )
                .expect("`Colon` should cast"),
            )),
            SyntaxKind::Comma => Some(Self::Comma(
                Comma::cast(
                    syntax
                        .into_token()
                        .expect("`Comma` should always cast to a token"),
                )
                .expect("`Comma` should cast"),
            )),
            SyntaxKind::CommandKeyword => Some(Self::CommandKeyword(
                CommandKeyword::cast(
                    syntax
                        .into_token()
                        .expect("`CommandKeyword` should always cast to a token"),
                )
                .expect("`CommandKeyword` should cast"),
            )),
            SyntaxKind::CommandSectionNode => Some(Self::CommandSection(
                CommandSection::cast(
                    syntax
                        .into_node()
                        .expect("`CommandSectionNode` should always cast to a node"),
                )
                .expect("`CommandSection` should cast"),
            )),
            SyntaxKind::Comment => Some(Self::Comment(
                Comment::cast(
                    syntax
                        .into_token()
                        .expect("`Comment` should always cast to a token"),
                )
                .expect("`Comment` should cast"),
            )),
            SyntaxKind::ConditionalStatementNode => Some(Self::ConditionalStatement(
                ConditionalStatement::cast(
                    syntax
                        .into_node()
                        .expect("`ConditionalStatementNode` should always cast to a node"),
                )
                .expect("`ConditionalStatement` should cast"),
            )),
            SyntaxKind::DirectoryTypeKeyword => Some(Self::DirectoryTypeKeyword(
                DirectoryTypeKeyword::cast(
                    syntax
                        .into_token()
                        .expect("`DirectoryTypeKeyword` should always cast to a token"),
                )
                .expect("`DirectoryTypeKeyword` should cast"),
            )),
            SyntaxKind::DivisionExprNode => Some(Self::DivisionExpr(
                DivisionExpr::cast(
                    syntax
                        .into_node()
                        .expect("`DivisionExprNode` should always cast to a node"),
                )
                .expect("`DivisionExpr` should cast"),
            )),
            SyntaxKind::Dot => Some(Self::Dot(
                Dot::cast(
                    syntax
                        .into_token()
                        .expect("`Dot` should always cast to a token"),
                )
                .expect("`Dot` should cast"),
            )),
            SyntaxKind::DoubleQuote => Some(Self::DoubleQuote(
                DoubleQuote::cast(
                    syntax
                        .into_token()
                        .expect("`DoubleQuote` should always cast to a token"),
                )
                .expect("`DoubleQuote` should cast"),
            )),
            SyntaxKind::ElseKeyword => Some(Self::ElseKeyword(
                ElseKeyword::cast(
                    syntax
                        .into_token()
                        .expect("`ElseKeyword` should always cast to a token"),
                )
                .expect("`ElseKeyword` should cast"),
            )),
            SyntaxKind::Equal => Some(Self::Equal(
                Equal::cast(
                    syntax
                        .into_token()
                        .expect("`Equal` should always cast to a token"),
                )
                .expect("`Equal` should cast"),
            )),
            SyntaxKind::EqualityExprNode => Some(Self::EqualityExpr(
                EqualityExpr::cast(
                    syntax
                        .into_node()
                        .expect("`EqualityExprNode` should always cast to a node"),
                )
                .expect("`EqualityExpr` should cast"),
            )),
            SyntaxKind::Exclamation => Some(Self::Exclamation(
                Exclamation::cast(
                    syntax
                        .into_token()
                        .expect("`Exclamation` should always cast to a token"),
                )
                .expect("`Exclamation` should cast"),
            )),
            SyntaxKind::Exponentiation => Some(Self::Exponentiation(
                Exponentiation::cast(
                    syntax
                        .into_token()
                        .expect("`Exponentiation` should always cast to a token"),
                )
                .expect("`Exponentiation` should cast"),
            )),
            SyntaxKind::ExponentiationExprNode => Some(Self::ExponentiationExpr(
                ExponentiationExpr::cast(
                    syntax
                        .into_node()
                        .expect("`ExponentiationExprNode` should always cast to a node"),
                )
                .expect("`ExponentiationExpr` should cast"),
            )),
            SyntaxKind::FalseKeyword => Some(Self::FalseKeyword(
                FalseKeyword::cast(
                    syntax
                        .into_token()
                        .expect("`FalseKeyword` should always cast to a token"),
                )
                .expect("`FalseKeyword` should cast"),
            )),
            SyntaxKind::FileTypeKeyword => Some(Self::FileTypeKeyword(
                FileTypeKeyword::cast(
                    syntax
                        .into_token()
                        .expect("`FileTypeKeyword` should always cast to a token"),
                )
                .expect("`FileTypeKeyword` should cast"),
            )),
            SyntaxKind::Float => Some(Self::Float(
                Float::cast(
                    syntax
                        .into_token()
                        .expect("`Float` should always cast to a token"),
                )
                .expect("`Float` should cast"),
            )),
            SyntaxKind::FloatTypeKeyword => Some(Self::FloatTypeKeyword(
                FloatTypeKeyword::cast(
                    syntax
                        .into_token()
                        .expect("`FloatTypeKeyword` should always cast to a token"),
                )
                .expect("`FloatTypeKeyword` should cast"),
            )),
            SyntaxKind::Greater => Some(Self::Greater(
                Greater::cast(
                    syntax
                        .into_token()
                        .expect("`Greater` should always cast to a token"),
                )
                .expect("`Greater` should cast"),
            )),
            SyntaxKind::GreaterEqual => Some(Self::GreaterEqual(
                GreaterEqual::cast(
                    syntax
                        .into_token()
                        .expect("`GreaterEqual` should always cast to a token"),
                )
                .expect("`GreaterEqual` should cast"),
            )),
            SyntaxKind::GreaterEqualExprNode => Some(Self::GreaterEqualExpr(
                GreaterEqualExpr::cast(
                    syntax
                        .into_node()
                        .expect("`GreaterEqualExprNode` should always cast to a node"),
                )
                .expect("`GreaterEqualExpr` should cast"),
            )),
            SyntaxKind::GreaterExprNode => Some(Self::GreaterExpr(
                GreaterExpr::cast(
                    syntax
                        .into_node()
                        .expect("`GreaterExprNode` should always cast to a node"),
                )
                .expect("`GreaterExpr` should cast"),
            )),
            SyntaxKind::HintsItemNode => Some(Self::HintsItem(
                HintsItem::cast(
                    syntax
                        .into_node()
                        .expect("`HintsItemNode` should always cast to a node"),
                )
                .expect("`HintsItem` should cast"),
            )),
            SyntaxKind::HintsKeyword => Some(Self::HintsKeyword(
                HintsKeyword::cast(
                    syntax
                        .into_token()
                        .expect("`HintsKeyword` should always cast to a token"),
                )
                .expect("`HintsKeyword` should cast"),
            )),
            SyntaxKind::HintsSectionNode => Some(Self::HintsSection(
                HintsSection::cast(
                    syntax
                        .into_node()
                        .expect("`HintsSectionNode` should always cast to a node"),
                )
                .expect("`HintsSection` should cast"),
            )),
            SyntaxKind::Ident => Some(Self::Ident(
                Ident::cast(
                    syntax
                        .into_token()
                        .expect("`Ident` should always cast to a token"),
                )
                .expect("`Ident` should cast"),
            )),
            SyntaxKind::IfExprNode => Some(Self::IfExpr(
                IfExpr::cast(
                    syntax
                        .into_node()
                        .expect("`IfExprNode` should always cast to a node"),
                )
                .expect("`IfExpr` should cast"),
            )),
            SyntaxKind::IfKeyword => Some(Self::IfKeyword(
                IfKeyword::cast(
                    syntax
                        .into_token()
                        .expect("`IfKeyword` should always cast to a token"),
                )
                .expect("`IfKeyword` should cast"),
            )),
            SyntaxKind::ImportAliasNode => Some(Self::ImportAlias(
                ImportAlias::cast(
                    syntax
                        .into_node()
                        .expect("`ImportAliasNode` should always cast to a node"),
                )
                .expect("`ImportAlias` should cast"),
            )),
            SyntaxKind::ImportKeyword => Some(Self::ImportKeyword(
                ImportKeyword::cast(
                    syntax
                        .into_token()
                        .expect("`ImportKeyword` should always cast to a token"),
                )
                .expect("`ImportKeyword` should cast"),
            )),
            SyntaxKind::ImportStatementNode => Some(Self::ImportStatement(
                ImportStatement::cast(
                    syntax
                        .into_node()
                        .expect("`ImportStatementNode` should always cast to a node"),
                )
                .expect("`ImportStatement` should cast"),
            )),
            SyntaxKind::IndexExprNode => Some(Self::IndexExpr(
                IndexExpr::cast(
                    syntax
                        .into_node()
                        .expect("`IndexExprNode` should always cast to a node"),
                )
                .expect("`IndexExpr` should cast"),
            )),
            SyntaxKind::InequalityExprNode => Some(Self::InequalityExpr(
                InequalityExpr::cast(
                    syntax
                        .into_node()
                        .expect("`InequalityExprNode` should always cast to a node"),
                )
                .expect("`InequalityExpr` should cast"),
            )),
            SyntaxKind::InKeyword => Some(Self::InKeyword(
                InKeyword::cast(
                    syntax
                        .into_token()
                        .expect("`InKeyword` should always cast to a token"),
                )
                .expect("`InKeyword` should cast"),
            )),
            SyntaxKind::InputKeyword => Some(Self::InputKeyword(
                InputKeyword::cast(
                    syntax
                        .into_token()
                        .expect("`InputKeyword` should always cast to a token"),
                )
                .expect("`InputKeyword` should cast"),
            )),
            SyntaxKind::InputSectionNode => Some(Self::InputSection(
                InputSection::cast(
                    syntax
                        .into_node()
                        .expect("`InputSectionNode` should always cast to a node"),
                )
                .expect("`InputSection` should cast"),
            )),
            SyntaxKind::Integer => Some(Self::Integer(
                Integer::cast(
                    syntax
                        .into_token()
                        .expect("`Integer` should always cast to a token"),
                )
                .expect("`Integer` should cast"),
            )),
            SyntaxKind::IntTypeKeyword => Some(Self::IntTypeKeyword(
                IntTypeKeyword::cast(
                    syntax
                        .into_token()
                        .expect("`IntTypeKeyword` should always cast to a token"),
                )
                .expect("`IntTypeKeyword` should cast"),
            )),
            SyntaxKind::Less => Some(Self::Less(
                Less::cast(
                    syntax
                        .into_token()
                        .expect("`Less` should always cast to a token"),
                )
                .expect("`Less` should cast"),
            )),
            SyntaxKind::LessEqual => Some(Self::LessEqual(
                LessEqual::cast(
                    syntax
                        .into_token()
                        .expect("`LessEqual` should always cast to a token"),
                )
                .expect("`LessEqual` should cast"),
            )),
            SyntaxKind::LessEqualExprNode => Some(Self::LessEqualExpr(
                LessEqualExpr::cast(
                    syntax
                        .into_node()
                        .expect("`LessEqualExprNode` should always cast to a node"),
                )
                .expect("`LessEqualExpr` should cast"),
            )),
            SyntaxKind::LessExprNode => Some(Self::LessExpr(
                LessExpr::cast(
                    syntax
                        .into_node()
                        .expect("`LessExprNode` should always cast to a node"),
                )
                .expect("`LessExpr` should cast"),
            )),
            SyntaxKind::LiteralArrayNode => Some(Self::LiteralArray(
                LiteralArray::cast(
                    syntax
                        .into_node()
                        .expect("`LiteralArrayNode` should always cast to a node"),
                )
                .expect("`LiteralArray` should cast"),
            )),
            SyntaxKind::LiteralBooleanNode => Some(Self::LiteralBoolean(
                LiteralBoolean::cast(
                    syntax
                        .into_node()
                        .expect("`LiteralBooleanNode` should always cast to a node"),
                )
                .expect("`LiteralBoolean` should cast"),
            )),
            SyntaxKind::LiteralCommandText => Some(Self::CommandText(
                CommandText::cast(
                    syntax
                        .into_token()
                        .expect("`LiteralCommandText` should always cast to a token"),
                )
                .expect("`CommandText` should cast"),
            )),
            SyntaxKind::LiteralFloatNode => Some(Self::LiteralFloat(
                LiteralFloat::cast(
                    syntax
                        .into_node()
                        .expect("`LiteralFloatNode` should always cast to a node"),
                )
                .expect("`LiteralFloat` should cast"),
            )),
            SyntaxKind::LiteralHintsItemNode => Some(Self::LiteralHintsItem(
                LiteralHintsItem::cast(
                    syntax
                        .into_node()
                        .expect("`LiteralHintsItemNode` should always cast to a node"),
                )
                .expect("`LiteralHintsItem` should cast"),
            )),
            SyntaxKind::LiteralHintsNode => Some(Self::LiteralHints(
                LiteralHints::cast(
                    syntax
                        .into_node()
                        .expect("`LiteralHintsNode` should always cast to a node"),
                )
                .expect("`LiteralHints` should cast"),
            )),
            SyntaxKind::LiteralInputItemNode => Some(Self::LiteralInputItem(
                LiteralInputItem::cast(
                    syntax
                        .into_node()
                        .expect("`LiteralInputItemNode` should always cast to a node"),
                )
                .expect("`LiteralInputItem` should cast"),
            )),
            SyntaxKind::LiteralInputNode => Some(Self::LiteralInput(
                LiteralInput::cast(
                    syntax
                        .into_node()
                        .expect("`LiteralInputNode` should always cast to a node"),
                )
                .expect("`LiteralInput` should cast"),
            )),
            SyntaxKind::LiteralIntegerNode => Some(Self::LiteralInteger(
                LiteralInteger::cast(
                    syntax
                        .into_node()
                        .expect("`LiteralIntegerNode` should always cast to a node"),
                )
                .expect("`LiteralInteger` should cast"),
            )),
            SyntaxKind::LiteralMapItemNode => Some(Self::LiteralMapItem(
                LiteralMapItem::cast(
                    syntax
                        .into_node()
                        .expect("`LiteralMapItemNode` should always cast to a node"),
                )
                .expect("`LiteralMapItem` should cast"),
            )),
            SyntaxKind::LiteralMapNode => Some(Self::LiteralMap(
                LiteralMap::cast(
                    syntax
                        .into_node()
                        .expect("`LiteralMapNode` should always cast to a node"),
                )
                .expect("`LiteralMap` should cast"),
            )),
            SyntaxKind::LiteralNoneNode => Some(Self::LiteralNone(
                LiteralNone::cast(
                    syntax
                        .into_node()
                        .expect("`LiteralNoneNode` should always cast to a node"),
                )
                .expect("`LiteralNone` should cast"),
            )),
            SyntaxKind::LiteralNullNode => Some(Self::LiteralNull(
                LiteralNull::cast(
                    syntax
                        .into_node()
                        .expect("`LiteralNullNode` should always cast to a node"),
                )
                .expect("`LiteralNull` should cast"),
            )),
            SyntaxKind::LiteralObjectItemNode => Some(Self::LiteralObjectItem(
                LiteralObjectItem::cast(
                    syntax
                        .into_node()
                        .expect("`LiteralObjectItemNode` should always cast to a node"),
                )
                .expect("`LiteralObjectItem` should cast"),
            )),
            SyntaxKind::LiteralObjectNode => Some(Self::LiteralObject(
                LiteralObject::cast(
                    syntax
                        .into_node()
                        .expect("`LiteralObjectNode` should always cast to a node"),
                )
                .expect("`LiteralObject` should cast"),
            )),
            SyntaxKind::LiteralOutputItemNode => Some(Self::LiteralOutputItem(
                LiteralOutputItem::cast(
                    syntax
                        .into_node()
                        .expect("`LiteralOutputItemNode` should always cast to a node"),
                )
                .expect("`LiteralOutputItem` should cast"),
            )),
            SyntaxKind::LiteralOutputNode => Some(Self::LiteralOutput(
                LiteralOutput::cast(
                    syntax
                        .into_node()
                        .expect("`LiteralOutputNode` should always cast to a node"),
                )
                .expect("`LiteralOutput` should cast"),
            )),
            SyntaxKind::LiteralPairNode => Some(Self::LiteralPair(
                LiteralPair::cast(
                    syntax
                        .into_node()
                        .expect("`LiteralPairNode` should always cast to a node"),
                )
                .expect("`LiteralPair` should cast"),
            )),
            SyntaxKind::LiteralStringNode => Some(Self::LiteralString(
                LiteralString::cast(
                    syntax
                        .into_node()
                        .expect("`LiteralStringNode` should always cast to a node"),
                )
                .expect("`LiteralString` should cast"),
            )),
            SyntaxKind::LiteralStringText => Some(Self::LiteralString(
                LiteralString::cast(
                    syntax
                        .into_node()
                        .expect("`LiteralStringText` should always cast to a node"),
                )
                .expect("`LiteralString` should cast"),
            )),
            SyntaxKind::LiteralStructItemNode => Some(Self::LiteralStructItem(
                LiteralStructItem::cast(
                    syntax
                        .into_node()
                        .expect("`LiteralStructItemNode` should always cast to a node"),
                )
                .expect("`LiteralStructItem` should cast"),
            )),
            SyntaxKind::LiteralStructNode => Some(Self::LiteralStruct(
                LiteralStruct::cast(
                    syntax
                        .into_node()
                        .expect("`LiteralStructNode` should always cast to a node"),
                )
                .expect("`LiteralStruct` should cast"),
            )),
            SyntaxKind::LogicalAnd => Some(Self::LogicalAnd(
                LogicalAnd::cast(
                    syntax
                        .into_token()
                        .expect("`LogicalAnd` should always cast to a token"),
                )
                .expect("`LogicalAnd` should cast"),
            )),
            SyntaxKind::LogicalAndExprNode => Some(Self::LogicalAndExpr(
                LogicalAndExpr::cast(
                    syntax
                        .into_node()
                        .expect("`LogicalAndExprNode` should always cast to a node"),
                )
                .expect("`LogicalAndExpr` should cast"),
            )),
            SyntaxKind::LogicalNotExprNode => Some(Self::LogicalNotExpr(
                LogicalNotExpr::cast(
                    syntax
                        .into_node()
                        .expect("`LogicalNotExprNode` should always cast to a node"),
                )
                .expect("`LogicalNotExpr` should cast"),
            )),
            SyntaxKind::LogicalOr => Some(Self::LogicalOr(
                LogicalOr::cast(
                    syntax
                        .into_token()
                        .expect("`LogicalOr` should always cast to a token"),
                )
                .expect("`LogicalOr` should cast"),
            )),
            SyntaxKind::LogicalOrExprNode => Some(Self::LogicalOrExpr(
                LogicalOrExpr::cast(
                    syntax
                        .into_node()
                        .expect("`LogicalOrExprNode` should always cast to a node"),
                )
                .expect("`LogicalOrExpr` should cast"),
            )),
            SyntaxKind::MapTypeKeyword => Some(Self::MapTypeKeyword(
                MapTypeKeyword::cast(
                    syntax
                        .into_token()
                        .expect("`MapTypeKeyword` should always cast to a token"),
                )
                .expect("`MapTypeKeyword` should cast"),
            )),
            SyntaxKind::MapTypeNode => Some(Self::MapType(
                MapType::cast(
                    syntax
                        .into_node()
                        .expect("`MapTypeNode` should always cast to a node"),
                )
                .expect("`MapType` should cast"),
            )),
            SyntaxKind::MetadataArrayNode => Some(Self::MetadataArray(
                MetadataArray::cast(
                    syntax
                        .into_node()
                        .expect("`MetadataArrayNode` should always cast to a node"),
                )
                .expect("`MetadataArray` should cast"),
            )),
            SyntaxKind::MetadataObjectItemNode => Some(Self::MetadataObjectItem(
                MetadataObjectItem::cast(
                    syntax
                        .into_node()
                        .expect("`MetadataObjectItemNode` should always cast to a node"),
                )
                .expect("`MetadataObjectItem` should cast"),
            )),
            SyntaxKind::MetadataObjectNode => Some(Self::MetadataObject(
                MetadataObject::cast(
                    syntax
                        .into_node()
                        .expect("`MetadataObjectNode` should always cast to a node"),
                )
                .expect("`MetadataObject` should cast"),
            )),
            SyntaxKind::MetadataSectionNode => Some(Self::MetadataSection(
                MetadataSection::cast(
                    syntax
                        .into_node()
                        .expect("`MetadataSectionNode` should always cast to a node"),
                )
                .expect("`MetadataSection` should cast"),
            )),
            SyntaxKind::MetaKeyword => Some(Self::MetaKeyword(
                MetaKeyword::cast(
                    syntax
                        .into_token()
                        .expect("`MetaKeyword` should always cast to a token"),
                )
                .expect("`MetaKeyword` should cast"),
            )),
            SyntaxKind::Minus => Some(Self::Minus(
                Minus::cast(
                    syntax
                        .into_token()
                        .expect("`Minus` should always cast to a token"),
                )
                .expect("`Minus` should cast"),
            )),
            SyntaxKind::ModuloExprNode => Some(Self::ModuloExpr(
                ModuloExpr::cast(
                    syntax
                        .into_node()
                        .expect("`ModuloExprNode` should always cast to a node"),
                )
                .expect("`ModuloExpr` should cast"),
            )),
            SyntaxKind::MultiplicationExprNode => Some(Self::MultiplicationExpr(
                MultiplicationExpr::cast(
                    syntax
                        .into_node()
                        .expect("`MultiplicationExprNode` should always cast to a node"),
                )
                .expect("`MultiplicationExpr` should cast"),
            )),
            SyntaxKind::NameRefNode => Some(Self::NameRef(
                NameRef::cast(
                    syntax
                        .into_node()
                        .expect("`NameRefNode` should always cast to a node"),
                )
                .expect("`NameRef` should cast"),
            )),
            SyntaxKind::NegationExprNode => Some(Self::NegationExpr(
                NegationExpr::cast(
                    syntax
                        .into_node()
                        .expect("`NegationExprNode` should always cast to a node"),
                )
                .expect("`NegationExpr` should cast"),
            )),
            SyntaxKind::NoneKeyword => Some(Self::NoneKeyword(
                NoneKeyword::cast(
                    syntax
                        .into_token()
                        .expect("`NoneKeyword` should always cast to a token"),
                )
                .expect("`NoneKeyword` should cast"),
            )),
            SyntaxKind::NotEqual => Some(Self::NotEqual(
                NotEqual::cast(
                    syntax
                        .into_token()
                        .expect("`NotEqual` should always cast to a token"),
                )
                .expect("`NotEqual` should cast"),
            )),
            SyntaxKind::NullKeyword => Some(Self::NullKeyword(
                NullKeyword::cast(
                    syntax
                        .into_token()
                        .expect("`NullKeyword` should always cast to a token"),
                )
                .expect("`NullKeyword` should cast"),
            )),
            SyntaxKind::ObjectKeyword => Some(Self::ObjectKeyword(
                ObjectKeyword::cast(
                    syntax
                        .into_token()
                        .expect("`ObjectKeyword` should always cast to a token"),
                )
                .expect("`ObjectKeyword` should cast"),
            )),
            SyntaxKind::ObjectTypeKeyword => Some(Self::ObjectTypeKeyword(
                ObjectTypeKeyword::cast(
                    syntax
                        .into_token()
                        .expect("`ObjectTypeKeyword` should always cast to a token"),
                )
                .expect("`ObjectTypeKeyword` should cast"),
            )),
            SyntaxKind::ObjectTypeNode => Some(Self::ObjectType(
                ObjectType::cast(
                    syntax
                        .into_node()
                        .expect("`ObjectTypeNode` should always cast to a node"),
                )
                .expect("`ObjectType` should cast"),
            )),
            SyntaxKind::OpenBrace => Some(Self::OpenBrace(
                OpenBrace::cast(
                    syntax
                        .into_token()
                        .expect("`OpenBrace` should always cast to a token"),
                )
                .expect("`OpenBrace` should cast"),
            )),
            SyntaxKind::OpenBracket => Some(Self::OpenBracket(
                OpenBracket::cast(
                    syntax
                        .into_token()
                        .expect("`OpenBracket` should always cast to a token"),
                )
                .expect("`OpenBracket` should cast"),
            )),
            SyntaxKind::OpenHeredoc => Some(Self::OpenHeredoc(
                OpenHeredoc::cast(
                    syntax
                        .into_token()
                        .expect("`OpenHeredoc` should always cast to a token"),
                )
                .expect("`OpenHeredoc` should cast"),
            )),
            SyntaxKind::OpenParen => Some(Self::OpenParen(
                OpenParen::cast(
                    syntax
                        .into_token()
                        .expect("`OpenParen` should always cast to a token"),
                )
                .expect("`OpenParen` should cast"),
            )),
            SyntaxKind::OutputKeyword => Some(Self::OutputKeyword(
                OutputKeyword::cast(
                    syntax
                        .into_token()
                        .expect("`OutputKeyword` should always cast to a token"),
                )
                .expect("`OutputKeyword` should cast"),
            )),
            SyntaxKind::OutputSectionNode => Some(Self::OutputSection(
                OutputSection::cast(
                    syntax
                        .into_node()
                        .expect("`OutputSectionNode` should always cast to a node"),
                )
                .expect("`OutputSection` should cast"),
            )),
            SyntaxKind::PairTypeKeyword => Some(Self::PairTypeKeyword(
                PairTypeKeyword::cast(
                    syntax
                        .into_token()
                        .expect("`PairTypeKeyword` should always cast to a token"),
                )
                .expect("`PairTypeKeyword` should cast"),
            )),
            SyntaxKind::PairTypeNode => Some(Self::PairType(
                PairType::cast(
                    syntax
                        .into_node()
                        .expect("`PairTypeNode` should always cast to a node"),
                )
                .expect("`PairType` should cast"),
            )),
            SyntaxKind::ParameterMetadataSectionNode => Some(Self::ParameterMetadataSection(
                ParameterMetadataSection::cast(
                    syntax
                        .into_node()
                        .expect("`ParameterMetadataSectionNode` should always cast to a node"),
                )
                .expect("`ParameterMetadataSection` should cast"),
            )),
            SyntaxKind::ParameterMetaKeyword => Some(Self::ParameterMetaKeyword(
                ParameterMetaKeyword::cast(
                    syntax
                        .into_token()
                        .expect("`ParameterMetaKeyword` should always cast to a token"),
                )
                .expect("`ParameterMetaKeyword` should cast"),
            )),
            SyntaxKind::ParenthesizedExprNode => Some(Self::ParenthesizedExpr(
                ParenthesizedExpr::cast(
                    syntax
                        .into_node()
                        .expect("`ParenthesizedExprNode` should always cast to a node"),
                )
                .expect("`ParenthesizedExpr` should cast"),
            )),
            SyntaxKind::Percent => Some(Self::Percent(
                Percent::cast(
                    syntax
                        .into_token()
                        .expect("`Percent` should always cast to a token"),
                )
                .expect("`Percent` should cast"),
            )),
            SyntaxKind::PlaceholderDefaultOptionNode => Some(Self::DefaultOption(
                DefaultOption::cast(
                    syntax
                        .into_node()
                        .expect("`PlaceholderDefaultOptionNode` should always cast to a node"),
                )
                .expect("`DefaultOption` should cast"),
            )),
            SyntaxKind::PlaceholderNode => Some(Self::Placeholder(
                Placeholder::cast(
                    syntax
                        .into_node()
                        .expect("`PlaceholderNode` should always cast to a node"),
                )
                .expect("`Placeholder` should cast"),
            )),
            SyntaxKind::PlaceholderOpen => Some(Self::PlaceholderOpen(
                PlaceholderOpen::cast(
                    syntax
                        .into_token()
                        .expect("`PlaceholderOpen` should always cast to a token"),
                )
                .expect("`PlaceholderOpen` should cast"),
            )),
            SyntaxKind::PlaceholderSepOptionNode => Some(Self::SepOption(
                SepOption::cast(
                    syntax
                        .into_node()
                        .expect("`PlaceholderSepOptionNode` should always cast to a node"),
                )
                .expect("`SepOption` should cast"),
            )),
            SyntaxKind::PlaceholderTrueFalseOptionNode => Some(Self::TrueFalseOption(
                TrueFalseOption::cast(
                    syntax
                        .into_node()
                        .expect("`PlaceholderTrueFalseOptionNode` should always cast to a node"),
                )
                .expect("`TrueFalseOption` should cast"),
            )),
            SyntaxKind::Plus => Some(Self::Plus(
                Plus::cast(
                    syntax
                        .into_token()
                        .expect("`Plus` should always cast to a token"),
                )
                .expect("`Plus` should cast"),
            )),
            SyntaxKind::PrimitiveTypeNode => Some(Self::PrimitiveType(
                PrimitiveType::cast(
                    syntax
                        .into_node()
                        .expect("`PrimitiveTypeNode` should always cast to a node"),
                )
                .expect("`PrimitiveType` should cast"),
            )),
            SyntaxKind::QuestionMark => Some(Self::QuestionMark(
                QuestionMark::cast(
                    syntax
                        .into_token()
                        .expect("`QuestionMark` should always cast to a token"),
                )
                .expect("`QuestionMark` should cast"),
            )),
            SyntaxKind::RequirementsItemNode => Some(Self::RequirementsItem(
                RequirementsItem::cast(
                    syntax
                        .into_node()
                        .expect("`RequirementsItemNode` should always cast to a node"),
                )
                .expect("`RequirementsItem` should cast"),
            )),
            SyntaxKind::RequirementsKeyword => Some(Self::RequirementsKeyword(
                RequirementsKeyword::cast(
                    syntax
                        .into_token()
                        .expect("`RequirementsKeyword` should always cast to a token"),
                )
                .expect("`RequirementsKeyword` should cast"),
            )),
            SyntaxKind::RequirementsSectionNode => Some(Self::RequirementsSection(
                RequirementsSection::cast(
                    syntax
                        .into_node()
                        .expect("`RequirementsSectionNode` should always cast to a node"),
                )
                .expect("`RequirementsSection` should cast"),
            )),
            SyntaxKind::RootNode => Some(Self::Ast(
                Ast::cast(
                    syntax
                        .into_node()
                        .expect("`RootNode` should always cast to a node"),
                )
                .expect("`Ast` should cast"),
            )),
            SyntaxKind::RuntimeItemNode => Some(Self::RuntimeItem(
                RuntimeItem::cast(
                    syntax
                        .into_node()
                        .expect("`RuntimeItemNode` should always cast to a node"),
                )
                .expect("`RuntimeItem` should cast"),
            )),
            SyntaxKind::RuntimeKeyword => Some(Self::RuntimeKeyword(
                RuntimeKeyword::cast(
                    syntax
                        .into_token()
                        .expect("`RuntimeKeyword` should always cast to a token"),
                )
                .expect("`RuntimeKeyword` should cast"),
            )),
            SyntaxKind::RuntimeSectionNode => Some(Self::RuntimeSection(
                RuntimeSection::cast(
                    syntax
                        .into_node()
                        .expect("`RuntimeSectionNode` should always cast to a node"),
                )
                .expect("`RuntimeSection` should cast"),
            )),
            SyntaxKind::ScatterKeyword => Some(Self::ScatterKeyword(
                ScatterKeyword::cast(
                    syntax
                        .into_token()
                        .expect("`ScatterKeyword` should always cast to a token"),
                )
                .expect("`ScatterKeyword` should cast"),
            )),
            SyntaxKind::ScatterStatementNode => Some(Self::ScatterStatement(
                ScatterStatement::cast(
                    syntax
                        .into_node()
                        .expect("`ScatterStatementNode` should always cast to a node"),
                )
                .expect("`ScatterStatement` should cast"),
            )),
            SyntaxKind::SingleQuote => Some(Self::SingleQuote(
                SingleQuote::cast(
                    syntax
                        .into_token()
                        .expect("`SingleQuote` should always cast to a token"),
                )
                .expect("`SingleQuote` should cast"),
            )),
            SyntaxKind::Slash => Some(Self::Slash(
                Slash::cast(
                    syntax
                        .into_token()
                        .expect("`Slash` should always cast to a token"),
                )
                .expect("`Slash` should cast"),
            )),
            SyntaxKind::StringTypeKeyword => Some(Self::StringTypeKeyword(
                StringTypeKeyword::cast(
                    syntax
                        .into_token()
                        .expect("`StringTypeKeyword` should always cast to a token"),
                )
                .expect("`StringTypeKeyword` should cast"),
            )),
            SyntaxKind::StructDefinitionNode => Some(Self::StructDefinition(
                StructDefinition::cast(
                    syntax
                        .into_node()
                        .expect("`StructDefinitionNode` should always cast to a node"),
                )
                .expect("`StructDefinition` should cast"),
            )),
            SyntaxKind::StructKeyword => Some(Self::StructKeyword(
                StructKeyword::cast(
                    syntax
                        .into_token()
                        .expect("`StructKeyword` should always cast to a token"),
                )
                .expect("`StructKeyword` should cast"),
            )),
            SyntaxKind::SubtractionExprNode => Some(Self::SubtractionExpr(
                SubtractionExpr::cast(
                    syntax
                        .into_node()
                        .expect("`SubtractionExprNode` should always cast to a node"),
                )
                .expect("`SubtractionExpr` should cast"),
            )),
            SyntaxKind::TaskDefinitionNode => Some(Self::TaskDefinition(
                TaskDefinition::cast(
                    syntax
                        .into_node()
                        .expect("`TaskDefinitionNode` should always cast to a node"),
                )
                .expect("`TaskDefinition` should cast"),
            )),
            SyntaxKind::TaskKeyword => Some(Self::TaskKeyword(
                TaskKeyword::cast(
                    syntax
                        .into_token()
                        .expect("`TaskKeyword` should always cast to a token"),
                )
                .expect("`TaskKeyword` should cast"),
            )),
            SyntaxKind::ThenKeyword => Some(Self::ThenKeyword(
                ThenKeyword::cast(
                    syntax
                        .into_token()
                        .expect("`ThenKeyword` should always cast to a token"),
                )
                .expect("`ThenKeyword` should cast"),
            )),
            SyntaxKind::TrueKeyword => Some(Self::TrueKeyword(
                TrueKeyword::cast(
                    syntax
                        .into_token()
                        .expect("`TrueKeyword` should always cast to a token"),
                )
                .expect("`TrueKeyword` should cast"),
            )),
            SyntaxKind::TypeRefNode => Some(Self::TypeRef(
                TypeRef::cast(
                    syntax
                        .into_node()
                        .expect("`TypeRefNode` should always cast to a node"),
                )
                .expect("`TypeRef` should cast"),
            )),
            SyntaxKind::UnboundDeclNode => Some(Self::UnboundDecl(
                UnboundDecl::cast(
                    syntax
                        .into_node()
                        .expect("`UnboundDeclNode` should always cast to a node"),
                )
                .expect("`UnboundDecl` should cast"),
            )),
            SyntaxKind::Version => Some(Self::Version(
                Version::cast(
                    syntax
                        .into_token()
                        .expect("`Version` should always cast to a token"),
                )
                .expect("`Version` should cast"),
            )),
            SyntaxKind::VersionKeyword => Some(Self::VersionKeyword(
                VersionKeyword::cast(
                    syntax
                        .into_token()
                        .expect("`VersionKeyword` should always cast to a token"),
                )
                .expect("`VersionKeyword` should cast"),
            )),
            SyntaxKind::VersionStatementNode => Some(Self::VersionStatement(
                VersionStatement::cast(
                    syntax
                        .into_node()
                        .expect("`VersionStatementNode` should always cast to a node"),
                )
                .expect("`VersionStatement` should cast"),
            )),
            SyntaxKind::Whitespace => Some(Self::Whitespace(
                Whitespace::cast(
                    syntax
                        .into_token()
                        .expect("`Whitespace` should always cast to a token"),
                )
                .expect("`Whitespace` should cast"),
            )),
            SyntaxKind::WorkflowDefinitionNode => Some(Self::WorkflowDefinition(
                WorkflowDefinition::cast(
                    syntax
                        .into_node()
                        .expect("`WorkflowDefinitionNode` should always cast to a node"),
                )
                .expect("`WorkflowDefinition` should cast"),
            )),
            SyntaxKind::WorkflowKeyword => Some(Self::WorkflowKeyword(
                WorkflowKeyword::cast(
                    syntax
                        .into_token()
                        .expect("`WorkflowKeyword` should always cast to a token"),
                )
                .expect("`WorkflowKeyword` should cast"),
            )),
            kind if kind.is_pseudokind() => None,
            _ => unreachable!(),
        }
    }
}
