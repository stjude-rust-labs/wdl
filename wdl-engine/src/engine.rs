//! Implementation of the WDL evaluation engine.

use std::sync::Arc;

use id_arena::Arena;
use id_arena::ArenaBehavior;
use id_arena::DefaultArenaBehavior;
use string_interner::DefaultStringInterner;
use wdl_analysis::types::CompoundTypeDef;
use wdl_analysis::types::Type;
use wdl_analysis::types::Types;

use crate::Coercible;
use crate::CompoundValue;
use crate::CompoundValueId;
use crate::Value;

/// Represents a WDL evaluation engine.
#[derive(Debug, Default)]
pub struct Engine {
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
    /// Panics if the given type is not a pair type.
    pub fn new_pair(
        &mut self,
        types: &Types,
        ty: Type,
        left: impl Into<Value>,
        right: impl Into<Value>,
    ) -> Option<Value> {
        if let Type::Compound(compound_ty) = ty {
            if let CompoundTypeDef::Pair(pair_ty) = types.type_definition(compound_ty.definition())
            {
                let left = left.into().coerce(self, types, pair_ty.left_type())?;
                left.assert_valid(self);
                let right = right.into().coerce(self, types, pair_ty.right_type())?;
                right.assert_valid(self);
                let id = self.values.alloc(CompoundValue::Pair(left, right));
                return Some(Value::Compound(ty, id));
            }
        }

        panic!("type `{ty}` is not a pair type", ty = ty.display(types));
    }

    /// Creates a new `Array` value for the given array type.
    ///
    /// Returns `None` if an element did not coerce to the array's element type.
    ///
    /// # Panics
    ///
    /// Panics if the given type is not an array type.
    pub fn new_array<V>(
        &mut self,
        types: &Types,
        ty: Type,
        elements: impl IntoIterator<Item = V>,
    ) -> Option<Value>
    where
        V: Into<Value>,
    {
        if let Type::Compound(compound_ty) = ty {
            if let CompoundTypeDef::Array(array_ty) =
                types.type_definition(compound_ty.definition())
            {
                let elements = elements
                    .into_iter()
                    .map(|v| {
                        let v = v.into();
                        v.assert_valid(self);
                        v.coerce(self, types, array_ty.element_type())
                    })
                    .collect::<Option<_>>()?;
                let id = self.values.alloc(CompoundValue::Array(elements));
                return Some(Value::Compound(ty, id));
            }
        }

        panic!("type `{ty}` is not an array type", ty = ty.display(types));
    }

    /// Creates a new empty `Array` value for the given array type.
    ///
    /// # Panics
    ///
    /// Panics if the given type is not an array type.
    pub fn new_empty_array(&mut self, types: &Types, ty: Type) -> Value {
        if let Type::Compound(compound_ty) = ty {
            if let CompoundTypeDef::Array(_) = types.type_definition(compound_ty.definition()) {
                let id = self.values.alloc(CompoundValue::Array(Vec::new().into()));
                return Value::Compound(ty, id);
            }
        }

        panic!("type `{ty}` is not an array type", ty = ty.display(types));
    }

    /// Creates a new `Map` value.
    ///
    /// Returns `None` if an key or value did not coerce to the map's key or
    /// value type, respectively.
    ///
    /// # Panics
    ///
    /// Panics if the given type is not an array type.
    pub fn new_map<K, V>(
        &mut self,
        types: &Types,
        ty: Type,
        elements: impl IntoIterator<Item = (K, V)>,
    ) -> Option<Value>
    where
        K: Into<Value>,
        V: Into<Value>,
    {
        if let Type::Compound(compound_ty) = ty {
            if let CompoundTypeDef::Map(map_ty) = types.type_definition(compound_ty.definition()) {
                let elements = elements
                    .into_iter()
                    .map(|(k, v)| {
                        let k = k.into();
                        k.assert_valid(self);
                        let v = v.into();
                        v.assert_valid(self);
                        Some((
                            k.coerce(self, types, map_ty.key_type())?,
                            v.coerce(self, types, map_ty.value_type())?,
                        ))
                    })
                    .collect::<Option<_>>()?;
                let id = self.values.alloc(CompoundValue::Map(Arc::new(elements)));
                return Some(Value::Compound(ty, id));
            }
        }

        panic!("type `{ty}` is not a map type", ty = ty.display(types));
    }

    /// Creates a new `Object` value.
    pub fn new_object<S, V>(&mut self, items: impl IntoIterator<Item = (S, V)>) -> Value
    where
        S: Into<String>,
        V: Into<Value>,
    {
        let id = self.values.alloc(CompoundValue::Object(Arc::new(
            items
                .into_iter()
                .map(|(n, v)| {
                    let n = n.into();
                    let v = v.into();
                    v.assert_valid(self);
                    (n, v)
                })
                .collect(),
        )));
        Value::Compound(Type::Object, id)
    }

    /// Creates a new struct value.
    pub fn new_struct<S, V>(
        &mut self,
        types: &Types,
        ty: Type,
        members: impl IntoIterator<Item = (S, V)>,
    ) -> Option<Value>
    where
        S: Into<String>,
        V: Into<Value>,
    {
        if let Type::Compound(compound_ty) = ty {
            if let CompoundTypeDef::Struct(struct_ty) =
                types.type_definition(compound_ty.definition())
            {
                let members = members
                    .into_iter()
                    .map(|(n, v)| {
                        let n = n.into();
                        let v = v.into();
                        v.assert_valid(self);
                        let v = v.coerce(self, types, *struct_ty.members().get(&n)?)?;
                        Some((n, v))
                    })
                    .collect::<Option<_>>()?;
                let id = self.values.alloc(CompoundValue::Struct(
                    compound_ty.definition(),
                    Arc::new(members),
                ));
                return Some(Value::Compound(ty, id));
            }
        }

        panic!("type `{ty}` is not a struct type", ty = ty.display(types));
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
