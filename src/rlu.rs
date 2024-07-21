///this module holds some pre made types that can be used to create generics.
pub mod generics;
mod named_parameters;
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

pub use crate::{
    create_generic_rlua as create_generic, create_union_rlua as create_union,
    rlua_create_named_parameters as create_named_parameters,
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
use crate::ToTypename;
///Gets the type of a function that is useful for the FromLuaConversion/ToLuaConversion error.
///
///it should NOT be used to get the real typename.
///
///# WARNING!
///
///The plan is to remove it if/when `rlua::Value::type_name` becomes public. Use at your own risk.
pub fn get_type_name(value: &rlua::Value) -> &'static str {
    #[allow(deprecated)]
    let x = match value {
        rlua::Value::Nil => return "Nil",
        rlua::Value::Boolean(_) => bool::to_old_type_parts(),
        rlua::Value::LightUserData(_) => return "LightUserData",
        #[cfg(all(
            not(feature = "rlua_builtin-lua51"),
            not(feature = "rlua_system-lua51")
        ))]
        rlua::Value::Integer(_) => rlua::Integer::to_old_type_parts(),
        rlua::Value::Number(_) => rlua::Number::to_old_type_parts(),
        rlua::Value::String(_) => String::to_old_type_parts(),
        rlua::Value::Table(_) => rlua::Table::to_old_type_parts(),
        rlua::Value::Function(_) => rlua::Table::to_old_type_parts(),
        rlua::Value::Thread(_) => rlua::Thread::to_old_type_parts(),
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
#[cfg(feature = "derive")]
pub use tealr_derive::RluaFromToLua as FromToLua;
