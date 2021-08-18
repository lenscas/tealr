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
    teal_data::TealData, teal_data_methods::TealDataMethods, typed_function::TypedFunction,
    user_data_wrapper::UserDataWrapper,
};

pub(crate) fn get_meta_name(name: rlua::MetaMethod) -> &'static str {
    use rlua::MetaMethod;
    match name {
        MetaMethod::Add => "__add",
        MetaMethod::Sub => "__su",
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
///#WARNING!
///
///The plan is to remove it if/when `rlua::Value::type_name` becomes public. Use at your own risk.
pub fn get_type_name(value: &rlua::Value, dir: crate::Direction) -> &'static str {
    let x = match value {
        rlua::Value::Nil => Cow::Borrowed("Nil"),
        rlua::Value::Boolean(_) => bool::get_type_name(dir),
        rlua::Value::LightUserData(_) => Cow::Borrowed("LightUserData"),
        rlua::Value::Integer(_) => rlua::Integer::get_type_name(dir),
        rlua::Value::Number(_) => rlua::Number::get_type_name(dir),
        rlua::Value::String(_) => String::get_type_name(dir),
        rlua::Value::Table(_) => rlua::Table::get_type_name(dir),
        rlua::Value::Function(_) => rlua::Table::get_type_name(dir),
        rlua::Value::Thread(_) => rlua::Thread::get_type_name(dir),
        rlua::Value::UserData(_) => Cow::Borrowed("userdata"),
        rlua::Value::Error(_) => Cow::Borrowed("any"),
    };
    match x {
        Cow::Borrowed(x) => x,
        Cow::Owned(_) => "any",
    }
}
