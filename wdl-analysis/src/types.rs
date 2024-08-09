//! Representation of the WDL type system.

use std::fmt;

use id_arena::Arena;
use id_arena::ArenaBehavior;
use id_arena::DefaultArenaBehavior;
use id_arena::Id;
use indexmap::IndexMap;

use crate::STDLIB;

/// A trait implemented on types that may be optional.
pub trait Optional {
    /// Determines if the type is optional.
    fn is_optional(&self) -> bool;
}

/// A trait implemented on types to make the type required.
///
/// Requiring a type removes any optional qualifier.
pub trait Requireable: Copy {
    /// Makes the type required if it is optional.
    ///
    /// If the type is optional, the optional qualifier is removed.
    ///
    /// If the type is already required, this is a no-op.
    fn require(&self) -> Self;
}

/// A trait implemented on types that are coercible to other types.
pub trait Coercible {
    /// Determines if the type is coercible to the target type.
    fn is_coercible_to(&self, types: &Types, target: &Self) -> bool;
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
}

impl Requireable for PrimitiveType {
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
}

impl Requireable for Type {
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
}

impl Requireable for CompoundType {
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
    pub fn add_array(&mut self, ty: ArrayType, optional: bool) -> Type {
        ty.assert_valid(self);
        Type::Compound(CompoundType {
            definition: self.0.alloc(CompoundTypeDef::Array(ty)),
            optional,
        })
    }

    /// Adds a pair type to the type collection.
    ///
    /// # Panics
    ///
    /// Panics if the provided type contains a type definition identifier from a
    /// different types collection.
    pub fn add_pair(&mut self, ty: PairType, optional: bool) -> Type {
        ty.assert_valid(self);
        Type::Compound(CompoundType {
            definition: self.0.alloc(CompoundTypeDef::Pair(ty)),
            optional,
        })
    }

    /// Adds a map type to the type collection.
    ///
    /// # Panics
    ///
    /// Panics if the provided type contains a type definition identifier from a
    /// different types collection.
    pub fn add_map(&mut self, ty: MapType, optional: bool) -> Type {
        ty.assert_valid(self);
        Type::Compound(CompoundType {
            definition: self.0.alloc(CompoundTypeDef::Map(ty)),
            optional,
        })
    }

