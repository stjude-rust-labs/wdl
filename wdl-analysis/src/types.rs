//! Representation of the WDL type system.

use std::fmt;

use id_arena::Arena;
use id_arena::ArenaBehavior;
use id_arena::DefaultArenaBehavior;
use id_arena::Id;
use indexmap::IndexMap;
use wdl_ast::v1;
use wdl_ast::AstToken;
use wdl_ast::Diagnostic;
use wdl_ast::Ident;

use crate::STDLIB;

/// Creates an "unknown type" diagnostic.
fn unknown_type(name: &Ident) -> Diagnostic {
    Diagnostic::error(format!("unknown type name `{name}`", name = name.as_str()))
        .with_highlight(name.span())
}

/// A trait implemented on types that may be optional.
pub trait Optional: Copy {
    /// Determines if the type is optional.
    fn is_optional(&self) -> bool;

    /// Makes the type optional if it isn't already optional.
    fn optional(&self) -> Self;

    /// Makes the type required if it isn't already required.
    fn require(&self) -> Self;
}

/// A trait implemented on types that are coercible to other types.
pub trait Coercible {
    /// Determines if the type is coercible to the target type.
    fn is_coercible_to(&self, types: &Types, target: &Self) -> bool;
}

/// A trait implement on types for type equality.
///
/// This is similar to `Eq` except it supports recursive types.
pub trait TypeEq {
    /// Determines if the two types are equal.
    fn type_eq(&self, types: &Types, other: &Self) -> bool;
}

/// Represents a kind of primitive WDL type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveTypeKind {
    /// The type is a `Boolean`.
    Boolean,
    /// The type is an `Int`.
    Integer,
    /// The type is a `Float`.
    Float,
    /// The type is a `String`.
    String,
    /// The type is a `File`.
    File,
    /// The type is a `Directory`.
    Directory,
}

impl Coercible for PrimitiveTypeKind {
    fn is_coercible_to(&self, _: &Types, target: &Self) -> bool {
        if self == target {
            return true;
        }

        match (self, target) {
            // String -> File
            (Self::String, Self::File) |
            // String -> Directory
            (Self::String, Self::Directory) |
            // Int -> Float
            (Self::Integer, Self::Float)
            => true,

            // Not coercible
            _ => false
        }
    }
}

impl From<v1::PrimitiveTypeKind> for PrimitiveTypeKind {
    fn from(value: v1::PrimitiveTypeKind) -> Self {
        match value {
            v1::PrimitiveTypeKind::Boolean => Self::Boolean,
            v1::PrimitiveTypeKind::Integer => Self::Integer,
            v1::PrimitiveTypeKind::Float => Self::Float,
            v1::PrimitiveTypeKind::String => Self::String,
            v1::PrimitiveTypeKind::File => Self::File,
            v1::PrimitiveTypeKind::Directory => Self::Directory,
        }
    }
}

/// Represents a primitive WDL type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PrimitiveType {
    /// The kind of primitive type.
    kind: PrimitiveTypeKind,
    /// Whether or not the primitive type is optional.
    optional: bool,
}

impl PrimitiveType {
    /// Constructs a new primitive type.
    pub fn new(kind: PrimitiveTypeKind) -> Self {
        Self {
            kind,
            optional: false,
        }
    }

    /// Constructs a new optional primitive type.
    pub fn optional(kind: PrimitiveTypeKind) -> Self {
        Self {
            kind,
            optional: true,
        }
    }

    /// Gets the kind of primitive type.
    pub fn kind(&self) -> PrimitiveTypeKind {
        self.kind
    }
}

impl Optional for PrimitiveType {
    fn is_optional(&self) -> bool {
        self.optional
    }

    fn optional(&self) -> Self {
        Self {
            kind: self.kind,
            optional: true,
        }
    }

    fn require(&self) -> Self {
        Self {
            kind: self.kind,
            optional: false,
        }
    }
}

impl Coercible for PrimitiveType {
    fn is_coercible_to(&self, types: &Types, target: &Self) -> bool {
        // An optional type cannot coerce into a required type
        if self.optional && !target.optional {
            return false;
        }

        self.kind.is_coercible_to(types, &target.kind)
    }
}

impl TypeEq for PrimitiveType {
    fn type_eq(&self, _: &Types, other: &Self) -> bool {
        self == other
    }
}

impl fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            PrimitiveTypeKind::Boolean => write!(f, "Boolean")?,
            PrimitiveTypeKind::Integer => write!(f, "Int")?,
            PrimitiveTypeKind::Float => write!(f, "Float")?,
            PrimitiveTypeKind::String => write!(f, "String")?,
            PrimitiveTypeKind::File => write!(f, "File")?,
            PrimitiveTypeKind::Directory => write!(f, "Directory")?,
        }

        if self.optional {
            write!(f, "?")?;
        }

        Ok(())
    }
}

impl From<PrimitiveTypeKind> for PrimitiveType {
    fn from(value: PrimitiveTypeKind) -> Self {
        Self {
            kind: value,
            optional: false,
        }
    }
}

impl From<v1::PrimitiveType> for PrimitiveType {
    fn from(ty: v1::PrimitiveType) -> Self {
        Self {
            kind: ty.kind().into(),
            optional: ty.is_optional(),
        }
    }
}

/// Represents an identifier of a defined compound type.
pub type CompoundTypeDefId = Id<CompoundTypeDef>;

/// Represents a WDL type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Type {
    /// The type is a primitive type.
    Primitive(PrimitiveType),
    /// The type is a compound type.
    Compound(CompoundType),
    /// The type is `Object`.
    Object,
    /// The type is `Object?`.
    OptionalObject,
    /// A special hidden type for a value that may have any one of several
    /// concrete types.
    ///
    /// This variant is also used to convey an "indeterminate" type; an
    /// indeterminate type may result from a previous type error.
    Union,
    /// A special type that behaves like an optional `Union`.
    None,
}

impl Type {
    /// Creates a new type from an V1 AST representation of a type.
    ///
    /// The provided callback is used to look up type name references.
    ///
    /// If a type could not created, an error with the relevant diagnostic is
    /// returned.
    pub fn from_ast_v1<F>(types: &mut Types, ty: v1::Type, lookup: &F) -> Result<Self, Diagnostic>
    where
        F: Fn(&str) -> Option<Type>,
    {
        let optional = ty.is_optional();

        let ty = match ty {
            v1::Type::Map(ty) => {
                let ty = MapType::from_ast_v1(types, ty, lookup)?;
                types.add_map(ty)
            }
            v1::Type::Array(ty) => {
                let ty = ArrayType::from_ast_v1(types, ty, lookup)?;
                types.add_array(ty)
            }
            v1::Type::Pair(ty) => {
                let ty = PairType::from_ast_v1(types, ty, lookup)?;
                types.add_pair(ty)
            }
            v1::Type::Object(_) => Type::Object,
            v1::Type::Ref(r) => {
                let name = r.name();
                lookup(name.as_str()).ok_or_else(|| unknown_type(&name))?
            }
            v1::Type::Primitive(ty) => Self::Primitive(ty.into()),
        };

        if optional { Ok(ty.optional()) } else { Ok(ty) }
    }

    /// Returns an object that implements `Display` for formatting the type.
    pub fn display<'a>(&self, types: &'a Types) -> impl fmt::Display + 'a {
        #[allow(clippy::missing_docs_in_private_items)]
        struct Display<'a> {
            types: &'a Types,
            ty: Type,
        }

        impl fmt::Display for Display<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self.ty {
                    Type::Primitive(ty) => ty.fmt(f),
                    Type::Compound(ty) => ty.display(self.types).fmt(f),
                    Type::Object => write!(f, "Object"),
                    Type::OptionalObject => write!(f, "Object?"),
                    Type::Union => write!(f, "Union"),
                    Type::None => write!(f, "None"),
                }
            }
        }

        Display { types, ty: *self }
    }

    /// Asserts that the type is valid.
    fn assert_valid(&self, types: &Types) {
        match self {
            Self::Compound(ty) => {
                let arena_id = DefaultArenaBehavior::arena_id(ty.definition());
                assert!(
                    arena_id == DefaultArenaBehavior::arena_id(types.0.next_id())
                        || arena_id == DefaultArenaBehavior::arena_id(STDLIB.types().0.next_id()),
                    "type comes from a different arena"
                );
                ty.assert_valid(types);
            }
            Self::Primitive(_) | Self::Object | Self::OptionalObject | Self::Union | Self::None => {
            }
        }
    }
}

impl Optional for Type {
    fn is_optional(&self) -> bool {
        match self {
            Self::Primitive(ty) => ty.is_optional(),
            Self::Compound(ty) => ty.is_optional(),
            Self::OptionalObject | Self::None => true,
            Self::Object | Self::Union => false,
        }
    }

