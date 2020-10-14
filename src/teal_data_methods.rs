use rlua::{Context, FromLuaMulti, MetaMethod, Result, ToLuaMulti};

use crate::teal_multivalue::TealMultiValue;

///The teal version of Rlua::UserDataMethods
///The meaning of every method is the same, and so is its use.
///Look at RLua for documentation
///The only diffrence is that TealDataMethods have extra type bounds on A and R
///These are to get the type names when generating the .d.tl file
pub trait TealDataMethods<'lua, T> {
    fn add_method<S, A, R, M>(&mut self, name: &S, method: M)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        M: 'static + Send + Fn(Context<'lua>, &T, A) -> Result<R>;

    fn add_method_mut<S, A, R, M>(&mut self, name: &S, method: M)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        M: 'static + Send + FnMut(Context<'lua>, &mut T, A) -> Result<R>;

    fn add_function<S, A, R, F>(&mut self, name: &S, function: F)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        F: 'static + Send + Fn(Context<'lua>, A) -> Result<R>;

    fn add_function_mut<S, A, R, F>(&mut self, name: &S, function: F)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        F: 'static + Send + FnMut(Context<'lua>, A) -> Result<R>;

    fn add_meta_method<A, R, M>(&mut self, meta: MetaMethod, method: M)
    where
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        M: 'static + Send + Fn(Context<'lua>, &T, A) -> Result<R>;

    fn add_meta_method_mut<A, R, M>(&mut self, meta: MetaMethod, method: M)
    where
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        M: 'static + Send + FnMut(Context<'lua>, &mut T, A) -> Result<R>;

    fn add_meta_function<A, R, F>(&mut self, meta: MetaMethod, function: F)
    where
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        F: 'static + Send + Fn(Context<'lua>, A) -> Result<R>;

    fn add_meta_function_mut<A, R, F>(&mut self, meta: MetaMethod, function: F)
    where
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        F: 'static + Send + FnMut(Context<'lua>, A) -> Result<R>;
}