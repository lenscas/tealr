use std::borrow::Cow;

use mlua::{FromLuaMulti, IntoLua as ToLua, IntoLuaMulti as ToLuaMulti, Lua, MetaMethod, Result};

use crate::{TealMultiValue, ToTypename};

use super::MaybeSend;

///The teal version of [UserDataMethods](mlua::UserDataMethods)
///
///The meaning of every method is the same, and so is its use.
///Look at  [mlua](mlua::UserDataMethods) for documentation
///
///The only 2 differences are that [TealDataMethods](crate::mlu::TealDataMethods) has an extra type bound on `A` and `R`.
///These are to get the type names when generating the `.d.tl` file.

pub trait TealDataMethods<T: ToTypename> {
    ///Exposes a method to lua
    fn add_method<S, A, R, M>(&mut self, name: S, method: M)
    where
        S: ToString + AsRef<str>,
        A: FromLuaMulti + TealMultiValue,
        R: ToLuaMulti + TealMultiValue,
        M: 'static + MaybeSend + Fn(&Lua, &T, A) -> Result<R>;
    ///Exposes a method to lua that has a mutable reference to Self
    fn add_method_mut<S, A, R, M>(&mut self, name: S, method: M)
    where
        S: ToString + AsRef<str>,
        A: FromLuaMulti + TealMultiValue,
        R: ToLuaMulti + TealMultiValue,
        M: 'static + MaybeSend + FnMut(&Lua, &mut T, A) -> Result<R>;

    #[cfg(feature = "mlua_async")]
    ///exposes an async method to lua
    fn add_async_method<S: ToString + AsRef<str>, A, R, M, MR>(&mut self, name: S, method: M)
    where
        T: 'static,
        M: Fn(Lua, UserDataRef<T>, A) -> MR + MaybeSend + 'static,
        A: FromLuaMulti + TealMultiValue,
        MR: std::future::Future<Output = Result<R>> + mlua::MaybeSend + 'static,
        R: ToLuaMulti + TealMultiValue;

    ///Exposes a function to lua (its a method that does not take Self)
    fn add_function<S, A, R, F>(&mut self, name: S, function: F)
    where
        S: ToString + AsRef<str>,
        A: FromLuaMulti + TealMultiValue,
        R: ToLuaMulti + TealMultiValue,
        F: 'static + MaybeSend + Fn(&Lua, A) -> Result<R>;

    ///Exposes a mutable function to lua
    fn add_function_mut<S, A, R, F>(&mut self, name: S, function: F)
    where
        S: ToString + AsRef<str>,
        A: FromLuaMulti + TealMultiValue,
        R: ToLuaMulti + TealMultiValue,
        F: 'static + MaybeSend + FnMut(&Lua, A) -> Result<R>;

    #[cfg(feature = "mlua_async")]
    ///exposes an async function to lua
    fn add_async_function<S, A, R, F, FR>(&mut self, name: S, function: F)
    where
        S: AsRef<str> + ToString,
        A: FromLuaMulti + TealMultiValue,
        R: ToLuaMulti + TealMultiValue,
        F: Fn(Lua, A) -> FR + mlua::MaybeSend + 'static,
        FR: std::future::Future<Output = Result<R>> + mlua::MaybeSend + 'static;

    ///Exposes a meta method to lua [http://lua-users.org/wiki/MetatableEvents](http://lua-users.org/wiki/MetatableEvents)
    fn add_meta_method<A, R, M>(&mut self, meta: MetaMethod, method: M)
    where
        A: FromLuaMulti + TealMultiValue,
        R: ToLuaMulti + TealMultiValue,
        M: 'static + MaybeSend + Fn(&Lua, &T, A) -> Result<R>;
    ///Exposes a meta and mutable method to lua [http://lua-users.org/wiki/MetatableEvents](http://lua-users.org/wiki/MetatableEvents)
    fn add_meta_method_mut<A, R, M>(&mut self, meta: MetaMethod, method: M)
    where
        A: FromLuaMulti + TealMultiValue,
        R: ToLuaMulti + TealMultiValue,
        M: 'static + MaybeSend + FnMut(&Lua, &mut T, A) -> Result<R>;
    ///Exposes a meta function to lua [http://lua-users.org/wiki/MetatableEvents](http://lua-users.org/wiki/MetatableEvents)
    fn add_meta_function<A, R, F>(&mut self, meta: MetaMethod, function: F)
    where
        A: FromLuaMulti + TealMultiValue,
        R: ToLuaMulti + TealMultiValue,
        F: 'static + MaybeSend + Fn(&Lua, A) -> Result<R>;
    ///Exposes a meta and mutable function to lua [http://lua-users.org/wiki/MetatableEvents](http://lua-users.org/wiki/MetatableEvents)
    fn add_meta_function_mut<A, R, F>(&mut self, meta: MetaMethod, function: F)
    where
        A: FromLuaMulti + TealMultiValue,
        R: ToLuaMulti + TealMultiValue,
        F: 'static + MaybeSend + FnMut(&Lua, A) -> Result<R>;
    ///Adds documentation to the next method/function that gets added
    fn document(&mut self, documentation: &str) -> &mut Self;
    ///Adds documentation for this type itself. They will be written right above the record in the .d.tl file
    fn document_type(&mut self, documentation: &str) -> &mut Self;
    ///generates a `.help()` function on lua's/teals side, which can be used at run time to view the documentation.
    fn generate_help(&mut self);
}

///collects every instance that a type has
pub trait InstanceCollector {
    ///adds an instance
    fn add_instance<P, T, F>(&mut self, global_name: P, instance: F) -> Result<&mut Self>
    where
        P: Into<Cow<'static, str>>,
        T: ToTypename + ToLua,
        F: FnOnce(&Lua) -> mlua::Result<T>;
    ///Adds documentation to the next global instance
    fn document_instance(&mut self, doc: &'static str) -> &mut Self;
}

///used to export instances to lua
pub fn set_global_env<T: ExportInstances>(env: T, lua: &Lua) -> Result<()> {
    let globals = lua.globals();
    env.add_instances(&mut (globals, lua))?;
    Ok(())
}

impl InstanceCollector for (mlua::Table, &Lua) {
    fn add_instance<P, T, F>(&mut self, global_name: P, instance: F) -> Result<&mut Self>
    where
        P: Into<Cow<'static, str>>,
        T: ToTypename + ToLua,
        F: FnOnce(&Lua) -> Result<T>,
    {
        let instance = instance(self.1)?;
        self.0.set(global_name.into(), instance)?;
        Ok(self)
    }
    fn document_instance(&mut self, _: &'static str) -> &mut Self {
        self
    }
}

///implement this to easily document what global instances are exposed to lua
pub trait ExportInstances: Default {
    ///adds the instances
    fn add_instances<T: InstanceCollector>(self, instance_collector: &mut T) -> Result<()>;
}