    fn optional(&self) -> Self {
        match self {
            Self::Primitive(ty) => Self::Primitive(ty.optional()),
            Self::Compound(ty) => Self::Compound(ty.optional()),
            Self::Object | Self::OptionalObject => Self::OptionalObject,
            Self::Union | Self::None => Self::None,
        }
    }

    fn require(&self) -> Self {
        match self {
            Self::Primitive(ty) => Self::Primitive(ty.require()),
            Self::Compound(ty) => Self::Compound(ty.require()),
            Self::Object | Self::OptionalObject => Self::Object,
            Self::Union | Self::None => Self::Union,
        }
    }
}

impl Coercible for Type {
    fn is_coercible_to(&self, types: &Types, target: &Self) -> bool {
        if self == target {
            return true;
        }

        match (self, target) {
            (Self::Primitive(src), Self::Primitive(target)) => src.is_coercible_to(types, target),
            (Self::Compound(src), Self::Compound(target)) => src.is_coercible_to(types, target),

            // Object -> Object, Object -> Object?, Object? -> Object?
            (Self::Object, Self::Object)
            | (Self::Object, Self::OptionalObject)
            | (Self::OptionalObject, Self::OptionalObject) => true,

            // Map[String, X] -> Object, Map[String, X] -> Object?, Map[String, X]? -> Object?
            // Struct -> Object, Struct -> Object?, Struct? -> Object?
            (Self::Compound(src), Self::Object) | (Self::Compound(src), Self::OptionalObject) => {
                if src.is_optional() && *target == Self::Object {
                    return false;
                }

                match types.type_definition(src.definition) {
                    CompoundTypeDef::Map(src) => {
                        if src.key_type.kind() != PrimitiveTypeKind::String {
                            return false;
                        }

                        true
                    }
                    CompoundTypeDef::Struct(_) => true,
                    _ => false,
                }
            }

            // Object -> Map[String, X], Object -> Map[String, X]?, Object? -> Map[String, X]? (if
            // all object members are coercible to X)
            // Object -> Struct, Object -> Struct?, Object? -> Struct? (if object keys match struct
            // member names and object values must be coercible to struct member types)
            (Self::Object, Self::Compound(target))
            | (Self::OptionalObject, Self::Compound(target)) => {
                if *self == Self::OptionalObject && !target.is_optional() {
                    return false;
                }

                match types.type_definition(target.definition) {
                    CompoundTypeDef::Map(target) => {
                        if target.key_type.kind() != PrimitiveTypeKind::String {
                            return false;
                        }

                        // Note: checking object members is a runtime value constraint
                        true
                    }
                    CompoundTypeDef::Struct(_) => {
                        // Note: checking object keys and values is a runtime constraint
                        true
                    }
                    _ => false,
                }
            }

            // Union is always coercible to the target
            (Self::Union, _) => true,

            // None is coercible to an optional type
            (Self::None, ty) if ty.is_optional() => true,

            // Not coercible
            _ => false,
        }
    }
}

impl TypeEq for Type {
    fn type_eq(&self, types: &Types, other: &Self) -> bool {
        match (self, other) {
            (Self::Primitive(a), Self::Primitive(b)) => a.type_eq(types, b),
            (Self::Compound(a), Self::Compound(b)) => a.type_eq(types, b),
            (Self::Object, Self::Object) => true,
            (Self::OptionalObject, Self::OptionalObject) => true,
            (Self::Union, Self::Union) => true,
            (Self::None, Self::None) => true,
            _ => false,
        }
    }
}

impl From<PrimitiveTypeKind> for Type {
    fn from(value: PrimitiveTypeKind) -> Self {
        Self::Primitive(PrimitiveType::new(value))
    }
}

impl From<PrimitiveType> for Type {
    fn from(value: PrimitiveType) -> Self {
        Self::Primitive(value)
    }
}

/// Represents a compound type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CompoundType {
    /// The definition identifier for the compound type.
    definition: CompoundTypeDefId,
    /// Whether or not the type is optional.
    optional: bool,
}

impl CompoundType {
    /// Gets the definition identifier of the compound type.
    pub fn definition(&self) -> CompoundTypeDefId {
        self.definition
    }

    /// Returns an object that implements `Display` for formatting the type.
    pub fn display<'a>(&self, types: &'a Types) -> impl fmt::Display + 'a {
        #[allow(clippy::missing_docs_in_private_items)]
        struct Display<'a> {
            types: &'a Types,
            ty: &'a CompoundTypeDef,
            optional: bool,
        }

        impl fmt::Display for Display<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self.ty {
                    CompoundTypeDef::Array(ty) => ty.display(self.types).fmt(f)?,
                    CompoundTypeDef::Pair(ty) => ty.display(self.types).fmt(f)?,
                    CompoundTypeDef::Map(ty) => ty.display(self.types).fmt(f)?,
                    CompoundTypeDef::Struct(ty) => ty.fmt(f)?,
                }

                if self.optional {
                    write!(f, "?")?;
                }

                Ok(())
            }
        }

        Display {
            types,
            ty: types.type_definition(self.definition),
            optional: self.optional,
        }
    }

    /// Asserts that the type is valid.
    fn assert_valid(&self, types: &Types) {
        types.type_definition(self.definition).assert_valid(types);
    }
}

impl Optional for CompoundType {
    fn is_optional(&self) -> bool {
        self.optional
    }

    fn optional(&self) -> Self {
        Self {
            definition: self.definition,
            optional: true,
        }
    }

    fn require(&self) -> Self {
        Self {
            definition: self.definition,
            optional: false,
        }
    }
}

impl Coercible for CompoundType {
    fn is_coercible_to(&self, types: &Types, target: &Self) -> bool {
        if self.is_optional() && !target.is_optional() {
            return false;
        }

        types
            .type_definition(self.definition)
            .is_coercible_to(types, types.type_definition(target.definition))
    }
}

impl TypeEq for CompoundType {
    fn type_eq(&self, types: &Types, other: &Self) -> bool {
        if self.optional != other.optional {
            return false;
        }

        if self.definition == other.definition {
            return true;
        }

        match (
            types.type_definition(self.definition),
            types.type_definition(other.definition),
        ) {
            (CompoundTypeDef::Array(a), CompoundTypeDef::Array(b)) => a.type_eq(types, b),
            (CompoundTypeDef::Pair(a), CompoundTypeDef::Pair(b)) => a.type_eq(types, b),
            (CompoundTypeDef::Map(a), CompoundTypeDef::Map(b)) => a.type_eq(types, b),
            (CompoundTypeDef::Struct(_), CompoundTypeDef::Struct(_)) => {
                // Struct types are only equivalent if they're the same definition
                false
            }
            _ => false,
        }
    }
}

/// Represents a compound type definition.
#[derive(Debug)]
pub enum CompoundTypeDef {
    /// The type is an `Array`.
    Array(ArrayType),
    /// The type is a `Pair`.
    Pair(PairType),
    /// The type is a `Map`.
    Map(MapType),
    /// The type is a struct (e.g. `Foo`).
    Struct(StructType),
}

impl CompoundTypeDef {
    /// Asserts that this type is valid.
    fn assert_valid(&self, types: &Types) {
        match self {
            Self::Array(ty) => {
                ty.assert_valid(types);
            }
            Self::Pair(ty) => {
                ty.assert_valid(types);
            }
            Self::Map(ty) => {
                ty.assert_valid(types);
            }
            Self::Struct(ty) => {
                ty.assert_valid(types);
            }
        }
    }
}

impl Coercible for CompoundTypeDef {
    fn is_coercible_to(&self, types: &Types, target: &Self) -> bool {
        match (self, target) {
            // Array[X] -> Array[Y], Array[X] -> Array[Y]?, Array[X]? -> Array[Y]?, Array[X]+ ->
            // Array[Y] (if X is coercible to Y)
            (Self::Array(src), Self::Array(target)) => src.is_coercible_to(types, target),

            // Pair[W, X] -> Pair[Y, Z], Pair[W, X] -> Pair[Y, Z]?, Pair[W, X]? -> Pair[Y, Z]? (if W
            // is coercible to Y and X is coercible to Z)
            (Self::Pair(src), Self::Pair(target)) => src.is_coercible_to(types, target),

            // Map[W, X] -> Map[Y, Z], Map[W, X] -> Map[Y, Z]?, Map[W, X]? -> Map[Y, Z]? (if W is
            // coercible to Y and X is coercible to Z)
            (Self::Map(src), Self::Map(target)) => src.is_coercible_to(types, target),

            // Struct -> Struct, Struct -> Struct?, Struct? -> Struct? (if the two struct types have
            // members with identical names and compatible types)
            (Self::Struct(src), Self::Struct(target)) => src.is_coercible_to(types, target),

            // Map[String, X] -> Struct, Map[String, X] -> Struct?, Map[String, X]? -> Struct? (if
            // `Map` keys match struct member name and all struct member types are coercible from X)
            (Self::Map(src), Self::Struct(target)) => {
                if src.key_type.kind() != PrimitiveTypeKind::String {
                    return false;
                }

                // Ensure the value type is coercible to every struct member type
                if !target
                    .members
                    .values()
                    .all(|ty| src.value_type.is_coercible_to(types, ty))
                {
                    return false;
                }

                // Note: checking map keys is a runtime value constraint
                true
            }

            // Struct -> Map[String, X], Struct -> Map[String, X]?, Struct? -> Map[String, X]? (if
            // all struct members are coercible to X)
            (Self::Struct(src), Self::Map(target)) => {
                if target.key_type.kind() != PrimitiveTypeKind::String {
                    return false;
                }

                // Ensure all the struct members are coercible to the value type
                if !src
                    .members
                    .values()
                    .all(|ty| ty.is_coercible_to(types, &target.value_type))
                {
                    return false;
                }

                true
            }

            _ => false,
        }
    }
}

