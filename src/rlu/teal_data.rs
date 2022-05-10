use super::TealDataMethods;

///This is the teal version of [UserData](rlua::UserData).
pub trait TealData: Sized {
    ///same as [UserData::add_methods](rlua::UserData::add_methods).
    ///Refer to its documentation on how to use it.
    ///
    ///only difference is that it takes a [TealDataMethods](crate::rlu::TealDataMethods),
    ///which is the teal version of [UserDataMethods](rlua::UserDataMethods)
    fn add_methods<'lua, T: TealDataMethods<'lua, Self>>(_methods: &mut T) {}
}
