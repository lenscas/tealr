use std::marker::PhantomData;

use mlua::{FromLua, FromLuaMulti, Function, IntoLua, IntoLuaMulti, Lua, Value};

use crate::{TealMultiValue, ToTypename};

///A typed wrapper around [mlua::Function]
#[derive(Debug)]
pub struct TypedFunction<Params, Response>
where
    Params: TealMultiValue,
    Response: TealMultiValue,
{
    inner_function: mlua::Function,
    _p: PhantomData<Params>,
    _r: PhantomData<Response>,
}

impl<Params, Response> mlua::FromLua for TypedFunction<Params, Response>
where
    Params: TealMultiValue,
    Response: TealMultiValue,
{
    fn from_lua(lua_value: mlua::Value, lua: &Lua) -> mlua::Result<Self> {
        Ok(Self {
            inner_function: FromLua::from_lua(lua_value, lua)?,
            _p: PhantomData,
            _r: PhantomData,
        })
    }
}

impl<Params, Response> IntoLua for TypedFunction<Params, Response>
where
    Params: TealMultiValue,
    Response: TealMultiValue,
{
    #[allow(clippy::wrong_self_convention)]
    fn into_lua(self, _: &Lua) -> mlua::Result<Value> {
        Ok(Value::Function(self.inner_function))
    }
}
impl<Params, Response> ToTypename for TypedFunction<Params, Response>
where
    Params: TealMultiValue,
    Response: TealMultiValue,
{
    fn to_typename() -> crate::Type {
        crate::Type::Function(crate::FunctionRepresentation {
            params: Params::get_types_as_params(),
            returns: Response::get_types(),
        })
    }
}
impl<Params, Response> TypedFunction<Params, Response>
where
    Params: IntoLuaMulti + TealMultiValue,
    Response: FromLuaMulti + TealMultiValue,
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
impl<Params, Response> Clone for TypedFunction<Params, Response>
where
    Params: TealMultiValue,
    Response: TealMultiValue,
{
    fn clone(&self) -> Self {
        Self {
            inner_function: self.inner_function.clone(),
            _p: PhantomData,
            _r: PhantomData,
        }
    }
}
impl<Params, Response> From<TypedFunction<Params, Response>> for Function
where
    Params: TealMultiValue,
    Response: TealMultiValue,
{
    fn from(fun: TypedFunction<Params, Response>) -> Self {
        fun.inner_function
    }
}
impl<Params, Response> TypedFunction<Params, Response>
where
    Params: FromLuaMulti + TealMultiValue,
    Response: IntoLuaMulti + TealMultiValue,
{
    ///make a typed function directly from a Rust one.
    pub fn from_rust<
        Func: 'static + crate::mlu::MaybeSend + Fn(&Lua, Params) -> mlua::Result<Response>,
    >(
        func: Func,
        lua: &Lua,
    ) -> mlua::Result<Self> {
        Ok(Self {
            inner_function: lua.create_function(func)?,
            _p: PhantomData,
            _r: PhantomData,
        })
    }

    ///make a typed function directly from a Rust one.
    pub fn from_rust_mut<
        Func: 'static + crate::mlu::MaybeSend + FnMut(&Lua, Params) -> mlua::Result<Response>,
    >(
        func: Func,
        lua: &Lua,
    ) -> mlua::Result<Self> {
        Ok(Self {
            inner_function: lua.create_function_mut(func)?,
            _p: PhantomData,
            _r: PhantomData,
        })
    }
}
impl<Params, Response> TypedFunction<Params, Response>
where
    Params: IntoLuaMulti + TealMultiValue,
    Response: TealMultiValue,
{
    ///call a function without trying to convert it to a rust type.
    pub fn call_as_lua(&self, params: Params) -> mlua::Result<mlua::Value> {
        self.inner_function.call(params)
    }
}