impl From<ArrayType> for CompoundTypeDef {
    fn from(value: ArrayType) -> Self {
        Self::Array(value)
    }
}

impl From<PairType> for CompoundTypeDef {
    fn from(value: PairType) -> Self {
        Self::Pair(value)
    }
}

impl From<MapType> for CompoundTypeDef {
    fn from(value: MapType) -> Self {
        Self::Map(value)
    }
}

impl From<StructType> for CompoundTypeDef {
    fn from(value: StructType) -> Self {
        Self::Struct(value)
    }
}

/// Represents the type of an `Array`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArrayType {
    /// The element type of the array.
    element_type: Type,
    /// Whether or not the array type is non-empty.
    non_empty: bool,
}

impl ArrayType {
    /// Constructs a new array type.
    pub fn new(element_type: impl Into<Type>) -> Self {
        Self {
            element_type: element_type.into(),
            non_empty: false,
        }
    }

    /// Constructs a new non-empty array type.
    pub fn non_empty(element_type: impl Into<Type>) -> Self {
        Self {
            element_type: element_type.into(),
            non_empty: true,
        }
    }

    /// Creates a new array type from an V1 AST representation of an array type.
    ///
    /// If a type could not created, an error with the relevant diagnostic is
    /// returned.
    pub fn from_ast_v1<F>(
        types: &mut Types,
        ty: v1::ArrayType,
        lookup: &F,
    ) -> Result<Self, Diagnostic>
    where
        F: Fn(&str) -> Option<Type>,
    {
        Ok(Self {
            element_type: Type::from_ast_v1(types, ty.element_type(), lookup)?,
            non_empty: ty.is_non_empty(),
        })
    }

    /// Gets the array's element type.
    pub fn element_type(&self) -> Type {
        self.element_type
    }

    /// Determines if the array type is non-empty.
    pub fn is_non_empty(&self) -> bool {
        self.non_empty
    }

    /// Returns an object that implements `Display` for formatting the type.
    pub fn display<'a>(&'a self, types: &'a Types) -> impl fmt::Display + 'a {
        #[allow(clippy::missing_docs_in_private_items)]
        struct Display<'a> {
            types: &'a Types,
            ty: &'a ArrayType,
        }

        impl fmt::Display for Display<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "Array[")?;
                self.ty.element_type.display(self.types).fmt(f)?;
                write!(f, "]")?;

                if self.ty.non_empty {
                    write!(f, "+")?;
                }

                Ok(())
            }
        }

        Display { types, ty: self }
    }

    /// Asserts that the type is valid.
    fn assert_valid(&self, types: &Types) {
        self.element_type.assert_valid(types);
    }
}

impl Coercible for ArrayType {
    fn is_coercible_to(&self, types: &Types, target: &Self) -> bool {
        if !self.is_non_empty() && target.is_non_empty() {
            return false;
        }

        self.element_type
            .is_coercible_to(types, &target.element_type)
    }
}

impl TypeEq for ArrayType {
    fn type_eq(&self, types: &Types, other: &Self) -> bool {
        self.non_empty == other.non_empty && self.element_type.type_eq(types, &other.element_type)
    }
}

/// Represents the type of a `Pair`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PairType {
    /// The type of the first element of the pair.
    first_type: Type,
    /// The type of the second element of the pair.
    second_type: Type,
}

impl PairType {
    /// Constructs a new pair type.
    pub fn new(first_type: impl Into<Type>, second_type: impl Into<Type>) -> Self {
        Self {
            first_type: first_type.into(),
            second_type: second_type.into(),
        }
    }

    /// Creates a new pair type from an V1 AST representation of a pair type.
    ///
    /// If a type could not created, an error with the relevant diagnostic is
    /// returned.
    pub fn from_ast_v1<F>(
        types: &mut Types,
        ty: v1::PairType,
        lookup: &F,
    ) -> Result<Self, Diagnostic>
    where
        F: Fn(&str) -> Option<Type>,
    {
        let (first_type, second_type) = ty.types();

        Ok(Self {
            first_type: Type::from_ast_v1(types, first_type, lookup)?,
            second_type: Type::from_ast_v1(types, second_type, lookup)?,
        })
    }

    /// Gets the pairs's first type.
    pub fn first_type(&self) -> Type {
        self.first_type
    }

    /// Gets the pairs's second type.
    pub fn second_type(&self) -> Type {
        self.second_type
    }

    /// Returns an object that implements `Display` for formatting the type.
    pub fn display<'a>(&'a self, types: &'a Types) -> impl fmt::Display + 'a {
        #[allow(clippy::missing_docs_in_private_items)]
        struct Display<'a> {
            types: &'a Types,
            ty: &'a PairType,
        }

        impl fmt::Display for Display<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "Pair[")?;
                self.ty.first_type.display(self.types).fmt(f)?;
                write!(f, ", ")?;
                self.ty.second_type.display(self.types).fmt(f)?;
                write!(f, "]")
            }
        }

        Display { types, ty: self }
    }

    /// Asserts that the type is valid.
    fn assert_valid(&self, types: &Types) {
        self.first_type.assert_valid(types);
        self.second_type.assert_valid(types);
    }
}

impl Coercible for PairType {
    fn is_coercible_to(&self, types: &Types, target: &Self) -> bool {
        self.first_type.is_coercible_to(types, &target.first_type)
            && self.second_type.is_coercible_to(types, &target.second_type)
    }
}

impl TypeEq for PairType {
    fn type_eq(&self, types: &Types, other: &Self) -> bool {
        self.first_type.type_eq(types, &other.first_type)
            && self.second_type.type_eq(types, &other.second_type)
    }
}

/// Represents the type of a `Map`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MapType {
    /// The key type of the map.
    key_type: PrimitiveType,
    /// The value type of the map.
    value_type: Type,
}

impl MapType {
    /// Constructs a new map type.
    pub fn new(key_type: impl Into<PrimitiveType>, value_type: impl Into<Type>) -> Self {
        Self {
            key_type: key_type.into(),
            value_type: value_type.into(),
        }
    }

    /// Creates a new map type from an V1 AST representation of a map type.
    ///
    /// If a type could not created, an error with the relevant diagnostic is
    /// returned.
    pub fn from_ast_v1<F>(
        types: &mut Types,
        ty: v1::MapType,
        lookup: &F,
    ) -> Result<Self, Diagnostic>
    where
        F: Fn(&str) -> Option<Type>,
    {
        let (key_type, value_type) = ty.types();

        Ok(Self {
            key_type: key_type.into(),
            value_type: Type::from_ast_v1(types, value_type, lookup)?,
        })
    }

    /// Gets the maps's key type.
    pub fn key_type(&self) -> PrimitiveType {
        self.key_type
    }

    /// Gets the maps's value type.
    pub fn value_type(&self) -> Type {
        self.value_type
    }

    /// Returns an object that implements `Display` for formatting the type.
    pub fn display<'a>(&'a self, types: &'a Types) -> impl fmt::Display + 'a {
        #[allow(clippy::missing_docs_in_private_items)]
        struct Display<'a> {
            types: &'a Types,
            ty: &'a MapType,
        }

        impl fmt::Display for Display<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "Map[")?;
                self.ty.key_type.fmt(f)?;
                write!(f, ", ")?;
                self.ty.value_type.display(self.types).fmt(f)?;
                write!(f, "]")
            }
        }

        Display { types, ty: self }
    }

    /// Asserts that the type is valid.
    fn assert_valid(&self, types: &Types) {
        self.value_type.assert_valid(types);
    }
}

impl Coercible for MapType {
    fn is_coercible_to(&self, types: &Types, target: &Self) -> bool {
        self.key_type.is_coercible_to(types, &target.key_type)
            && self.value_type.is_coercible_to(types, &target.value_type)
    }
}

