///this module holds some pre made types that can be used to create generics.
pub mod generics;
mod picker_macro;
pub(crate) mod teal_data;
mod teal_data_fields;
pub(crate) mod teal_data_methods;
mod typed_function;
pub(crate) mod user_data_wrapper;
use std::borrow::Cow;

pub use self::{
    teal_data::TealData, teal_data_methods::TealDataMethods, typed_function::TypedFunction,
    user_data_wrapper::UserDataWrapper,
};
pub use mlua;
pub use teal_data_fields::TealDataFields;

pub(crate) fn get_meta_name(name: mlua::MetaMethod) -> Cow<'static, str> {
    use mlua::MetaMethod;
    match name {
        MetaMethod::Add => Cow::Borrowed("__add"),
        MetaMethod::Sub => Cow::Borrowed("__su"),
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
        #[cfg(any(feature = "mlua_lua54"))]
        MetaMethod::Close => Cow::Borrowed("__close"),

        MetaMethod::Custom(x) => Cow::Owned(x),
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
