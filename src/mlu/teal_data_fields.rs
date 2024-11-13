use mlua::{AnyUserData, FromLua, IntoLua, Lua, MetaMethod};

use crate::ToTypename;

use super::{MaybeSend, TealData};

///The teal version of [UserDataFields](mlua::UserDataFields)
///
///The meaning of every method is the same, and so is its use.
///Look at  [mlua](mlua::UserDataFields) for documentation
///
///The only 2 differences are that [TealDataFields](crate::mlu::TealDataFields) has an extra type bound on `R`.
///These are to get the type names when generating the `.d.tl` file.
pub trait TealDataFields<T: TealData> {
    ///Adds documentation to the next field that gets added
    fn document(&mut self, documentation: &str);
    /// the teal version of [UserDataFields](mlua::UserDataFields::add_field_method_get)
    fn add_field_method_get<S, R, M>(&mut self, name: S, method: M)
    where
        S: AsRef<str> + ToString,
        R: IntoLua + ToTypename,
        M: 'static + MaybeSend + Fn(&Lua, &T) -> mlua::Result<R>;
    /// the teal version of [UserDataFields](mlua::UserDataFields::add_field_method_set)
    fn add_field_method_set<S, A, M>(&mut self, name: S, method: M)
    where
        S: AsRef<str> + ToString,
        A: FromLua + ToTypename,
        M: 'static + MaybeSend + FnMut(&Lua, &mut T, A) -> mlua::Result<()>;
    /// the teal version of [UserDataFields](mlua::UserDataFields::add_field_function_get)
    fn add_field_function_get<S, R, F>(&mut self, name: S, function: F)
    where
        S: ToString + AsRef<str>,
        R: IntoLua + ToTypename,
        F: 'static + MaybeSend + Fn(&Lua, AnyUserData) -> mlua::Result<R>;
    /// the teal version of [UserDataFields](mlua::UserDataFields::add_field_function_set)
    fn add_field_function_set<S, A, F>(&mut self, name: S, function: F)
    where
        S: ToString + AsRef<str>,
        A: FromLua + ToTypename,
        F: 'static + MaybeSend + FnMut(&Lua, AnyUserData, A) -> mlua::Result<()>;
    /// the teal version of [UserDataFields](mlua::UserDataFields::add_meta_field_with)
    fn add_meta_field_with<R, F>(&mut self, meta: MetaMethod, f: F)
    where
        F: 'static + MaybeSend + Fn(&Lua) -> mlua::Result<R>,
        R: IntoLua + ToTypename;
}
