///this module holds some pre made types that can be used to create generics.
pub mod generics;
mod picker_macro;
pub(crate) mod teal_data;
pub(crate) mod teal_data_methods;
mod typed_function;
pub(crate) mod user_data_wrapper;
use std::borrow::Cow;

pub use rlua;

pub use self::{
    picker_macro::FromLuaExact,
    teal_data::TealData,
    teal_data_methods::{set_global_env, ExportInstances, InstanceCollector, TealDataMethods},
    typed_function::TypedFunction,
    user_data_wrapper::UserDataWrapper,
};

pub(crate) fn get_meta_name(name: rlua::MetaMethod) -> &'static str {
    use rlua::MetaMethod;
    match name {
        MetaMethod::Add => "__add",
        MetaMethod::Sub => "__sub",
        MetaMethod::Mul => "__mul",
        MetaMethod::Div => "__div",
        MetaMethod::Mod => "__mod",
        MetaMethod::Pow => "__pow",
        MetaMethod::Unm => "__unm",
        MetaMethod::IDiv => "__idiv",
        MetaMethod::BAnd => "__band",
        MetaMethod::BOr => "__bor",
        MetaMethod::BXor => "__bxor",
        MetaMethod::BNot => "__bnot",
        MetaMethod::Shl => "__shl",
        MetaMethod::Shr => "__shr",
        MetaMethod::Concat => "__concat",
        MetaMethod::Len => "__len",
        MetaMethod::Eq => "__eq",
        MetaMethod::Lt => "__lt",
        MetaMethod::Le => "__le",
        MetaMethod::Index => "__index",
        MetaMethod::NewIndex => "__newindex",
        MetaMethod::Call => "__call",
        MetaMethod::ToString => "__tostring",
        MetaMethod::Pairs => "__pairs",
    }
}
use crate::TypeName;
///Gets the type of a function that is useful for the FromLuaConversion/ToLuaConversion error.
///
///it should NOT be used to get the real typename.
///
///# WARNING!
///
///The plan is to remove it if/when `rlua::Value::type_name` becomes public. Use at your own risk.
pub fn get_type_name(value: &rlua::Value) -> &'static str {
    let x = match value {
        rlua::Value::Nil => return "Nil",
        rlua::Value::Boolean(_) => bool::get_type_parts(),
        rlua::Value::LightUserData(_) => return "LightUserData",
        rlua::Value::Integer(_) => rlua::Integer::get_type_parts(),
        rlua::Value::Number(_) => rlua::Number::get_type_parts(),
        rlua::Value::String(_) => String::get_type_parts(),
        rlua::Value::Table(_) => rlua::Table::get_type_parts(),
        rlua::Value::Function(_) => rlua::Table::get_type_parts(),
        rlua::Value::Thread(_) => rlua::Thread::get_type_parts(),
        rlua::Value::UserData(_) => return "userdata",
        rlua::Value::Error(_) => return "any",
    };
    match crate::type_parts_to_str(x) {
        Cow::Borrowed(x) => x,
        Cow::Owned(_) => "any",
    }
}

///Implements [rlua::UserData](rlua::UserData) and `tealr::TypeBody`
///
///It wraps the [rlua::UserDataMethods](rlua::UserDataMethods) into `tealr::rlu::UserDataWrapper`
///and then passes it to `tealr::rlu::TealData::add_methods`.
///
///Type body is implemented in a similar way, where it uses the `tealr::TealData` implementation to get the types
#[cfg(feature = "derive")]
pub use tealr_derive::RluaUserData as UserData;

///Implement both [rlua::UserData](rlua::UserData) and `[TypeName](tealr::TypeName]`.
///
///Look at [tealr_derive::RluaUserData](tealr_derive::RluaUserData) and [tealr_derive::TypeName](tealr_derive::TypeName)
///for more information on how the implemented traits will behave.
#[cfg(feature = "derive")]
pub use tealr_derive::RluaTealDerive as TealDerive;

#[doc = include_str!("rlu/to_from_macro_doc.md")]
#[cfg(all(feature = "derive"))]
pub use tealr_derive::RluaFromToLua as FromToLua;
