use rlua::{Context, FromLua, FromLuaMulti, Function, ToLua, ToLuaMulti, Value};

use crate::{teal_data_methods::TealDataMethods, TealMultiValue};

///This is the teal version of [UserData](rlua::UserData).
pub trait TealData: Sized {
    ///same as [UserData::add_methods](rlua::UserData::add_methods).
    ///Refer to its documentation on how to use it.
    ///
    ///only diffrence is that it takes a [TealDataMethods](crate::TealDataMethods),
    ///which is the teal version of [UserDataMethods](rlua::UserDataMethods)
    fn add_methods<'lua, T: TealDataMethods<'lua, Self>>(_methods: &mut T) {}
}

macro_rules! impl_teal_data {
    ($teal_type:literal $current_type:ty) => {
        impl TypeRepresentation for $current_type {
            fn get_type_name() -> Cow<'static, str> {
                Cow::from($teal_type)
            }
            fn is_external() -> bool {
                false
            }
        }
    };
    ($teal_type:literal $current_type:ty,$($types:ty),*) => {
        impl_teal_data!($teal_type $current_type);
        impl_teal_data!($teal_type $($types),+);
    };
}

///A trait to collect the required type information like the name of the type.
pub trait TypeRepresentation {
    ///returns the type name as how it should show up in the generated `.d.tl` file
    fn get_type_name() -> Cow<'static, str>;
    ///This method tells the generator that the type will always be available
    ///This is pretty much only the case for native lua/teal types like `number`
    ///
    ///***DO NOT*** overwrite it unless you are ***ABSOLUTLY*** sure you need to.
    ///The default is correct for 99.999% of the cases.
    fn is_external() -> bool {
        true
    }
}

use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    marker::PhantomData,
};

impl_teal_data!("boolean" bool);
impl_teal_data!("string" String,std::ffi::CString,bstr::BString,&str,&std::ffi::CStr,&bstr::BStr);
impl_teal_data!("number" i8,u8,u16,i16,u32,i32,u64,i64,u128,i128,isize,usize,f32,f64);

impl<'lua> TypeRepresentation for rlua::Value<'lua> {
    fn get_type_name() -> Cow<'static, str> {
        Cow::from("any")
    }
    fn is_external() -> bool {
        false
    }
}

impl<'lua> TypeRepresentation for rlua::Table<'lua> {
    fn get_type_name() -> Cow<'static, str> {
        Cow::from("{any:any}")
    }
    fn is_external() -> bool {
        false
    }
}
impl<'lua> TypeRepresentation for rlua::String<'lua> {
    fn get_type_name() -> Cow<'static, str> {
        Cow::from("string")
    }
    fn is_external() -> bool {
        false
    }
}
impl<'lua> TypeRepresentation for rlua::Function<'lua> {
    fn get_type_name() -> Cow<'static, str> {
        Cow::from("function(...:any):any...")
    }
    fn is_external() -> bool {
        false
    }
}

impl<T: TypeRepresentation> TypeRepresentation for Vec<T> {
    fn get_type_name() -> Cow<'static, str> {
        Cow::from(format!("{{{}}}", T::get_type_name()))
    }
    fn is_external() -> bool {
        false
    }
}

impl<T: TypeRepresentation> TypeRepresentation for Option<T> {
    fn get_type_name() -> Cow<'static, str> {
        T::get_type_name()
    }
    fn is_external() -> bool {
        false
    }
}

impl<K: TypeRepresentation, V: TypeRepresentation> TypeRepresentation for HashMap<K, V> {
    fn get_type_name() -> Cow<'static, str> {
        Cow::from(format!("{{{}:{}}}", K::get_type_name(), V::get_type_name()))
    }
    fn is_external() -> bool {
        false
    }
}
impl<K: TypeRepresentation, V: TypeRepresentation> TypeRepresentation for BTreeMap<K, V> {
    fn get_type_name() -> Cow<'static, str> {
        Cow::from(format!("{{{}:{}}}", K::get_type_name(), V::get_type_name()))
    }
    fn is_external() -> bool {
        false
    }
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
impl<'lua, Params, Response> TypeRepresentation for TypedFunction<'lua, Params, Response>
where
    Params: ToLuaMulti<'lua> + TealMultiValue,
    Response: FromLuaMulti<'lua> + TealMultiValue,
{
    fn get_type_name() -> Cow<'static, str> {
        let params = Params::get_types()
            .into_iter()
            .map(|v| v.name)
            .collect::<Vec<_>>()
            .join(",");
        let output = Response::get_types()
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
