//! Implementation of the WDL evaluation engine.

use std::sync::Arc;

use id_arena::Arena;
use id_arena::ArenaBehavior;
use id_arena::DefaultArenaBehavior;
use string_interner::DefaultStringInterner;
use wdl_analysis::types::CompoundTypeDef;
use wdl_analysis::types::Type;
use wdl_analysis::types::Types;

use crate::Array;
use crate::Coercible;
use crate::CompoundValue;
use crate::CompoundValueId;
use crate::Map;
use crate::Object;
use crate::Pair;
use crate::Struct;
use crate::Value;

/// Represents a WDL evaluation engine.
#[derive(Debug, Default)]
pub struct Engine {
    /// The engine's type collection.
    types: Types,
    /// The storage arena for compound values.
    values: Arena<CompoundValue>,
    /// The string interner used to intern string/file/directory values.
    interner: DefaultStringInterner,
}

impl Engine {
    /// Constructs a new WDL evaluation engine.
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets the engine's type collection.
    pub fn types(&self) -> &Types {
        &self.types
    }

    /// Gets a mutable reference to the engine's type collection.
    pub fn types_mut(&mut self) -> &mut Types {
        &mut self.types
    }

    /// Creates a new `String` value.
    pub fn new_string(&mut self, s: impl AsRef<str>) -> Value {
        Value::String(self.interner.get_or_intern(s))
    }

    /// Creates a new `File` value.
    pub fn new_file(&mut self, s: impl AsRef<str>) -> Value {
        Value::File(self.interner.get_or_intern(s))
    }

    /// Creates a new `Directory` value.
    pub fn new_directory(&mut self, s: impl AsRef<str>) -> Value {
        Value::Directory(self.interner.get_or_intern(s))
    }

    /// Creates a new `Pair` value.
    ///
    /// Returns `None` if either the `left` value or the `right` value did not
    /// coerce to the pair's `left` type or `right`` type, respectively.
    ///
    /// # Panics
    ///
    /// Panics if the given type is not a pair type from this engine's types
    /// collection or if any of the values are not from this engine.
    pub fn new_pair(
        &mut self,
        ty: Type,
        left: impl Into<Value>,
        right: impl Into<Value>,
    ) -> Option<Value> {
        if let Type::Compound(compound_ty) = ty {
            if let CompoundTypeDef::Pair(pair_ty) =
                self.types.type_definition(compound_ty.definition())
            {
                let left_ty = pair_ty.left_type();
                let right_ty = pair_ty.right_type();

                let left = left.into().coerce(self, left_ty)?;
                left.assert_valid(self);
                let right = right.into().coerce(self, right_ty)?;
                right.assert_valid(self);

                let id = self
                    .values
                    .alloc(CompoundValue::Pair(Pair::new(ty, left, right)));
                return Some(Value::Compound(id));
            }
        }

        panic!(
            "type `{ty}` is not a pair type",
            ty = ty.display(&self.types)
        );
    }

    /// Creates a new `Array` value for the given array type.
    ///
    /// Returns `None` if an element did not coerce to the array's element type.
    ///
    /// # Panics
    ///
    /// Panics if the given type is not an array type from this engine's types
    /// collection or if any of the values are not from this engine.
    pub fn new_array<V>(&mut self, ty: Type, elements: impl IntoIterator<Item = V>) -> Option<Value>
    where
        V: Into<Value>,
    {
        if let Type::Compound(compound_ty) = ty {
            if let CompoundTypeDef::Array(array_ty) =
                self.types.type_definition(compound_ty.definition())
            {
                let element_type = array_ty.element_type();
                let elements = elements
                    .into_iter()
                    .map(|v| {
                        let v = v.into();
                        v.assert_valid(self);
                        v.coerce(self, element_type)
                    })
                    .collect::<Option<_>>()?;
                let id = self
                    .values
                    .alloc(CompoundValue::Array(Array::new(ty, elements)));
                return Some(Value::Compound(id));
            }
        }

        panic!(
            "type `{ty}` is not an array type",
            ty = ty.display(&self.types)
        );
    }

    /// Creates a new empty `Array` value for the given array type.
    ///
    /// # Panics
    ///
    /// Panics if the given type is not an array type from this engine's types
    /// collection.
    pub fn new_empty_array(&mut self, ty: Type) -> Value {
        if let Type::Compound(compound_ty) = ty {
            if let CompoundTypeDef::Array(_) = self.types.type_definition(compound_ty.definition())
            {
                let id = self
                    .values
                    .alloc(CompoundValue::Array(Array::new(ty, Vec::new().into())));
                return Value::Compound(id);
            }
        }

        panic!(
            "type `{ty}` is not an array type",
            ty = ty.display(&self.types)
        );
    }

