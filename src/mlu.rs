///this module holds some pre made types that can be used to create generics.
pub mod generics;
mod named_parameters;
mod picker_macro;
pub(crate) mod teal_data;
mod teal_data_fields;
pub(crate) mod teal_data_macros;
pub(crate) mod teal_data_methods;
mod typed_function;
/// Module containing functionality to do with user data proxies
pub mod user_data_proxy;
pub(crate) mod user_data_wrapper;
mod variadics;
use std::borrow::Cow;

pub use self::{
    picker_macro::FromLuaExact,
    teal_data::TealData,
    teal_data_methods::{set_global_env, ExportInstances, InstanceCollector, TealDataMethods},
    typed_function::TypedFunction,
    user_data_proxy::UserDataProxy,
    user_data_wrapper::UserDataWrapper,
};
pub use crate::{
    create_generic_mlua as create_generic, create_union_mlua as create_union,
    mlua_create_named_parameters as create_named_parameters,
};
use crate::{ToTypename, Type};
pub use mlua;
use mlua::{UserDataRef, UserDataRefMut};
pub use teal_data_fields::TealDataFields;

pub(crate) fn get_meta_name(name: mlua::MetaMethod) -> Cow<'static, str> {
    use mlua::MetaMethod;
    match name {
        MetaMethod::Add => Cow::Borrowed("__add"),
        MetaMethod::Sub => Cow::Borrowed("__sub"),
        MetaMethod::Mul => Cow::Borrowed("__mul"),
        MetaMethod::Div => Cow::Borrowed("__div"),
        MetaMethod::Mod => Cow::Borrowed("__mod"),
        MetaMethod::Pow => Cow::Borrowed("__pow"),
        MetaMethod::Unm => Cow::Borrowed("__unm"),
        #[cfg(any(feature = "mlua_lua54", feature = "mlua_lua53"))]
        MetaMethod::IDiv => Cow::Borrowed("__idiv"),
        #[cfg(any(feature = "mlua_lua54", feature = "mlua_lua53"))]
        MetaMethod::BAnd => Cow::Borrowed("__band"),
        #[cfg(any(feature = "mlua_lua54", feature = "mlua_lua53"))]
        MetaMethod::BOr => Cow::Borrowed("__bor"),
        #[cfg(any(feature = "mlua_lua54", feature = "mlua_lua53"))]
        MetaMethod::BXor => Cow::Borrowed("__bxor"),
        #[cfg(any(feature = "mlua_lua54", feature = "mlua_lua53"))]
        MetaMethod::BNot => Cow::Borrowed("__bnot"),
        #[cfg(any(feature = "mlua_lua54", feature = "mlua_lua53"))]
        MetaMethod::Shl => Cow::Borrowed("__shl"),
        #[cfg(any(feature = "mlua_lua54", feature = "mlua_lua53"))]
        MetaMethod::Shr => Cow::Borrowed("__shr"),
        MetaMethod::Concat => Cow::Borrowed("__concat"),
        MetaMethod::Len => Cow::Borrowed("__len"),
        MetaMethod::Eq => Cow::Borrowed("__eq"),
        MetaMethod::Lt => Cow::Borrowed("__lt"),
        MetaMethod::Le => Cow::Borrowed("__le"),
        MetaMethod::Index => Cow::Borrowed("__index"),
        MetaMethod::NewIndex => Cow::Borrowed("__newindex"),
        MetaMethod::Call => Cow::Borrowed("__call"),
        MetaMethod::ToString => Cow::Borrowed("__tostring"),
        #[cfg(any(
            feature = "mlua_lua54",
            feature = "mlua_lua53",
            feature = "mlua_lua52",
            feature = "mlua_luajit52"
        ))]
        MetaMethod::Pairs => Cow::Borrowed("__pairs"),
        #[cfg(any(feature = "mlua_lua52", feature = "mlua_luajit52"))]
        MetaMethod::IPairs => Cow::Borrowed("__ipairs"),
        #[cfg(feature = "mlua_lua54")]
        MetaMethod::Close => Cow::Borrowed("__close"),
        #[cfg(feature = "mlua_luau")]
        MetaMethod::Iter => Cow::Borrowed("__iter"),
        _ => Cow::Borrowed("unknown"),
    }
}

#[cfg(feature = "mlua_send")]
///used by the `mlua_send` feature
pub trait MaybeSend: Send {}
#[cfg(feature = "mlua_send")]
impl<T: Send> MaybeSend for T {}

#[cfg(not(feature = "mlua_send"))]
///used by the `mlua_send` feature
pub trait MaybeSend {}
#[cfg(not(feature = "mlua_send"))]
impl<T> MaybeSend for T {}

#[doc = include_str!("mlu/to_from_macro_doc.md")]
#[cfg(feature = "derive")]
pub use tealr_derive::MluaFromToLua as FromToLua;

///Implement both [mlua::UserData](mlua::UserData) and [TypeName](crate::ToTypename).
///
///Look at [tealr_derive::MluaUserData](tealr_derive::MluaUserData) and [tealr_derive::TypeName](tealr_derive::TypeName)
///for more information on how the implemented traits will behave.
#[cfg(feature = "derive")]
pub use tealr_derive::MluaTealDerive as TealDerive;

///Implements [UserData](mlua::UserData) and [TypeBody](crate::TypeBody)
///
///It wraps the [mlua::UserDataMethods](mlua::UserDataMethods) into [UserDataWrapper](crate::mlu::UserDataWrapper)
///and then passes it to `crate::TealData::add_methods`.
///
///Type body is implemented in a similar way, where it uses the [TealData](crate::mlu::TealData) implementation to get the types
#[cfg(feature = "derive")]
pub use tealr_derive::MluaUserData as UserData;

impl<T: ToTypename> ToTypename for UserDataRef<T> {
    fn to_typename() -> Type {
        T::to_typename()
    }
    fn to_function_param() -> Vec<crate::FunctionParam> {
        T::to_function_param()
    }
}

impl<T: ToTypename> ToTypename for UserDataRefMut<T> {
    fn to_typename() -> Type {
        T::to_typename()
    }
    fn to_function_param() -> Vec<crate::FunctionParam> {
        T::to_function_param()
    }
}