impl TypeEq for MapType {
    fn type_eq(&self, types: &Types, other: &Self) -> bool {
        self.key_type.type_eq(types, &other.key_type)
            && self.value_type.type_eq(types, &other.value_type)
    }
}

/// Represents the type of a struct.
#[derive(Debug)]
pub struct StructType {
    /// The name of the struct.
    name: String,
    /// The members of the struct.
    members: IndexMap<String, Type>,
}

impl StructType {
    /// Constructs a new struct type definition.
    pub fn new<N, T>(name: impl Into<String>, members: impl IntoIterator<Item = (N, T)>) -> Self
    where
        N: Into<String>,
        T: Into<Type>,
    {
        Self {
            name: name.into(),
            members: members
                .into_iter()
                .map(|(n, ty)| (n.into(), ty.into()))
                .collect(),
        }
    }

    /// Creates a new struct type from an V1 AST representation of a struct
    /// definition.
    ///
    /// The provided callback is used to look up type name references.
    ///
    /// If the type could not created, an error with the relevant diagnostic is
    /// returned.
    pub fn from_ast_v1<F>(
        types: &mut Types,
        definition: &v1::StructDefinition,
        lookup: &F,
    ) -> Result<Self, Diagnostic>
    where
        F: Fn(&str) -> Option<Type>,
    {
        Ok(Self {
            name: definition.name().as_str().into(),
            members: definition
                .members()
                .map(|d| {
                    Ok((
                        d.name().as_str().to_string(),
                        Type::from_ast_v1(types, d.ty(), lookup)?,
                    ))
                })
                .collect::<Result<_, _>>()?,
        })
    }

    /// Gets the name of the struct.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets the members of the struct.
    pub fn members(&self) -> &IndexMap<String, Type> {
        &self.members
    }

    /// Asserts that this type is valid.
    fn assert_valid(&self, types: &Types) {
        for v in self.members.values() {
            v.assert_valid(types);
        }
    }
}

impl Coercible for StructType {
    fn is_coercible_to(&self, types: &Types, target: &Self) -> bool {
        if self.members.len() != target.members.len() {
            return false;
        }

        self.members.iter().all(|(k, v)| {
            target
                .members
                .get(k)
                .map(|target| v.is_coercible_to(types, target))
                .unwrap_or(false)
        })
    }
}

impl fmt::Display for StructType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{name}", name = self.name)
    }
}

/// Represents a collection of types.
#[derive(Debug, Default)]
pub struct Types(Arena<CompoundTypeDef>);

impl Types {
    /// Constructs a new type collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an array type to the type collection.
    ///
    /// # Panics
    ///
    /// Panics if the provided type contains a type definition identifier from a
    /// different types collection.
    pub fn add_array(&mut self, ty: ArrayType) -> Type {
        ty.assert_valid(self);
        Type::Compound(CompoundType {
            definition: self.0.alloc(CompoundTypeDef::Array(ty)),
            optional: false,
        })
    }

    /// Adds a pair type to the type collection.
    ///
    /// # Panics
    ///
    /// Panics if the provided type contains a type definition identifier from a
    /// different types collection.
    pub fn add_pair(&mut self, ty: PairType) -> Type {
        ty.assert_valid(self);
        Type::Compound(CompoundType {
            definition: self.0.alloc(CompoundTypeDef::Pair(ty)),
            optional: false,
        })
    }

    /// Adds a map type to the type collection.
    ///
    /// # Panics
    ///
    /// Panics if the provided type contains a type definition identifier from a
    /// different types collection.
    pub fn add_map(&mut self, ty: MapType) -> Type {
        ty.assert_valid(self);
        Type::Compound(CompoundType {
            definition: self.0.alloc(CompoundTypeDef::Map(ty)),
            optional: false,
        })
    }

    /// Adds a struct type to the type collection.
    ///
    /// # Panics
    ///
    /// Panics if the provided type contains a type definition identifier from a
    /// different types collection.
    pub fn add_struct(&mut self, ty: StructType) -> Type {
        ty.assert_valid(self);
        Type::Compound(CompoundType {
            definition: self.0.alloc(CompoundTypeDef::Struct(ty)),
            optional: false,
        })
    }

    /// Gets a compound type definition by identifier.
    ///
    /// # Panics
    ///
    /// Panics if the identifier is not for this type collection.
    pub fn type_definition(&self, id: CompoundTypeDefId) -> &CompoundTypeDef {
        self.0
            .get(id)
            // Fall back to types defined by the standard library
            .or_else(|| STDLIB.types().0.get(id))
            .expect("invalid type identifier")
    }

    /// Imports a type from a foreign type collection.
    ///
    /// Returns the new type that is local to this type collection.
    pub fn import(&mut self, types: &Self, ty: Type) -> Type {
        match ty {
            Type::Primitive(ty) => Type::Primitive(ty),
            Type::Compound(ty) => match &types.0[ty.definition] {
                CompoundTypeDef::Array(ty) => {
                    let element_type = self.import(types, ty.element_type);
                    self.add_array(ArrayType {
                        element_type,
                        non_empty: ty.non_empty,
                    })
                }
                CompoundTypeDef::Pair(ty) => {
                    let first_type = self.import(types, ty.first_type);
                    let second_type = self.import(types, ty.second_type);
                    self.add_pair(PairType {
                        first_type,
                        second_type,
                    })
                }
                CompoundTypeDef::Map(ty) => {
                    let value_type = self.import(types, ty.value_type);
                    self.add_map(MapType {
                        key_type: ty.key_type,
                        value_type,
                    })
                }
                CompoundTypeDef::Struct(ty) => {
                    let members = ty
                        .members
                        .iter()
                        .map(|(k, v)| (k.clone(), self.import(types, *v)))
                        .collect();

                    self.add_struct(StructType {
                        name: ty.name.clone(),
                        members,
                    })
                }
            },
            Type::Object => Type::Object,
            Type::OptionalObject => Type::OptionalObject,
            Type::Union => Type::Union,
            Type::None => Type::None,
        }
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn primitive_type_display() {
        assert_eq!(
            PrimitiveType::new(PrimitiveTypeKind::Boolean).to_string(),
            "Boolean"
        );
        assert_eq!(
            PrimitiveType::new(PrimitiveTypeKind::Integer).to_string(),
            "Int"
        );
        assert_eq!(
            PrimitiveType::new(PrimitiveTypeKind::Float).to_string(),
            "Float"
        );
        assert_eq!(
            PrimitiveType::new(PrimitiveTypeKind::String).to_string(),
            "String"
        );
        assert_eq!(
            PrimitiveType::new(PrimitiveTypeKind::File).to_string(),
            "File"
        );
        assert_eq!(
            PrimitiveType::new(PrimitiveTypeKind::Directory).to_string(),
            "Directory"
        );
        assert_eq!(
            PrimitiveType::optional(PrimitiveTypeKind::Boolean).to_string(),
            "Boolean?"
        );
        assert_eq!(
            PrimitiveType::optional(PrimitiveTypeKind::Integer).to_string(),
            "Int?"
        );
        assert_eq!(
            PrimitiveType::optional(PrimitiveTypeKind::Float).to_string(),
            "Float?"
        );
        assert_eq!(
            PrimitiveType::optional(PrimitiveTypeKind::String).to_string(),
            "String?"
        );
        assert_eq!(
            PrimitiveType::optional(PrimitiveTypeKind::File).to_string(),
            "File?"
        );
        assert_eq!(
            PrimitiveType::optional(PrimitiveTypeKind::Directory).to_string(),
            "Directory?"
        );
    }

    #[test]
    fn array_type_display() {
        let mut types = Types::new();
        assert_eq!(
            ArrayType::new(PrimitiveTypeKind::String)
                .display(&types)
                .to_string(),
            "Array[String]"
        );
        assert_eq!(
            ArrayType::non_empty(PrimitiveTypeKind::String)
                .display(&types)
                .to_string(),
            "Array[String]+"
        );

        let ty = types.add_array(ArrayType::new(PrimitiveTypeKind::String));
        assert_eq!(
            types
                .add_array(ArrayType::new(ty))
                .display(&types)
                .to_string(),
            "Array[Array[String]]"
        );

        let ty = types
            .add_array(ArrayType::non_empty(PrimitiveType::optional(
                PrimitiveTypeKind::String,
            )))
            .optional();
        assert_eq!(
            types
                .add_array(ArrayType::non_empty(ty))
                .optional()
                .display(&types)
                .to_string(),
            "Array[Array[String?]+?]+?"
        );
    }

    #[test]
    fn pair_type_display() {
        let mut types = Types::new();
        assert_eq!(
            PairType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::Boolean)
                .display(&types)
                .to_string(),
            "Pair[String, Boolean]"
        );

        let ty = types.add_array(ArrayType::new(PrimitiveTypeKind::String));
        assert_eq!(
            types
                .add_pair(PairType::new(ty, ty))
                .display(&types)
                .to_string(),
            "Pair[Array[String], Array[String]]"
        );

        let ty = types
            .add_array(ArrayType::non_empty(PrimitiveType::optional(
                PrimitiveTypeKind::File,
            )))
            .optional();
        assert_eq!(
            types
                .add_pair(PairType::new(ty, ty))
                .optional()
                .display(&types)
                .to_string(),
            "Pair[Array[File?]+?, Array[File?]+?]?"
        );
    }

