pub(crate) mod teal_data;
pub(crate) mod teal_data_methods;
pub(crate) mod user_data_wrapper;
use std::borrow::Cow;

pub use mlua;

pub use self::{
    teal_data::{TealData, TypedFunction},
    teal_data_methods::TealDataMethods,
    user_data_wrapper::UserDataWrapper,
};

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
        #[cfg(any(feature = "mlua_lua54", feature = "mlua_lua53", feature = "mlua_lua52"))]
        MetaMethod::Pairs => Cow::Borrowed("__pairs"),
        #[cfg(any(feature = "mlua_lua52"))]
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
