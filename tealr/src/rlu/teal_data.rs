use std::{borrow::Cow, marker::PhantomData};

use rlua::{Context, FromLua, FromLuaMulti, Function, ToLua, ToLuaMulti, Value};

use super::TealDataMethods;
use crate::{Direction, TealMultiValue, TypeName};

///This is the teal version of [UserData](rlua::UserData).
pub trait TealData: Sized {
    ///same as [UserData::add_methods](rlua::UserData::add_methods).
    ///Refer to its documentation on how to use it.
    ///
    ///only difference is that it takes a [TealDataMethods](crate::TealDataMethods),
    ///which is the teal version of [UserDataMethods](rlua::UserDataMethods)
    fn add_methods<'lua, T: TealDataMethods<'lua, Self>>(_methods: &mut T) {}
}

///A typed wrapper around [rlua::Function]
#[derive(Debug)]
pub struct TypedFunction<'lua, Params, Response>
where
    Params: ToLuaMulti<'lua> + TealMultiValue,
    Response: FromLuaMulti<'lua> + TealMultiValue,
{
    inner_function: rlua::Function<'lua>,
    _p: PhantomData<Params>,
    _r: PhantomData<Response>,
}
impl<'lua, Params, Response> rlua::FromLua<'lua> for TypedFunction<'lua, Params, Response>
where
    Params: ToLuaMulti<'lua> + TealMultiValue,
    Response: FromLuaMulti<'lua> + TealMultiValue,
{
    fn from_lua(lua_value: rlua::Value<'lua>, lua: rlua::Context<'lua>) -> rlua::Result<Self> {
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
    fn to_lua(self, _: Context<'lua>) -> rlua::Result<Value<'lua>> {
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
}
impl<'lua, Params, Response> TypedFunction<'lua, Params, Response>
where
    Params: ToLuaMulti<'lua> + TealMultiValue,
    Response: FromLuaMulti<'lua> + TealMultiValue,
{
    ///same as [rlua::Function::call](rlua::Function#method.call). Calls the function with the given parameters.
    pub fn call(&self, params: Params) -> rlua::Result<Response> {
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