    #[test]
    fn map_type_display() {
        let mut types = Types::new();
        assert_eq!(
            MapType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::Boolean)
                .display(&types)
                .to_string(),
            "Map[String, Boolean]"
        );

        let ty = types.add_array(ArrayType::new(PrimitiveTypeKind::String));
        assert_eq!(
            types
                .add_map(MapType::new(PrimitiveTypeKind::Boolean, ty))
                .display(&types)
                .to_string(),
            "Map[Boolean, Array[String]]"
        );

        let ty = types
            .add_array(ArrayType::non_empty(PrimitiveType::optional(
                PrimitiveTypeKind::File,
            )))
            .optional();
        assert_eq!(
            types
                .add_map(MapType::new(PrimitiveTypeKind::String, ty))
                .optional()
                .display(&types)
                .to_string(),
            "Map[String, Array[File?]+?]?"
        );
    }

    #[test]
    fn struct_type_display() {
        assert_eq!(
            StructType::new("Foobar", std::iter::empty::<(String, Type)>()).to_string(),
            "Foobar"
        );
    }

    #[test]
    fn object_type_display() {
        let types = Types::new();
        assert_eq!(Type::Object.display(&types).to_string(), "Object");
        assert_eq!(Type::OptionalObject.display(&types).to_string(), "Object?");
    }

    #[test]
    fn union_type_display() {
        let types = Types::new();
        assert_eq!(Type::Union.display(&types).to_string(), "Union");
    }

    #[test]
    fn none_type_display() {
        let types = Types::new();
        assert_eq!(Type::None.display(&types).to_string(), "None");
    }

    #[test]
    fn primitive_type_coercion() {
        let types = Types::new();

        // All types should be coercible to self, and required should coerce to optional
        // (but not vice versa)
        for kind in [
            PrimitiveTypeKind::Boolean,
            PrimitiveTypeKind::Directory,
            PrimitiveTypeKind::File,
            PrimitiveTypeKind::Float,
            PrimitiveTypeKind::Integer,
            PrimitiveTypeKind::String,
        ] {
            assert!(PrimitiveType::new(kind).is_coercible_to(&types, &PrimitiveType::new(kind)));
            assert!(
                PrimitiveType::optional(kind)
                    .is_coercible_to(&types, &PrimitiveType::optional(kind))
            );
            assert!(
                PrimitiveType::new(kind).is_coercible_to(&types, &PrimitiveType::optional(kind))
            );
            assert!(
                !PrimitiveType::optional(kind).is_coercible_to(&types, &PrimitiveType::new(kind))
            );
        }

        // Check the valid coercions
        assert!(
            PrimitiveType::new(PrimitiveTypeKind::String)
                .is_coercible_to(&types, &PrimitiveType::new(PrimitiveTypeKind::File))
        );
        assert!(
            PrimitiveType::new(PrimitiveTypeKind::String)
                .is_coercible_to(&types, &PrimitiveType::new(PrimitiveTypeKind::Directory))
        );
        assert!(
            PrimitiveType::new(PrimitiveTypeKind::Integer)
                .is_coercible_to(&types, &PrimitiveType::new(PrimitiveTypeKind::Float))
        );
        assert!(
            !PrimitiveType::new(PrimitiveTypeKind::File)
                .is_coercible_to(&types, &PrimitiveType::new(PrimitiveTypeKind::String))
        );
        assert!(
            !PrimitiveType::new(PrimitiveTypeKind::Directory)
                .is_coercible_to(&types, &PrimitiveType::new(PrimitiveTypeKind::String))
        );
        assert!(
            !PrimitiveType::new(PrimitiveTypeKind::Float)
                .is_coercible_to(&types, &PrimitiveType::new(PrimitiveTypeKind::Integer))
        );
    }

    #[test]
    fn object_type_coercion() {
        let mut types = Types::new();
        assert!(Type::Object.is_coercible_to(&types, &Type::Object));
        assert!(Type::Object.is_coercible_to(&types, &Type::OptionalObject));
        assert!(Type::OptionalObject.is_coercible_to(&types, &Type::OptionalObject));
        assert!(!Type::OptionalObject.is_coercible_to(&types, &Type::Object));

        // Object -> Map[String, X]
        let ty = types.add_map(MapType::new(
            PrimitiveTypeKind::String,
            PrimitiveTypeKind::String,
        ));
        assert!(!Type::OptionalObject.is_coercible_to(&types, &ty));

        // Object -> Map[Int, X] (not a string key)
        let ty = types.add_map(MapType::new(
            PrimitiveTypeKind::Integer,
            PrimitiveTypeKind::String,
        ));
        assert!(!Type::Object.is_coercible_to(&types, &ty));

        // Object -> Map[String, X]?
        let ty = types
            .add_map(MapType::new(
                PrimitiveTypeKind::String,
                PrimitiveTypeKind::String,
            ))
            .optional();
        assert!(Type::Object.is_coercible_to(&types, &ty));

        // Object? -> Map[String, X]?
        let ty = types
            .add_map(MapType::new(
                PrimitiveTypeKind::String,
                PrimitiveTypeKind::String,
            ))
            .optional();
        assert!(Type::OptionalObject.is_coercible_to(&types, &ty));

        // Object? -> Map[String, X]
        let ty = types.add_map(MapType::new(
            PrimitiveTypeKind::String,
            PrimitiveTypeKind::String,
        ));
        assert!(!Type::OptionalObject.is_coercible_to(&types, &ty));

        // Object -> Struct
        let ty = types.add_struct(StructType::new("Foo", [("foo", PrimitiveTypeKind::String)]));
        assert!(Type::Object.is_coercible_to(&types, &ty));

        // Object -> Struct?
        let ty = types
            .add_struct(StructType::new("Foo", [("foo", PrimitiveTypeKind::String)]))
            .optional();
        assert!(Type::Object.is_coercible_to(&types, &ty));

        // Object? -> Struct?
        let ty = types
            .add_struct(StructType::new("Foo", [("foo", PrimitiveTypeKind::String)]))
            .optional();
        assert!(Type::OptionalObject.is_coercible_to(&types, &ty));

        // Object? -> Struct
        let ty = types.add_struct(StructType::new("Foo", [("foo", PrimitiveTypeKind::String)]));
        assert!(!Type::OptionalObject.is_coercible_to(&types, &ty));
    }

    #[test]
    fn array_type_coercion() {
        let mut types = Types::new();

        // Array[X] -> Array[Y]
        assert!(
            ArrayType::new(PrimitiveTypeKind::String)
                .is_coercible_to(&types, &ArrayType::new(PrimitiveTypeKind::String))
        );
        assert!(
            !ArrayType::new(PrimitiveTypeKind::File)
                .is_coercible_to(&types, &ArrayType::new(PrimitiveTypeKind::String))
        );
        assert!(
            ArrayType::new(PrimitiveTypeKind::String)
                .is_coercible_to(&types, &ArrayType::new(PrimitiveTypeKind::File))
        );

        // Array[X] -> Array[Y?]
        let type1 = types.add_array(ArrayType::new(PrimitiveTypeKind::String));
        let type2 = types.add_array(ArrayType::new(PrimitiveType::optional(
            PrimitiveTypeKind::File,
        )));
        assert!(type1.is_coercible_to(&types, &type2));
        assert!(!type2.is_coercible_to(&types, &type1));

        // Array[Array[X]] -> Array[Array[Y]]
        let type1 = types.add_array(ArrayType::new(type1));
        let type2 = types.add_array(ArrayType::new(type2));
        assert!(type1.is_coercible_to(&types, &type2));
        assert!(!type2.is_coercible_to(&types, &type1));

        // Array[X]+ -> Array[Y]
        let type1 = types.add_array(ArrayType::non_empty(PrimitiveTypeKind::String));
        let type2 = types.add_array(ArrayType::new(PrimitiveType::optional(
            PrimitiveTypeKind::File,
        )));
        assert!(type1.is_coercible_to(&types, &type2));
        assert!(!type2.is_coercible_to(&types, &type1));

        // Array[X] -> Array[X]
        let type1 = types.add_array(ArrayType::new(PrimitiveTypeKind::String));
        let type2 = types.add_array(ArrayType::new(PrimitiveTypeKind::String));
        assert!(type1.is_coercible_to(&types, &type2));
        assert!(type2.is_coercible_to(&types, &type1));

        // Array[X]? -> Array[X]?
        let type1 = types
            .add_array(ArrayType::new(PrimitiveTypeKind::String))
            .optional();
        let type2 = types
            .add_array(ArrayType::new(PrimitiveTypeKind::String))
            .optional();
        assert!(type1.is_coercible_to(&types, &type2));
        assert!(type2.is_coercible_to(&types, &type1));

        // Array[X] -> Array[X]?
        let type1 = types.add_array(ArrayType::new(PrimitiveTypeKind::String));
        let type2 = types
            .add_array(ArrayType::new(PrimitiveTypeKind::String))
            .optional();
        assert!(type1.is_coercible_to(&types, &type2));
        assert!(!type2.is_coercible_to(&types, &type1));
    }

    #[test]
    fn pair_type_coercion() {
        let mut types = Types::new();

        // Pair[W, X] -> Pair[Y, Z]
        assert!(
            PairType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::String).is_coercible_to(
                &types,
                &PairType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::String)
            )
        );
        assert!(
            PairType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::String).is_coercible_to(
                &types,
                &PairType::new(PrimitiveTypeKind::File, PrimitiveTypeKind::Directory)
            )
        );
        assert!(
            !PairType::new(PrimitiveTypeKind::File, PrimitiveTypeKind::Directory).is_coercible_to(
                &types,
                &PairType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::String)
            )
        );

        // Pair[W, X] -> Pair[Y?, Z?]
        let type1 = types.add_pair(PairType::new(
            PrimitiveTypeKind::String,
            PrimitiveTypeKind::String,
        ));
        let type2 = types.add_pair(PairType::new(
            PrimitiveType::optional(PrimitiveTypeKind::File),
            PrimitiveType::optional(PrimitiveTypeKind::Directory),
        ));
        assert!(type1.is_coercible_to(&types, &type2));
        assert!(!type2.is_coercible_to(&types, &type1));

        // Pair[Pair[W, X], Pair[W, X]] -> Pair[Pair[Y, Z], Pair[Y, Z]]
        let type1 = types.add_pair(PairType::new(type1, type1));
        let type2 = types.add_pair(PairType::new(type2, type2));
        assert!(type1.is_coercible_to(&types, &type2));
        assert!(!type2.is_coercible_to(&types, &type1));

        // Pair[W, X] -> Pair[W, X]
        let type1 = types.add_pair(PairType::new(
            PrimitiveTypeKind::String,
            PrimitiveTypeKind::String,
        ));
        let type2 = types.add_pair(PairType::new(
            PrimitiveTypeKind::String,
            PrimitiveTypeKind::String,
        ));
        assert!(type1.is_coercible_to(&types, &type2));
        assert!(type2.is_coercible_to(&types, &type1));

        // Pair[W, X]? -> Pair[W, X]?
        let type1 = types
            .add_pair(PairType::new(
                PrimitiveTypeKind::String,
                PrimitiveTypeKind::String,
            ))
            .optional();
        let type2 = types
            .add_pair(PairType::new(
                PrimitiveTypeKind::String,
                PrimitiveTypeKind::String,
            ))
            .optional();
        assert!(type1.is_coercible_to(&types, &type2));
        assert!(type2.is_coercible_to(&types, &type1));

        // Pair[W, X] -> Pair[W, X]?
        let type1 = types.add_pair(PairType::new(
            PrimitiveTypeKind::String,
            PrimitiveTypeKind::String,
        ));
        let type2 = types
            .add_pair(PairType::new(
                PrimitiveTypeKind::String,
                PrimitiveTypeKind::String,
            ))
            .optional();
        assert!(type1.is_coercible_to(&types, &type2));
        assert!(!type2.is_coercible_to(&types, &type1));
    }

    #[test]
    fn map_type_coercion() {
        let mut types = Types::new();

        // Map[W, X] -> Map[Y, Z]
        assert!(
            MapType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::String).is_coercible_to(
                &types,
                &MapType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::String)
            )
        );
        assert!(
            MapType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::String).is_coercible_to(
                &types,
                &MapType::new(PrimitiveTypeKind::File, PrimitiveTypeKind::Directory)
            )
        );
        assert!(
            !MapType::new(PrimitiveTypeKind::File, PrimitiveTypeKind::Directory).is_coercible_to(
                &types,
                &MapType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::String)
            )
        );

        // Map[W, X] -> Map[Y?, Z?]
        let type1 = types.add_map(MapType::new(
            PrimitiveTypeKind::String,
            PrimitiveTypeKind::String,
        ));
        let type2 = types.add_map(MapType::new(
            PrimitiveType::optional(PrimitiveTypeKind::File),
            PrimitiveType::optional(PrimitiveTypeKind::Directory),
        ));
        assert!(type1.is_coercible_to(&types, &type2));
        assert!(!type2.is_coercible_to(&types, &type1));

        // Map[P, Map[W, X]] -> Map[Q, Map[Y, Z]]
        let type1 = types.add_map(MapType::new(PrimitiveTypeKind::String, type1));
        let type2 = types.add_map(MapType::new(PrimitiveTypeKind::Directory, type2));
        assert!(type1.is_coercible_to(&types, &type2));
        assert!(!type2.is_coercible_to(&types, &type1));

        // Map[W, X] -> Map[W, X]
        let type1 = types.add_map(MapType::new(
            PrimitiveTypeKind::String,
            PrimitiveTypeKind::String,
        ));
        let type2 = types.add_map(MapType::new(
            PrimitiveTypeKind::String,
            PrimitiveTypeKind::String,
        ));
        assert!(type1.is_coercible_to(&types, &type2));
        assert!(type2.is_coercible_to(&types, &type1));

        // Map[W, X]? -> Map[W, X]?
        let type1 = types
            .add_map(MapType::new(
                PrimitiveTypeKind::String,
                PrimitiveTypeKind::String,
            ))
            .optional();
        let type2: Type = types
            .add_map(MapType::new(
                PrimitiveTypeKind::String,
                PrimitiveTypeKind::String,
            ))
            .optional();
        assert!(type1.is_coercible_to(&types, &type2));
        assert!(type2.is_coercible_to(&types, &type1));

        // Map[W, X] -> Map[W, X]?
        let type1 = types.add_map(MapType::new(
            PrimitiveTypeKind::String,
            PrimitiveTypeKind::String,
        ));
        let type2 = types
            .add_map(MapType::new(
                PrimitiveTypeKind::String,
                PrimitiveTypeKind::String,
            ))
            .optional();
        assert!(type1.is_coercible_to(&types, &type2));
        assert!(!type2.is_coercible_to(&types, &type1));

        // Map[String, X] -> Struct
        let type1 = types.add_map(MapType::new(
            PrimitiveTypeKind::String,
            PrimitiveTypeKind::Integer,
        ));
        let type2 = types.add_struct(StructType::new(
            "Foo",
            [
                ("foo", PrimitiveTypeKind::Integer),
                ("bar", PrimitiveTypeKind::Integer),
                ("baz", PrimitiveTypeKind::Integer),
            ],
        ));
        assert!(type1.is_coercible_to(&types, &type2));

        // Map[String, X] -> Struct (mismatched fields)
        let type1 = types.add_map(MapType::new(
            PrimitiveTypeKind::String,
            PrimitiveTypeKind::Integer,
        ));
        let type2 = types.add_struct(StructType::new(
            "Foo",
            [
                ("foo", PrimitiveTypeKind::Integer),
                ("bar", PrimitiveTypeKind::String),
                ("baz", PrimitiveTypeKind::Integer),
            ],
        ));
        assert!(!type1.is_coercible_to(&types, &type2));

        // Map[Int, X] -> Struct
        let type1 = types.add_map(MapType::new(
            PrimitiveTypeKind::Integer,
            PrimitiveTypeKind::Integer,
        ));
        let type2 = types.add_struct(StructType::new(
            "Foo",
            [
                ("foo", PrimitiveTypeKind::Integer),
                ("bar", PrimitiveTypeKind::Integer),
                ("baz", PrimitiveTypeKind::Integer),
            ],
        ));
        assert!(!type1.is_coercible_to(&types, &type2));

        // Map[String, X] -> Object
        let type1 = types.add_map(MapType::new(
            PrimitiveTypeKind::String,
            PrimitiveTypeKind::Integer,
        ));
        assert!(type1.is_coercible_to(&types, &Type::Object));

        // Map[String, X] -> Object?
        let type1 = types.add_map(MapType::new(
            PrimitiveTypeKind::String,
            PrimitiveTypeKind::Integer,
        ));
        assert!(type1.is_coercible_to(&types, &Type::OptionalObject));

        // Map[String, X]? -> Object?
        let type1 = types
            .add_map(MapType::new(
                PrimitiveTypeKind::String,
                PrimitiveTypeKind::Integer,
            ))
            .optional();
        assert!(type1.is_coercible_to(&types, &Type::OptionalObject));

        // Map[String, X]? -> Object
        let type1 = types
            .add_map(MapType::new(
                PrimitiveTypeKind::String,
                PrimitiveTypeKind::Integer,
            ))
            .optional();
        assert!(!type1.is_coercible_to(&types, &Type::Object));

        // Map[Integer, X] -> Object
        let type1 = types.add_map(MapType::new(
            PrimitiveTypeKind::Integer,
            PrimitiveTypeKind::Integer,
        ));
        assert!(!type1.is_coercible_to(&types, &Type::Object));
    }

    #[test]
    fn struct_type_coercion() {
        let mut types = Types::new();

        // S -> S (identical)
        let type1 = types.add_struct(StructType::new(
            "Foo",
            [
                ("foo", PrimitiveTypeKind::String),
                ("bar", PrimitiveTypeKind::String),
                ("baz", PrimitiveTypeKind::Integer),
            ],
        ));
        let type2 = types.add_struct(StructType::new(
            "Foo",
            [
                ("foo", PrimitiveTypeKind::String),
                ("bar", PrimitiveTypeKind::String),
                ("baz", PrimitiveTypeKind::Integer),
            ],
        ));
        assert!(type1.is_coercible_to(&types, &type2));
        assert!(type2.is_coercible_to(&types, &type1));

        // S -> S?
        let type1 = types.add_struct(StructType::new(
            "Foo",
            [
                ("foo", PrimitiveTypeKind::String),
                ("bar", PrimitiveTypeKind::String),
                ("baz", PrimitiveTypeKind::Integer),
            ],
        ));
        let type2 = types
            .add_struct(StructType::new(
                "Foo",
                [
                    ("foo", PrimitiveTypeKind::String),
                    ("bar", PrimitiveTypeKind::String),
                    ("baz", PrimitiveTypeKind::Integer),
                ],
            ))
            .optional();
        assert!(type1.is_coercible_to(&types, &type2));
        assert!(!type2.is_coercible_to(&types, &type1));

        // S? -> S?
        let type1 = types
            .add_struct(StructType::new(
                "Foo",
                [
                    ("foo", PrimitiveTypeKind::String),
                    ("bar", PrimitiveTypeKind::String),
                    ("baz", PrimitiveTypeKind::Integer),
                ],
            ))
            .optional();
        let type2 = types
            .add_struct(StructType::new(
                "Foo",
                [
                    ("foo", PrimitiveTypeKind::String),
                    ("bar", PrimitiveTypeKind::String),
                    ("baz", PrimitiveTypeKind::Integer),
                ],
            ))
            .optional();
        assert!(type1.is_coercible_to(&types, &type2));
        assert!(type2.is_coercible_to(&types, &type1));

        // S -> S (coercible fields)
        let type1 = types.add_struct(StructType::new(
            "Foo",
            [
                ("foo", PrimitiveTypeKind::String),
                ("bar", PrimitiveTypeKind::String),
                ("baz", PrimitiveTypeKind::Integer),
            ],
        ));
        let type2 = types.add_struct(StructType::new(
            "Bar",
            [
                ("foo", PrimitiveTypeKind::File),
                ("bar", PrimitiveTypeKind::Directory),
                ("baz", PrimitiveTypeKind::Float),
            ],
        ));
        assert!(type1.is_coercible_to(&types, &type2));
        assert!(!type2.is_coercible_to(&types, &type1));

        // S -> S (mismatched fields)
        let type1 = types.add_struct(StructType::new(
            "Foo",
            [
                ("foo", PrimitiveTypeKind::String),
                ("bar", PrimitiveTypeKind::String),
                ("baz", PrimitiveTypeKind::Integer),
            ],
        ));
        let type2 = types.add_struct(StructType::new("Bar", [("baz", PrimitiveTypeKind::Float)]));
        assert!(!type1.is_coercible_to(&types, &type2));
        assert!(!type2.is_coercible_to(&types, &type1));

        // Struct -> Map[String, X]
        let type1 = types.add_struct(StructType::new(
            "Foo",
            [
                ("foo", PrimitiveTypeKind::String),
                ("bar", PrimitiveTypeKind::String),
                ("baz", PrimitiveTypeKind::String),
            ],
        ));
        let type2 = types.add_map(MapType::new(
            PrimitiveTypeKind::String,
            PrimitiveTypeKind::String,
        ));
        assert!(type1.is_coercible_to(&types, &type2));

        // Struct -> Map[String, X] (mismatched types)
        let type1 = types.add_struct(StructType::new(
            "Foo",
            [
                ("foo", PrimitiveTypeKind::String),
                ("bar", PrimitiveTypeKind::Integer),
                ("baz", PrimitiveTypeKind::String),
            ],
        ));
        let type2 = types.add_map(MapType::new(
            PrimitiveTypeKind::String,
            PrimitiveTypeKind::String,
        ));
        assert!(!type1.is_coercible_to(&types, &type2));

        // Struct -> Map[Int, X] (not a string key)
        let type1 = types.add_struct(StructType::new(
            "Foo",
            [
                ("foo", PrimitiveTypeKind::String),
                ("bar", PrimitiveTypeKind::String),
                ("baz", PrimitiveTypeKind::String),
            ],
        ));
        let type2 = types.add_map(MapType::new(
            PrimitiveTypeKind::Integer,
            PrimitiveTypeKind::String,
        ));
        assert!(!type1.is_coercible_to(&types, &type2));

        // Struct -> Object
        assert!(type1.is_coercible_to(&types, &Type::Object));

        // Struct -> Object?
        assert!(type1.is_coercible_to(&types, &Type::OptionalObject));

        // Struct? -> Object?
        let type1 = types
            .add_struct(StructType::new("Foo", [("foo", PrimitiveTypeKind::String)]))
            .optional();
        assert!(type1.is_coercible_to(&types, &Type::OptionalObject));

        // Struct? -> Object
        assert!(!type1.is_coercible_to(&types, &Type::Object));
    }

    #[test]
    fn union_type_coercion() {
        let mut types = Types::new();
        // Union -> anything (ok)
        for kind in [
            PrimitiveTypeKind::Boolean,
            PrimitiveTypeKind::Directory,
            PrimitiveTypeKind::File,
            PrimitiveTypeKind::Float,
            PrimitiveTypeKind::Integer,
            PrimitiveTypeKind::String,
        ] {
            assert!(Type::Union.is_coercible_to(&types, &kind.into()));
            assert!(Type::Union.is_coercible_to(&types, &PrimitiveType::optional(kind).into()));
            assert!(!Type::from(kind).is_coercible_to(&types, &Type::Union));
        }

        for optional in [true, false] {
            // Union -> Array[X], Union -> Array[X]?
            let ty = types.add_array(ArrayType::new(PrimitiveTypeKind::String));
            let ty = if optional { ty.optional() } else { ty };

            let coercible = Type::Union.is_coercible_to(&types, &ty);
            assert!(coercible);

            // Union -> Pair[X, Y], Union -> Pair[X, Y]?
            let ty = types.add_pair(PairType::new(
                PrimitiveTypeKind::String,
                PrimitiveTypeKind::Boolean,
            ));
            let ty = if optional { ty.optional() } else { ty };
            let coercible = Type::Union.is_coercible_to(&types, &ty);
            assert!(coercible);

            // Union -> Map[X, Y], Union -> Map[X, Y]?
            let ty = types.add_map(MapType::new(
                PrimitiveTypeKind::String,
                PrimitiveTypeKind::Boolean,
            ));
            let ty = if optional { ty.optional() } else { ty };
            let coercible = Type::Union.is_coercible_to(&types, &ty);
            assert!(coercible);

            // Union -> Struct, Union -> Struct?
            let ty = types.add_struct(StructType::new("Foo", [("foo", PrimitiveTypeKind::String)]));
            let ty = if optional { ty.optional() } else { ty };
            let coercible = Type::Union.is_coercible_to(&types, &ty);
            assert!(coercible);
        }
    }

    #[test]
    fn none_type_coercion() {
        let mut types = Types::new();
        // None -> optional type (ok)
        for kind in [
            PrimitiveTypeKind::Boolean,
            PrimitiveTypeKind::Directory,
            PrimitiveTypeKind::File,
            PrimitiveTypeKind::Float,
            PrimitiveTypeKind::Integer,
            PrimitiveTypeKind::String,
        ] {
            assert!(!Type::None.is_coercible_to(&types, &kind.into()));
            assert!(Type::None.is_coercible_to(&types, &PrimitiveType::optional(kind).into()));
            assert!(!Type::from(kind).is_coercible_to(&types, &Type::None));
        }

        for optional in [true, false] {
            // None -> Array[X], None -> Array[X]?
            let ty = types.add_array(ArrayType::new(PrimitiveTypeKind::String));
            let ty = if optional { ty.optional() } else { ty };
            let coercible = Type::None.is_coercible_to(&types, &ty);
            if optional {
                assert!(coercible);
            } else {
                assert!(!coercible);
            }

            // None -> Pair[X, Y], None -> Pair[X, Y]?
            let ty = types.add_pair(PairType::new(
                PrimitiveTypeKind::String,
                PrimitiveTypeKind::Boolean,
            ));
            let ty = if optional { ty.optional() } else { ty };
            let coercible = Type::None.is_coercible_to(&types, &ty);
            if optional {
                assert!(coercible);
            } else {
                assert!(!coercible);
            }

            // None -> Map[X, Y], None -> Map[X, Y]?
            let ty = types.add_map(MapType::new(
                PrimitiveTypeKind::String,
                PrimitiveTypeKind::Boolean,
            ));
            let ty = if optional { ty.optional() } else { ty };
            let coercible = Type::None.is_coercible_to(&types, &ty);
            if optional {
                assert!(coercible);
            } else {
                assert!(!coercible);
            }

            // None -> Struct, None -> Struct?
            let ty = types.add_struct(StructType::new("Foo", [("foo", PrimitiveTypeKind::String)]));
            let ty = if optional { ty.optional() } else { ty };
            let coercible = Type::None.is_coercible_to(&types, &ty);
            if optional {
                assert!(coercible);
            } else {
                assert!(!coercible);
            }
        }
    }

    #[test]
    fn primitive_type_equality() {
        let types = Types::new();

        for kind in [
            PrimitiveTypeKind::Boolean,
            PrimitiveTypeKind::Directory,
            PrimitiveTypeKind::File,
            PrimitiveTypeKind::Float,
            PrimitiveTypeKind::Integer,
            PrimitiveTypeKind::String,
        ] {
            assert!(PrimitiveType::new(kind).type_eq(&types, &PrimitiveType::new(kind)));
            assert!(!PrimitiveType::optional(kind).type_eq(&types, &PrimitiveType::new(kind)));
            assert!(!PrimitiveType::new(kind).type_eq(&types, &PrimitiveType::optional(kind)));
            assert!(PrimitiveType::optional(kind).type_eq(&types, &PrimitiveType::optional(kind)));
            assert!(!Type::from(PrimitiveType::new(kind)).type_eq(&types, &Type::Object));
            assert!(!Type::from(PrimitiveType::new(kind)).type_eq(&types, &Type::OptionalObject));
            assert!(!Type::from(PrimitiveType::new(kind)).type_eq(&types, &Type::Union));
            assert!(!Type::from(PrimitiveType::new(kind)).type_eq(&types, &Type::None));
        }
    }

    #[test]
    fn array_type_equality() {
        let mut types = Types::new();

        // Array[String] == Array[String]
        let a = types.add_array(ArrayType::new(PrimitiveTypeKind::String));
        let b = types.add_array(ArrayType::new(PrimitiveTypeKind::String));
        assert!(a.type_eq(&types, &b));
        assert!(!a.optional().type_eq(&types, &b));
        assert!(!a.type_eq(&types, &b.optional()));
        assert!(a.optional().type_eq(&types, &b.optional()));

        // Array[Array[String]] == Array[Array[String]
        let a = types.add_array(ArrayType::new(a));
        let b = types.add_array(ArrayType::new(b));
        assert!(a.type_eq(&types, &b));

        // Array[Array[Array[String]]]+ == Array[Array[Array[String]]+
        let a = types.add_array(ArrayType::non_empty(a));
        let b = types.add_array(ArrayType::non_empty(b));
        assert!(a.type_eq(&types, &b));

        // Array[String] != Array[String]+
        let a = types.add_array(ArrayType::new(PrimitiveTypeKind::String));
        let b = types.add_array(ArrayType::non_empty(PrimitiveTypeKind::String));
        assert!(!a.type_eq(&types, &b));

        // Array[String] != Array[Int]
        let a = types.add_array(ArrayType::new(PrimitiveTypeKind::String));
        let b = types.add_array(ArrayType::new(PrimitiveTypeKind::Integer));
        assert!(!a.type_eq(&types, &b));

        assert!(!a.type_eq(&types, &Type::Object));
        assert!(!a.type_eq(&types, &Type::OptionalObject));
        assert!(!a.type_eq(&types, &Type::Union));
        assert!(!a.type_eq(&types, &Type::None));
    }

    #[test]
    fn pair_type_equality() {
        let mut types = Types::new();

        // Pair[String, Int] == Pair[String, Int]
        let a = types.add_pair(PairType::new(
            PrimitiveTypeKind::String,
            PrimitiveTypeKind::Integer,
        ));
        let b = types.add_pair(PairType::new(
            PrimitiveTypeKind::String,
            PrimitiveTypeKind::Integer,
        ));
        assert!(a.type_eq(&types, &b));
        assert!(!a.optional().type_eq(&types, &b));
        assert!(!a.type_eq(&types, &b.optional()));
        assert!(a.optional().type_eq(&types, &b.optional()));

        // Pair[Pair[String, Int], Pair[String, Int]] == Pair[Pair[String, Int],
        // Pair[String, Int]]
        let a = types.add_pair(PairType::new(a, a));
        let b = types.add_pair(PairType::new(b, b));
        assert!(a.type_eq(&types, &b));

        // Pair[String, Int] != Pair[String, Int]?
        let a = types.add_pair(PairType::new(
            PrimitiveTypeKind::String,
            PrimitiveTypeKind::Integer,
        ));
        let b = types
            .add_pair(PairType::new(
                PrimitiveTypeKind::String,
                PrimitiveTypeKind::Integer,
            ))
            .optional();
        assert!(!a.type_eq(&types, &b));

        assert!(!a.type_eq(&types, &Type::Object));
        assert!(!a.type_eq(&types, &Type::OptionalObject));
        assert!(!a.type_eq(&types, &Type::Union));
        assert!(!a.type_eq(&types, &Type::None));
    }

    #[test]
    fn map_type_equality() {
        let mut types = Types::new();

        // Map[String, Int] == Map[String, Int]
        let a = types.add_map(MapType::new(
            PrimitiveTypeKind::String,
            PrimitiveTypeKind::Integer,
        ));
        let b = types.add_map(MapType::new(
            PrimitiveTypeKind::String,
            PrimitiveTypeKind::Integer,
        ));
        assert!(a.type_eq(&types, &b));
        assert!(!a.optional().type_eq(&types, &b));
        assert!(!a.type_eq(&types, &b.optional()));
        assert!(a.optional().type_eq(&types, &b.optional()));

        // Map[File, Map[String, Int]] == Map[File, Map[String, Int]]
        let a = types.add_map(MapType::new(PrimitiveTypeKind::File, a));
        let b = types.add_map(MapType::new(PrimitiveTypeKind::File, b));
        assert!(a.type_eq(&types, &b));

        // Map[String, Int] != Map[Int, String]
        let a = types.add_map(MapType::new(
            PrimitiveTypeKind::String,
            PrimitiveTypeKind::Integer,
        ));
        let b = types.add_map(MapType::new(
            PrimitiveTypeKind::Integer,
            PrimitiveTypeKind::String,
        ));
        assert!(!a.type_eq(&types, &b));

        assert!(!a.type_eq(&types, &Type::Object));
        assert!(!a.type_eq(&types, &Type::OptionalObject));
        assert!(!a.type_eq(&types, &Type::Union));
        assert!(!a.type_eq(&types, &Type::None));
    }

    #[test]
    fn struct_type_equality() {
        let mut types = Types::new();

        let a = types.add_struct(StructType::new("Foo", [("foo", PrimitiveTypeKind::String)]));
        assert!(a.type_eq(&types, &a));
        assert!(!a.optional().type_eq(&types, &a));
        assert!(!a.type_eq(&types, &a.optional()));
        assert!(a.optional().type_eq(&types, &a.optional()));

        let b = types.add_struct(StructType::new("Foo", [("foo", PrimitiveTypeKind::String)]));
        assert!(!a.type_eq(&types, &b));
    }

    #[test]
    fn object_type_equality() {
        let types = Types::new();
        assert!(Type::Object.type_eq(&types, &Type::Object));
        assert!(!Type::OptionalObject.type_eq(&types, &Type::Object));
        assert!(!Type::Object.type_eq(&types, &Type::OptionalObject));
        assert!(Type::OptionalObject.type_eq(&types, &Type::OptionalObject));
    }

    #[test]
    fn union_type_equality() {
        let types = Types::new();
        assert!(Type::Union.type_eq(&types, &Type::Union));
        assert!(!Type::None.type_eq(&types, &Type::Union));
        assert!(!Type::Union.type_eq(&types, &Type::None));
        assert!(Type::None.type_eq(&types, &Type::None));
    }
}
