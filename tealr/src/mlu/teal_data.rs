use std::{borrow::Cow, marker::PhantomData};

use mlua::{FromLua, FromLuaMulti, Function, Lua, ToLua, ToLuaMulti, Value};

use super::TealDataMethods;
use crate::{Direction, TealMultiValue, TypeName};

///This is the teal version of [UserData](mlua::UserData).
pub trait TealData: Sized {
    ///same as [UserData::add_methods](mlua::UserData::add_methods).
    ///Refer to its documentation on how to use it.
    ///
    ///only difference is that it takes a [TealDataMethods](crate::mlu::TealDataMethods),
    ///which is the teal version of [UserDataMethods](mlua::UserDataMethods)
    fn add_methods<'lua, T: TealDataMethods<'lua, Self>>(_methods: &mut T) {}
}

///A typed wrapper around [mlua::Function]
#[derive(Debug)]
pub struct TypedFunction<'lua, Params, Response>
where
    Params: ToLuaMulti<'lua> + TealMultiValue,
    Response: FromLuaMulti<'lua> + TealMultiValue,
{
    inner_function: mlua::Function<'lua>,
    _p: PhantomData<Params>,
    _r: PhantomData<Response>,
}
impl<'lua, Params, Response> mlua::FromLua<'lua> for TypedFunction<'lua, Params, Response>
where
    Params: ToLuaMulti<'lua> + TealMultiValue,
    Response: FromLuaMulti<'lua> + TealMultiValue,
{
    fn from_lua(lua_value: mlua::Value<'lua>, lua: &'lua Lua) -> mlua::Result<Self> {
        Ok(Self {
            inner_function: FromLua::from_lua(lua_value, lua)?,
            _p: PhantomData,
            _r: PhantomData,
        })
    }
}

impl<'lua, Params, Response> ToLua<'lua> for TypedFunction<'lua, Params, Response>
where
    Params: ToLuaMulti<'lua> + TealMultiValue,
    Response: FromLuaMulti<'lua> + TealMultiValue,
{
    #[allow(clippy::wrong_self_convention)]
    fn to_lua(self, _: &'lua Lua) -> mlua::Result<Value<'lua>> {
        Ok(Value::Function(self.inner_function))
    }
}
impl<'lua, Params, Response> TypeName for TypedFunction<'lua, Params, Response>
where
    Params: ToLuaMulti<'lua> + TealMultiValue,
    Response: FromLuaMulti<'lua> + TealMultiValue,
{
    fn get_type_name(_: Direction) -> Cow<'static, str> {
        let params = Params::get_types(Direction::FromLua)
            .into_iter()
            .map(|v| v.name)
            .collect::<Vec<_>>()
            .join(",");
        let output = Response::get_types(Direction::ToLua)
            .into_iter()
            .map(|v| v.name)
            .collect::<Vec<_>>()
            .join(",");
        Cow::Owned(format!("function({}):({})", params, output))
    }

    fn collect_children(generics: &mut Vec<crate::TealType>) {
        let params = Params::get_types(Direction::FromLua)
            .into_iter()
            .chain(Response::get_types(Direction::ToLua));
        generics.extend(params);
    }
}
impl<'lua, Params, Response> TypedFunction<'lua, Params, Response>
where
    Params: ToLuaMulti<'lua> + TealMultiValue,
    Response: FromLuaMulti<'lua> + TealMultiValue,
{
    ///Same as [mlua::Function::call](mlua::Function#method.call). Calls the function with the given parameters.
    pub fn call(&self, params: Params) -> mlua::Result<Response> {
        self.inner_function.call(params)
    }
    ///Calls the function with the given parameters. Panics if something has gone wrong.
    pub fn force_call(&self, params: Params) -> Response {
        self.inner_function.call(params).unwrap()
    }
}
impl<'lua, Params, Response> Clone for TypedFunction<'lua, Params, Response>
where
    Params: ToLuaMulti<'lua> + TealMultiValue,
    Response: FromLuaMulti<'lua> + TealMultiValue,
{
    fn clone(&self) -> Self {
        Self {
            inner_function: self.inner_function.clone(),
            _p: PhantomData,
            _r: PhantomData,
        }
    }
}
impl<'lua, Params, Response> From<TypedFunction<'lua, Params, Response>> for Function<'lua>
where
    Params: ToLuaMulti<'lua> + TealMultiValue,
    Response: FromLuaMulti<'lua> + TealMultiValue,
{
    fn from(fun: TypedFunction<'lua, Params, Response>) -> Self {
        fun.inner_function
    }
}