    /// Creates a new `Map` value.
    ///
    /// Returns `None` if an key or value did not coerce to the map's key or
    /// value type, respectively.
    ///
    /// # Panics
    ///
    /// Panics if the given type is not a map type from this engine's types
    /// collection or if any of the values are not from this engine.
    pub fn new_map<K, V>(
        &mut self,
        ty: Type,
        elements: impl IntoIterator<Item = (K, V)>,
    ) -> Option<Value>
    where
        K: Into<Value>,
        V: Into<Value>,
    {
        if let Type::Compound(compound_ty) = ty {
            if let CompoundTypeDef::Map(map_ty) =
                self.types.type_definition(compound_ty.definition())
            {
                let key_type = map_ty.key_type();
                let value_type = map_ty.value_type();

                let elements = elements
                    .into_iter()
                    .map(|(k, v)| {
                        let k = k.into();
                        k.assert_valid(self);
                        let v = v.into();
                        v.assert_valid(self);
                        Some((k.coerce(self, key_type)?, v.coerce(self, value_type)?))
                    })
                    .collect::<Option<_>>()?;
                let id = self
                    .values
                    .alloc(CompoundValue::Map(Map::new(ty, Arc::new(elements))));
                return Some(Value::Compound(id));
            }
        }

        panic!(
            "type `{ty}` is not a map type",
            ty = ty.display(&self.types)
        );
    }

    /// Creates a new `Object` value.
    ///
    /// # Panics
    ///
    /// Panics if any of the values are not from this engine.
    pub fn new_object<S, V>(&mut self, items: impl IntoIterator<Item = (S, V)>) -> Value
    where
        S: Into<String>,
        V: Into<Value>,
    {
        let id = self
            .values
            .alloc(CompoundValue::Object(Object::new(Arc::new(
                items
                    .into_iter()
                    .map(|(n, v)| {
                        let n = n.into();
                        let v = v.into();
                        v.assert_valid(self);
                        (n, v)
                    })
                    .collect(),
            ))));
        Value::Compound(id)
    }

    /// Creates a new struct value.
    ///
    /// # Panics
    ///
    /// Panics if the given type is not a struct type from this engine's types
    /// collection or if any of the values are not from this engine.
    pub fn new_struct<S, V>(
        &mut self,
        ty: Type,
        members: impl IntoIterator<Item = (S, V)>,
    ) -> Option<Value>
    where
        S: Into<String>,
        V: Into<Value>,
    {
        if let Type::Compound(compound_ty) = ty {
            if let CompoundTypeDef::Struct(_) = self.types.type_definition(compound_ty.definition())
            {
                let members = members
                    .into_iter()
                    .map(|(n, v)| {
                        let n = n.into();
                        let v = v.into();
                        v.assert_valid(self);
                        let v = v.coerce(
                            self,
                            *self
                                .types
                                .type_definition(compound_ty.definition())
                                .as_struct()
                                .expect("should be a struct")
                                .members()
                                .get(&n)?,
                        )?;
                        Some((n, v))
                    })
                    .collect::<Option<_>>()?;
                let id = self
                    .values
                    .alloc(CompoundValue::Struct(Struct::new(ty, Arc::new(members))));
                return Some(Value::Compound(id));
            }
        }

        panic!(
            "type `{ty}` is not a struct type",
            ty = ty.display(&self.types)
        );
    }

    /// Gets a compound value given its identifier.
    pub fn value(&self, id: CompoundValueId) -> &CompoundValue {
        &self.values[id]
    }

    /// Allocates a new compound value in the engine.
    pub(crate) fn alloc(&mut self, value: CompoundValue) -> CompoundValueId {
        self.values.alloc(value)
    }

    /// Gets the string interner of the engine.
    pub(crate) fn interner(&self) -> &DefaultStringInterner {
        &self.interner
    }

    /// Asserts that the given id comes from this engine's values arena.
    pub(crate) fn assert_same_arena(&self, id: CompoundValueId) {
        assert!(
            DefaultArenaBehavior::arena_id(id)
                == DefaultArenaBehavior::arena_id(self.values.next_id()),
            "id comes from a different values arena"
        );
    }
}
