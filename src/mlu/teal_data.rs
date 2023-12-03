use crate::ToTypename;

use super::{TealDataFields, TealDataMethods};

///This is the teal version of [UserData](mlua::UserData).
pub trait TealData: Sized + ToTypename {
    ///same as [UserData::add_methods](mlua::UserData::add_methods).
    ///Refer to its documentation on how to use it.
    ///
    ///only difference is that it takes a [TealDataMethods](crate::mlu::TealDataMethods),
    ///which is the teal version of [UserDataMethods](mlua::UserDataMethods)
    fn add_methods<'lua, T: TealDataMethods<'lua, Self>>(_methods: &mut T) {}
    ///same as [UserData::add_fields](mlua::UserData::add_fields).
    ///Refer to its documentation on how to use it.
    ///
    ///only difference is that it takes a [TealDataFields](crate::mlu::TealDataFields),
    ///which is the teal version of [UserDataFields](mlua::UserDataFields)
    fn add_fields<'lua, F: TealDataFields<'lua, Self>>(_fields: &mut F) {}
}