    /// Adds a struct type to the type collection.
    ///
    /// # Panics
    ///
    /// Panics if the provided type contains a type definition identifier from a
    /// different types collection.
    pub fn add_struct(&mut self, ty: StructType, optional: bool) -> Type {
        ty.assert_valid(self);
        Type::Compound(CompoundType {
            definition: self.0.alloc(CompoundTypeDef::Struct(ty)),
            optional,
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

        let ty = types.add_array(ArrayType::new(PrimitiveTypeKind::String), false);
        assert_eq!(
            types
                .add_array(ArrayType::new(ty), false)
                .display(&types)
                .to_string(),
            "Array[Array[String]]"
        );

        let ty = types.add_array(
            ArrayType::non_empty(PrimitiveType::optional(PrimitiveTypeKind::String)),
            true,
        );
        assert_eq!(
            types
                .add_array(ArrayType::non_empty(ty), true)
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

        let ty = types.add_array(ArrayType::new(PrimitiveTypeKind::String), false);
        assert_eq!(
            types
                .add_pair(PairType::new(ty, ty), false)
                .display(&types)
                .to_string(),
            "Pair[Array[String], Array[String]]"
        );

        let ty = types.add_array(
            ArrayType::non_empty(PrimitiveType::optional(PrimitiveTypeKind::File)),
            true,
        );
        assert_eq!(
            types
                .add_pair(PairType::new(ty, ty), true)
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

        let ty = types.add_array(ArrayType::new(PrimitiveTypeKind::String), false);
        assert_eq!(
            types
                .add_map(MapType::new(PrimitiveTypeKind::Boolean, ty), false)
                .display(&types)
                .to_string(),
            "Map[Boolean, Array[String]]"
        );

        let ty = types.add_array(
            ArrayType::non_empty(PrimitiveType::optional(PrimitiveTypeKind::File)),
            true,
        );
        assert_eq!(
            types
                .add_map(MapType::new(PrimitiveTypeKind::String, ty), true)
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
        let ty = types.add_map(
            MapType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::String),
            false,
        );
        assert!(!Type::OptionalObject.is_coercible_to(&types, &ty));

        // Object -> Map[Int, X] (not a string key)
        let ty = types.add_map(
            MapType::new(PrimitiveTypeKind::Integer, PrimitiveTypeKind::String),
            false,
        );
        assert!(!Type::Object.is_coercible_to(&types, &ty));

        // Object -> Map[String, X]?
        let ty = types.add_map(
            MapType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::String),
            true,
        );
        assert!(Type::Object.is_coercible_to(&types, &ty));

        // Object? -> Map[String, X]?
        let ty = types.add_map(
            MapType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::String),
            true,
        );
        assert!(Type::OptionalObject.is_coercible_to(&types, &ty));

        // Object? -> Map[String, X]
        let ty = types.add_map(
            MapType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::String),
            false,
        );
        assert!(!Type::OptionalObject.is_coercible_to(&types, &ty));

        // Object -> Struct
        let ty = types.add_struct(
            StructType::new("Foo", [("foo", PrimitiveTypeKind::String)]),
            false,
        );
        assert!(Type::Object.is_coercible_to(&types, &ty));

        // Object -> Struct?
        let ty = types.add_struct(
            StructType::new("Foo", [("foo", PrimitiveTypeKind::String)]),
            true,
        );
        assert!(Type::Object.is_coercible_to(&types, &ty));

        // Object? -> Struct?
        let ty = types.add_struct(
            StructType::new("Foo", [("foo", PrimitiveTypeKind::String)]),
            true,
        );
        assert!(Type::OptionalObject.is_coercible_to(&types, &ty));

        // Object? -> Struct
        let ty = types.add_struct(
            StructType::new("Foo", [("foo", PrimitiveTypeKind::String)]),
            false,
        );
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
        let type1 = types.add_array(ArrayType::new(PrimitiveTypeKind::String), false);
        let type2 = types.add_array(
            ArrayType::new(PrimitiveType::optional(PrimitiveTypeKind::File)),
            false,
        );
        assert!(type1.is_coercible_to(&types, &type2));
        assert!(!type2.is_coercible_to(&types, &type1));

        // Array[Array[X]] -> Array[Array[Y]]
        let type1 = types.add_array(ArrayType::new(type1), false);
        let type2 = types.add_array(ArrayType::new(type2), false);
        assert!(type1.is_coercible_to(&types, &type2));
        assert!(!type2.is_coercible_to(&types, &type1));

        // Array[X]+ -> Array[Y]
        let type1 = types.add_array(ArrayType::non_empty(PrimitiveTypeKind::String), false);
        let type2 = types.add_array(
            ArrayType::new(PrimitiveType::optional(PrimitiveTypeKind::File)),
            false,
        );
        assert!(type1.is_coercible_to(&types, &type2));
        assert!(!type2.is_coercible_to(&types, &type1));

        // Array[X] -> Array[X]
        let type1 = types.add_array(ArrayType::new(PrimitiveTypeKind::String), false);
        let type2 = types.add_array(ArrayType::new(PrimitiveTypeKind::String), false);
        assert!(type1.is_coercible_to(&types, &type2));
        assert!(type2.is_coercible_to(&types, &type1));

        // Array[X]? -> Array[X]?
        let type1 = types.add_array(ArrayType::new(PrimitiveTypeKind::String), true);
        let type2 = types.add_array(ArrayType::new(PrimitiveTypeKind::String), true);
        assert!(type1.is_coercible_to(&types, &type2));
        assert!(type2.is_coercible_to(&types, &type1));

        // Array[X] -> Array[X]?
        let type1 = types.add_array(ArrayType::new(PrimitiveTypeKind::String), false);
        let type2 = types.add_array(ArrayType::new(PrimitiveTypeKind::String), true);
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
        let type1 = types.add_pair(
            PairType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::String),
            false,
        );
        let type2 = types.add_pair(
            PairType::new(
                PrimitiveType::optional(PrimitiveTypeKind::File),
                PrimitiveType::optional(PrimitiveTypeKind::Directory),
            ),
            false,
        );
        assert!(type1.is_coercible_to(&types, &type2));
        assert!(!type2.is_coercible_to(&types, &type1));

        // Pair[Pair[W, X], Pair[W, X]] -> Pair[Pair[Y, Z], Pair[Y, Z]]
        let type1 = types.add_pair(PairType::new(type1, type1), false);
        let type2 = types.add_pair(PairType::new(type2, type2), false);
        assert!(type1.is_coercible_to(&types, &type2));
        assert!(!type2.is_coercible_to(&types, &type1));

        // Pair[W, X] -> Pair[W, X]
        let type1 = types.add_pair(
            PairType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::String),
            false,
        );
        let type2 = types.add_pair(
            PairType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::String),
            false,
        );
        assert!(type1.is_coercible_to(&types, &type2));
        assert!(type2.is_coercible_to(&types, &type1));

        // Pair[W, X]? -> Pair[W, X]?
        let type1 = types.add_pair(
            PairType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::String),
            true,
        );
        let type2 = types.add_pair(
            PairType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::String),
            true,
        );
        assert!(type1.is_coercible_to(&types, &type2));
        assert!(type2.is_coercible_to(&types, &type1));

        // Pair[W, X] -> Pair[W, X]?
        let type1 = types.add_pair(
            PairType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::String),
            false,
        );
        let type2 = types.add_pair(
            PairType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::String),
            true,
        );
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
        let type1 = types.add_map(
            MapType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::String),
            false,
        );
        let type2 = types.add_map(
            MapType::new(
                PrimitiveType::optional(PrimitiveTypeKind::File),
                PrimitiveType::optional(PrimitiveTypeKind::Directory),
            ),
            false,
        );
        assert!(type1.is_coercible_to(&types, &type2));
        assert!(!type2.is_coercible_to(&types, &type1));

        // Map[P, Map[W, X]] -> Map[Q, Map[Y, Z]]
        let type1 = types.add_map(MapType::new(PrimitiveTypeKind::String, type1), false);
        let type2 = types.add_map(MapType::new(PrimitiveTypeKind::Directory, type2), false);
        assert!(type1.is_coercible_to(&types, &type2));
        assert!(!type2.is_coercible_to(&types, &type1));

        // Map[W, X] -> Map[W, X]
        let type1 = types.add_map(
            MapType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::String),
            false,
        );
        let type2 = types.add_map(
            MapType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::String),
            false,
        );
        assert!(type1.is_coercible_to(&types, &type2));
        assert!(type2.is_coercible_to(&types, &type1));

        // Map[W, X]? -> Map[W, X]?
        let type1 = types.add_map(
            MapType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::String),
            true,
        );
        let type2: Type = types.add_map(
            MapType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::String),
            true,
        );
        assert!(type1.is_coercible_to(&types, &type2));
        assert!(type2.is_coercible_to(&types, &type1));

        // Map[W, X] -> Map[W, X]?
        let type1 = types.add_map(
            MapType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::String),
            false,
        );
        let type2 = types.add_map(
            MapType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::String),
            true,
        );
        assert!(type1.is_coercible_to(&types, &type2));
        assert!(!type2.is_coercible_to(&types, &type1));

        // Map[String, X] -> Struct
        let type1 = types.add_map(
            MapType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::Integer),
            false,
        );
        let type2 = types.add_struct(
            StructType::new(
                "Foo",
                [
                    ("foo", PrimitiveTypeKind::Integer),
                    ("bar", PrimitiveTypeKind::Integer),
                    ("baz", PrimitiveTypeKind::Integer),
                ],
            ),
            false,
        );
        assert!(type1.is_coercible_to(&types, &type2));

        // Map[String, X] -> Struct (mismatched fields)
        let type1 = types.add_map(
            MapType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::Integer),
            false,
        );
        let type2 = types.add_struct(
            StructType::new(
                "Foo",
                [
                    ("foo", PrimitiveTypeKind::Integer),
                    ("bar", PrimitiveTypeKind::String),
                    ("baz", PrimitiveTypeKind::Integer),
                ],
            ),
            false,
        );
        assert!(!type1.is_coercible_to(&types, &type2));

        // Map[Int, X] -> Struct
        let type1 = types.add_map(
            MapType::new(PrimitiveTypeKind::Integer, PrimitiveTypeKind::Integer),
            false,
        );
        let type2 = types.add_struct(
            StructType::new(
                "Foo",
                [
                    ("foo", PrimitiveTypeKind::Integer),
                    ("bar", PrimitiveTypeKind::Integer),
                    ("baz", PrimitiveTypeKind::Integer),
                ],
            ),
            false,
        );
        assert!(!type1.is_coercible_to(&types, &type2));

        // Map[String, X] -> Object
        let type1 = types.add_map(
            MapType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::Integer),
            false,
        );
        assert!(type1.is_coercible_to(&types, &Type::Object));

        // Map[String, X] -> Object?
        let type1 = types.add_map(
            MapType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::Integer),
            false,
        );
        assert!(type1.is_coercible_to(&types, &Type::OptionalObject));

        // Map[String, X]? -> Object?
        let type1 = types.add_map(
            MapType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::Integer),
            true,
        );
        assert!(type1.is_coercible_to(&types, &Type::OptionalObject));

        // Map[String, X]? -> Object
        let type1 = types.add_map(
            MapType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::Integer),
            true,
        );
        assert!(!type1.is_coercible_to(&types, &Type::Object));

        // Map[Integer, X] -> Object
        let type1 = types.add_map(
            MapType::new(PrimitiveTypeKind::Integer, PrimitiveTypeKind::Integer),
            false,
        );
        assert!(!type1.is_coercible_to(&types, &Type::Object));
    }

    #[test]
    fn struct_type_coercion() {
        let mut types = Types::new();

        // S -> S (identical)
        let type1 = types.add_struct(
            StructType::new(
                "Foo",
                [
                    ("foo", PrimitiveTypeKind::String),
                    ("bar", PrimitiveTypeKind::String),
                    ("baz", PrimitiveTypeKind::Integer),
                ],
            ),
            false,
        );
        let type2 = types.add_struct(
            StructType::new(
                "Foo",
                [
                    ("foo", PrimitiveTypeKind::String),
                    ("bar", PrimitiveTypeKind::String),
                    ("baz", PrimitiveTypeKind::Integer),
                ],
            ),
            false,
        );
        assert!(type1.is_coercible_to(&types, &type2));
        assert!(type2.is_coercible_to(&types, &type1));

        // S -> S?
        let type1 = types.add_struct(
            StructType::new(
                "Foo",
                [
                    ("foo", PrimitiveTypeKind::String),
                    ("bar", PrimitiveTypeKind::String),
                    ("baz", PrimitiveTypeKind::Integer),
                ],
            ),
            false,
        );
        let type2 = types.add_struct(
            StructType::new(
                "Foo",
                [
                    ("foo", PrimitiveTypeKind::String),
                    ("bar", PrimitiveTypeKind::String),
                    ("baz", PrimitiveTypeKind::Integer),
                ],
            ),
            true,
        );
        assert!(type1.is_coercible_to(&types, &type2));
        assert!(!type2.is_coercible_to(&types, &type1));

        // S? -> S?
        let type1 = types.add_struct(
            StructType::new(
                "Foo",
                [
                    ("foo", PrimitiveTypeKind::String),
                    ("bar", PrimitiveTypeKind::String),
                    ("baz", PrimitiveTypeKind::Integer),
                ],
            ),
            true,
        );
        let type2 = types.add_struct(
            StructType::new(
                "Foo",
                [
                    ("foo", PrimitiveTypeKind::String),
                    ("bar", PrimitiveTypeKind::String),
                    ("baz", PrimitiveTypeKind::Integer),
                ],
            ),
            true,
        );
        assert!(type1.is_coercible_to(&types, &type2));
        assert!(type2.is_coercible_to(&types, &type1));

        // S -> S (coercible fields)
        let type1 = types.add_struct(
            StructType::new(
                "Foo",
                [
                    ("foo", PrimitiveTypeKind::String),
                    ("bar", PrimitiveTypeKind::String),
                    ("baz", PrimitiveTypeKind::Integer),
                ],
            ),
            false,
        );
        let type2 = types.add_struct(
            StructType::new(
                "Bar",
                [
                    ("foo", PrimitiveTypeKind::File),
                    ("bar", PrimitiveTypeKind::Directory),
                    ("baz", PrimitiveTypeKind::Float),
                ],
            ),
            false,
        );
        assert!(type1.is_coercible_to(&types, &type2));
        assert!(!type2.is_coercible_to(&types, &type1));

        // S -> S (mismatched fields)
        let type1 = types.add_struct(
            StructType::new(
                "Foo",
                [
                    ("foo", PrimitiveTypeKind::String),
                    ("bar", PrimitiveTypeKind::String),
                    ("baz", PrimitiveTypeKind::Integer),
                ],
            ),
            false,
        );
        let type2 = types.add_struct(
            StructType::new("Bar", [("baz", PrimitiveTypeKind::Float)]),
            false,
        );
        assert!(!type1.is_coercible_to(&types, &type2));
        assert!(!type2.is_coercible_to(&types, &type1));

        // Struct -> Map[String, X]
        let type1 = types.add_struct(
            StructType::new(
                "Foo",
                [
                    ("foo", PrimitiveTypeKind::String),
                    ("bar", PrimitiveTypeKind::String),
                    ("baz", PrimitiveTypeKind::String),
                ],
            ),
            false,
        );
        let type2 = types.add_map(
            MapType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::String),
            false,
        );
        assert!(type1.is_coercible_to(&types, &type2));

        // Struct -> Map[String, X] (mismatched types)
        let type1 = types.add_struct(
            StructType::new(
                "Foo",
                [
                    ("foo", PrimitiveTypeKind::String),
                    ("bar", PrimitiveTypeKind::Integer),
                    ("baz", PrimitiveTypeKind::String),
                ],
            ),
            false,
        );
        let type2 = types.add_map(
            MapType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::String),
            false,
        );
        assert!(!type1.is_coercible_to(&types, &type2));

        // Struct -> Map[Int, X] (not a string key)
        let type1 = types.add_struct(
            StructType::new(
                "Foo",
                [
                    ("foo", PrimitiveTypeKind::String),
                    ("bar", PrimitiveTypeKind::String),
                    ("baz", PrimitiveTypeKind::String),
                ],
            ),
            false,
        );
        let type2 = types.add_map(
            MapType::new(PrimitiveTypeKind::Integer, PrimitiveTypeKind::String),
            false,
        );
        assert!(!type1.is_coercible_to(&types, &type2));

        // Struct -> Object
        assert!(type1.is_coercible_to(&types, &Type::Object));

        // Struct -> Object?
        assert!(type1.is_coercible_to(&types, &Type::OptionalObject));

        // Struct? -> Object?
        let type1 = types.add_struct(
            StructType::new("Foo", [("foo", PrimitiveTypeKind::String)]),
            true,
        );
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
            let ty = types.add_array(ArrayType::new(PrimitiveTypeKind::String), optional);
            let coercible = Type::Union.is_coercible_to(&types, &ty);
            assert!(coercible);

            // Union -> Pair[X, Y], Union -> Pair[X, Y]?
            let ty = types.add_pair(
                PairType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::Boolean),
                optional,
            );
            let coercible = Type::Union.is_coercible_to(&types, &ty);
            assert!(coercible);

            // Union -> Map[X, Y], Union -> Map[X, Y]?
            let ty = types.add_map(
                MapType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::Boolean),
                optional,
            );
            let coercible = Type::Union.is_coercible_to(&types, &ty);
            assert!(coercible);

            // Union -> Struct, Union -> Struct?
            let ty = types.add_struct(
                StructType::new("Foo", [("foo", PrimitiveTypeKind::String)]),
                optional,
            );
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
            let ty = types.add_array(ArrayType::new(PrimitiveTypeKind::String), optional);
            let coercible = Type::None.is_coercible_to(&types, &ty);
            if optional {
                assert!(coercible);
            } else {
                assert!(!coercible);
            }

            // None -> Pair[X, Y], None -> Pair[X, Y]?
            let ty = types.add_pair(
                PairType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::Boolean),
                optional,
            );
            let coercible = Type::None.is_coercible_to(&types, &ty);
            if optional {
                assert!(coercible);
            } else {
                assert!(!coercible);
            }

            // None -> Map[X, Y], None -> Map[X, Y]?
            let ty = types.add_map(
                MapType::new(PrimitiveTypeKind::String, PrimitiveTypeKind::Boolean),
                optional,
            );
            let coercible = Type::None.is_coercible_to(&types, &ty);
            if optional {
                assert!(coercible);
            } else {
                assert!(!coercible);
            }

            // None -> Struct, None -> Struct?
            let ty = types.add_struct(
                StructType::new("Foo", [("foo", PrimitiveTypeKind::String)]),
                optional,
            );
            let coercible = Type::None.is_coercible_to(&types, &ty);
            if optional {
                assert!(coercible);
            } else {
                assert!(!coercible);
            }
        }
    }
}
