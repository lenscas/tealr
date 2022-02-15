use crate::TypeName;

use super::TealDataMethods;

///This is the teal version of [UserData](mlua::UserData).
pub trait TealData: Sized + TypeName {
    ///same as [UserData::add_methods](mlua::UserData::add_methods).
    ///Refer to its documentation on how to use it.
    ///
    ///only difference is that it takes a [TealDataMethods](crate::mlu::TealDataMethods),
    ///which is the teal version of [UserDataMethods](mlua::UserDataMethods)
    fn add_methods<'lua, T: TealDataMethods<'lua, Self>>(_methods: &mut T) {}
}
