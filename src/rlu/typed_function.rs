use std::marker::PhantomData;

use rlua::{Context, FromLua, FromLuaMulti, Function, ToLua, ToLuaMulti, Value};

use crate::{TealMultiValue, ToTypename};

///A typed wrapper around [rlua::Function]
#[derive(Debug)]
pub struct TypedFunction<'lua, Params, Response>
where
    Params: TealMultiValue,
    Response: TealMultiValue,
{
    inner_function: rlua::Function<'lua>,
    _p: PhantomData<Params>,
    _r: PhantomData<Response>,
}
impl<'lua, Params, Response> rlua::FromLua<'lua> for TypedFunction<'lua, Params, Response>
where
    Params: TealMultiValue,
    Response: TealMultiValue,
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
    Params: TealMultiValue,
    Response: TealMultiValue,
{
    #[allow(clippy::wrong_self_convention)]
    fn to_lua(self, _: Context<'lua>) -> rlua::Result<Value<'lua>> {
        Ok(Value::Function(self.inner_function))
    }
}
impl<'lua, Params, Response> ToTypename for TypedFunction<'lua, Params, Response>
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
impl<'lua, Params, Response> TypedFunction<'lua, Params, Response>
where
    Params: ToLuaMulti<'lua> + TealMultiValue,
    Response: FromLuaMulti<'lua> + TealMultiValue,
{
    ///Same as [rlua::Function::call](rlua::Function#method.call). Calls the function with the given parameters.
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
impl<'lua, Params, Response> From<TypedFunction<'lua, Params, Response>> for Function<'lua>
where
    Params: TealMultiValue,
    Response: TealMultiValue,
{
    fn from(fun: TypedFunction<'lua, Params, Response>) -> Self {
        fun.inner_function
    }
}

impl<'lua, Params, Response> TypedFunction<'lua, Params, Response>
where
    Params: FromLuaMulti<'lua> + TealMultiValue,
    Response: ToLuaMulti<'lua> + TealMultiValue,
{
    ///make a typed function directly from a Rust one.
    pub fn from_rust<Func: 'static + Send + Fn(Context<'lua>, Params) -> rlua::Result<Response>>(
        func: Func,
        context: Context<'lua>,
    ) -> rlua::Result<Self> {
        Ok(Self {
            inner_function: context.create_function(func)?,
            _p: PhantomData,
            _r: PhantomData,
        })
    }
    ///make a typed function directly from a Rust one.
    pub fn from_rust_mut<
        Func: 'static + Send + FnMut(Context<'lua>, Params) -> rlua::Result<Response>,
    >(
        func: Func,
        context: Context<'lua>,
    ) -> rlua::Result<Self> {
        Ok(Self {
            inner_function: context.create_function_mut(func)?,
            _p: PhantomData,
            _r: PhantomData,
        })
    }
}
impl<'lua, Params, Response> TypedFunction<'lua, Params, Response>
where
    Params: ToLuaMulti<'lua> + TealMultiValue,
    Response: TealMultiValue,
{
    ///call a function without trying to convert it to a rust type.
    pub fn call_as_lua(&self, params: Params) -> rlua::Result<rlua::Value<'lua>> {
        self.inner_function.call(params)
    }
}
