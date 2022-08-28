use std::{borrow::Cow, marker::PhantomData};

use rlua::{Context, FromLua, FromLuaMulti, Function, ToLua, ToLuaMulti, Value};

use crate::{NamePart, TealMultiValue, TypeName};

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
    Params: ToLuaMulti<'lua> + TealMultiValue,
    Response: FromLuaMulti<'lua> + TealMultiValue,
{
    #[allow(clippy::wrong_self_convention)]
    fn to_lua(self, _: Context<'lua>) -> rlua::Result<Value<'lua>> {
        Ok(Value::Function(self.inner_function))
    }
}
impl<'lua, Params, Response> TypeName for TypedFunction<'lua, Params, Response>
where
    Params: TealMultiValue,
    Response: TealMultiValue,
{
    fn get_type_parts() -> Cow<'static, [crate::NamePart]> {
        let params = Params::get_types();
        let returns = Response::get_types();
        let mut v = vec!["function(".into()];
        v.extend(params);
        v.push("):(".into());
        v.extend(returns);
        v.push(")".into());
        Cow::Owned(v)
    }
    fn get_type_parts_as_global() -> Cow<'static, [NamePart]> {
        let mut generics = Vec::new();
        let params = Params::get_types();
        let returns = Response::get_types();
        params
            .iter()
            .chain(returns.iter())
            .for_each(|param| match param {
                NamePart::Symbol(_) => (),
                NamePart::Type(x) => {
                    if x.type_kind.is_generic() && !generics.contains(x) {
                        generics.push(x.clone())
                    }
                }
            });
        if generics.is_empty() {
            return Self::get_type_parts();
        }
        let mut type_name = vec!["function<".into()];
        let last = generics.len() - 1;
        for (key, generic) in generics.into_iter().enumerate() {
            type_name.push(NamePart::Type(generic));
            if key != last {
                type_name.push(",".into())
            }
        }
        type_name.push(">(".into());
        type_name.extend(params);
        type_name.push("):(".into());
        type_name.extend(returns);
        type_name.push(")".into());
        Cow::Owned(type_name)
    }
    fn collect_children(generics: &mut Vec<crate::TealType>) {
        let params = Params::get_types()
            .into_iter()
            .chain(Response::get_types().into_iter())
            .filter_map(|v| match v {
                NamePart::Symbol(_) => None,
                NamePart::Type(x) => Some(x),
            });

        generics.extend(params);
    }
    fn get_type_kind() -> crate::KindOfType {
        crate::KindOfType::Builtin
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
