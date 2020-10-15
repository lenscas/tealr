use std::marker::PhantomData;

use rlua::{Context, FromLuaMulti, MetaMethod, Result, ToLuaMulti, UserData, UserDataMethods};

use crate::{teal_data_methods::TealDataMethods, teal_multivalue::TealMultiValue};

///Used to turn [UserDataMethods](rlua::UserDataMethods) into [TealDataMethods](crate::TealDataMethods).
///
///This allows you to easily implement [UserData](rlua::UserData) by wrapping the [UserDataMethods](rlua::UserDataMethods) in this struct
///and then passing it to the TealData implementation
///
///TODO: Make a derive that does this automatically.
pub struct UserDataWrapper<'a, 'lua, Container, T>
where
    Container: UserDataMethods<'lua, T>,
    T: UserData,
{
    cont: &'a mut Container,
    _t: &'lua PhantomData<T>,
}
impl<'a, 'lua, Container, T> UserDataWrapper<'a, 'lua, Container, T>
where
    Container: UserDataMethods<'lua, T> + 'a,
    T: UserData,
{
    ///wraps it.
    ///```
    ///# use rlua::{Lua, Result, UserData, UserDataMethods};
    ///# use tealr::{TealData, TealDataMethods, TypePrinter, TypeWalker, UserDataWrapper};
    ///struct Example {}
    ///impl TealData for Example {
    ///    fn get_type_name() -> &'static str {
    ///        "Example"
    ///    }
    ///}
     ///impl UserData for Example {
    ///    fn add_methods<'lua, T: UserDataMethods<'lua, Self>>(methods: &mut T) {
    ///        let mut x =UserDataWrapper::from_user_data_methods(methods);
    ///        <Self as TealData>::add_methods(&mut x);
    ///    }
    ///}
    ///```
    pub fn from_user_data_methods(cont: &'a mut Container) -> Self {
        Self {
            cont,
            _t: &PhantomData,
        }
    }
}

impl<'a, 'lua, Container, T> TealDataMethods<'lua, T> for UserDataWrapper<'a, 'lua, Container, T>
where
    T: UserData,
    Container: UserDataMethods<'lua, T>,
{
    #[inline(always)]
    fn add_method<S, A, R, M>(&mut self, name: &S, method: M)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        M: 'static + Send + Fn(Context<'lua>, &T, A) -> Result<R>,
    {
        self.cont.add_method(name, method)
    }
    #[inline(always)]
    fn add_method_mut<S, A, R, M>(&mut self, name: &S, method: M)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMulti<'lua>,
        R: ToLuaMulti<'lua>,
        M: 'static + Send + FnMut(Context<'lua>, &mut T, A) -> Result<R>,
    {
        self.cont.add_method_mut(name, method)
    }
    #[inline(always)]
    fn add_function<S, A, R, F>(&mut self, name: &S, function: F)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMulti<'lua>,
        R: ToLuaMulti<'lua>,
        F: 'static + Send + Fn(Context<'lua>, A) -> Result<R>,
    {
        self.cont.add_function(name, function)
    }
    #[inline(always)]
    fn add_function_mut<S, A, R, F>(&mut self, name: &S, function: F)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMulti<'lua>,
        R: ToLuaMulti<'lua>,
        F: 'static + Send + FnMut(Context<'lua>, A) -> Result<R>,
    {
        self.cont.add_function_mut(name, function)
    }
    #[inline(always)]
    fn add_meta_method<A, R, M>(&mut self, meta: MetaMethod, method: M)
    where
        A: FromLuaMulti<'lua>,
        R: ToLuaMulti<'lua>,
        M: 'static + Send + Fn(Context<'lua>, &T, A) -> Result<R>,
    {
        self.cont.add_meta_method(meta, method)
    }
    #[inline(always)]
    fn add_meta_method_mut<A, R, M>(&mut self, meta: MetaMethod, method: M)
    where
        A: FromLuaMulti<'lua>,
        R: ToLuaMulti<'lua>,
        M: 'static + Send + FnMut(Context<'lua>, &mut T, A) -> Result<R>,
    {
        self.cont.add_meta_method_mut(meta, method)
    }
    #[inline(always)]
    fn add_meta_function<A, R, F>(&mut self, meta: MetaMethod, function: F)
    where
        A: FromLuaMulti<'lua>,
        R: ToLuaMulti<'lua>,
        F: 'static + Send + Fn(Context<'lua>, A) -> Result<R>,
    {
        self.cont.add_meta_function(meta, function)
    }
    #[inline(always)]
    fn add_meta_function_mut<A, R, F>(&mut self, meta: MetaMethod, function: F)
    where
        A: FromLuaMulti<'lua>,
        R: ToLuaMulti<'lua>,
        F: 'static + Send + FnMut(Context<'lua>, A) -> Result<R>,
    {
        self.cont.add_meta_function_mut(meta, function)
    }
}
