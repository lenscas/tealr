use std::marker::PhantomData;

use mlua::{AnyUserData, Error, Lua, ToLua, UserData};

use crate::{TypeBody, TypeName};

/// A userdata which can be used as a static proxy
pub trait StaticUserdata: UserData + 'static {}
impl<T: UserData + 'static> StaticUserdata for T {}

/// A newtype storing userdata created via [`mlua::Lua::create_proxy`].
///
/// if `T` implements TypeName or TypeBody the implementations are forwarded to this type.
pub struct UserDataProxy<'lua, T: StaticUserdata> {
    user_data: AnyUserData<'lua>,
    ph_: PhantomData<T>,
}

impl<'lua, T: StaticUserdata> UserDataProxy<'lua, T> {
    /// Creates a new UserDataProxy
    pub fn new(lua: &'lua Lua) -> Result<Self, Error> {
        Ok(Self {
            user_data: lua.create_proxy::<T>()?,
            ph_: Default::default(),
        })
    }
}

impl<T: StaticUserdata + TypeName> TypeName for UserDataProxy<'_, T> {
    fn get_type_parts() -> std::borrow::Cow<'static, [crate::NamePart]> {
        T::get_type_parts()
    }
}

impl<T: StaticUserdata + TypeBody> TypeBody for UserDataProxy<'_, T> {
    fn get_type_body() -> crate::TypeGenerator {
        T::get_type_body()
    }
}

impl<'lua, T: StaticUserdata> ToLua<'lua> for UserDataProxy<'lua, T> {
    fn to_lua(self, lua: &'lua Lua) -> mlua::Result<mlua::Value<'lua>> {
        self.user_data.to_lua(lua)
    }
}
