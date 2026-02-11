use std::ops::Deref;

use mlua::{Table, Value};
use serde::{Deserialize, Serialize};

use crate::{ToTypename, TypeBody};

/// Languages that compile to lua, as well as lua language server have inheritance.
/// This trait allows you to model this properly.
pub trait Extendable: Sized {
    ///extend the given value with the contents of this type
    fn extend(self, to_extend: &mut impl Extend, lua: &mlua::Lua) -> mlua::Result<()>;
    ///recreates self from the given value
    fn from_part(from: &impl BackMerger, lua: &mlua::Lua) -> mlua::Result<Self>;
}

/// Used by the Extendable trait, represents the lua value that will be extended
pub trait Extend {
    /// adds a value to the lua value
    fn add(
        &mut self,
        name: impl mlua::IntoLua,
        value: impl mlua::IntoLua,
    ) -> mlua::Result<&mut Self>;
    ///checks if a field with the given name already exists
    fn has_field(&self, name: impl mlua::IntoLua) -> mlua::Result<bool>;
}
impl Extend for mlua::Table {
    fn add(
        &mut self,
        name: impl mlua::IntoLua,
        value: impl mlua::IntoLua,
    ) -> mlua::Result<&mut Self> {
        self.set(name, value)?;
        Ok(self)
    }

    fn has_field(&self, name: impl mlua::IntoLua) -> mlua::Result<bool> {
        self.contains_key(name)
    }
}

///Used by the Extendable trait. Represents the value that will be read from to recreate the value in rust
pub trait BackMerger {
    ///check if the given field has a value
    fn has_field(&self, name: impl mlua::IntoLua) -> mlua::Result<bool>;
    ///gets the value of the given field
    fn get<T: mlua::FromLua>(&self, name: impl mlua::IntoLua) -> mlua::Result<T>;
    ///returns an iterator over all the existing fields
    fn get_existing_fields(&self) -> impl Iterator<Item = mlua::Result<mlua::Value>>;
    ///turns this value back into a raw lua value
    fn to_value(&self) -> mlua::Value;
}
impl BackMerger for Table {
    fn has_field(&self, name: impl mlua::IntoLua) -> mlua::Result<bool> {
        self.contains_key(name)
    }

    fn get<T: mlua::FromLua>(&self, name: impl mlua::IntoLua) -> mlua::Result<T> {
        self.get(name)
    }
    fn get_existing_fields(&self) -> impl Iterator<Item = mlua::Result<mlua::Value>> {
        self.pairs::<mlua::Value, mlua::Value>()
            .map(|v| v.map(|x| x.0))
    }
    fn to_value(&self) -> mlua::Value {
        Value::Table(self.clone())
    }
}

impl<T: mlua::IntoLua + mlua::FromLua> Extendable for Vec<T> {
    fn extend(self, to_extend: &mut impl Extend, _: &mlua::Lua) -> mlua::Result<()> {
        for (key, value) in self.into_iter().enumerate() {
            to_extend.add(key, value)?;
        }
        Ok(())
    }
    fn from_part(from: &impl BackMerger, _: &mlua::Lua) -> mlua::Result<Self> {
        (1..).map(|v| from.get(v)).collect::<mlua::Result<Vec<T>>>()
    }
}

/// Allows you to use any type that can be converted to a lua table as an extendable
///
/// This converts the type first to a lua table and then iterators over it through .pairs()
#[derive(Clone, Serialize, Deserialize)]
pub struct Extender<T>(pub T);
impl<T: mlua::IntoLua + mlua::FromLua> Extendable for Extender<T> {
    fn extend(self, to_extend: &mut impl Extend, lua: &mlua::Lua) -> mlua::Result<()> {
        let lua_value = self.0.into_lua(lua)?;
        let table = lua_value
            .as_table()
            .ok_or(mlua::Error::FromLuaConversionError {
                from: lua_value.type_name(),
                to: "table".into(),
                message: Some("The extender can only work with table values.".into()),
            })?;
        for pairs in table.pairs::<Value, Value>() {
            let (key, value) = pairs?;
            to_extend.add(key, value)?;
        }
        Ok(())
    }
    fn from_part(from: &impl BackMerger, lua: &mlua::Lua) -> mlua::Result<Self> {
        T::from_lua(from.to_value(), lua).map(Extender)
    }
}
impl<T> Deref for Extender<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: crate::ToTypename> ToTypename for Extender<T> {
    fn to_typename() -> crate::Type {
        T::to_typename()
    }
}
impl<T: mlua::IntoLua> mlua::IntoLua for Extender<T> {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<Value> {
        self.0.into_lua(lua)
    }
}
impl<T: mlua::FromLua> mlua::FromLua for Extender<T> {
    fn from_lua(lua_value: Value, lua: &mlua::Lua) -> mlua::Result<Self> {
        Ok(Extender(T::from_lua(lua_value, lua)?))
    }
}

impl<T: TypeBody> TypeBody for Extender<T> {
    fn get_type_body() -> crate::TypeGenerator {
        T::get_type_body()
    }
}
