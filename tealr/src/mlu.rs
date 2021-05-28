pub(crate) mod teal_data;
pub(crate) mod teal_data_methods;
pub(crate) mod user_data_wrapper;

pub use self::{
    teal_data::{TealData, TypedFunction},
    teal_data_methods::TealDataMethods,
    user_data_wrapper::UserDataWrapper,
};

pub(crate) fn get_meta_name(name: mlua::MetaMethod) -> &'static str {
    use mlua::MetaMethod;
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
        MetaMethod::Close => "__close",
    }
}
