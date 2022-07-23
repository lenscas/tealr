use std::borrow::Cow;

use mlua::{FromLuaMulti, Lua, MetaMethod, Result, ToLua, ToLuaMulti};

use crate::{TealMultiValue, TypeName};

use super::MaybeSend;

///The teal version of [UserDataMethods](mlua::UserDataMethods)
///
///The meaning of every method is the same, and so is its use.
///Look at  [mlua](mlua::UserDataMethods) for documentation
///
///The only 2 differences are that [TealDataMethods](crate::mlu::TealDataMethods) has an extra type bound on `A` and `R`.
///These are to get the type names when generating the `.d.tl` file.

pub trait TealDataMethods<'lua, T: TypeName> {
    ///Exposes a method to lua
    fn add_method<S, A, R, M>(&mut self, name: &S, method: M)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        M: 'static + MaybeSend + Fn(&'lua Lua, &T, A) -> Result<R>;
    ///Exposes a method to lua that has a mutable reference to Self
    fn add_method_mut<S, A, R, M>(&mut self, name: &S, method: M)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        M: 'static + MaybeSend + FnMut(&'lua Lua, &mut T, A) -> Result<R>;

    #[cfg(feature = "mlua_async")]
    ///exposes an async method to lua
    fn add_async_method<S: ?Sized, A, R, M, MR>(&mut self, name: &S, method: M)
    where
        T: Clone,
        S: AsRef<[u8]>,
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        M: 'static + MaybeSend + Fn(&'lua Lua, T, A) -> MR,
        MR: 'lua + std::future::Future<Output = Result<R>>;

    ///Exposes a function to lua (its a method that does not take Self)
    fn add_function<S, A, R, F>(&mut self, name: &S, function: F)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        F: 'static + MaybeSend + Fn(&'lua Lua, A) -> Result<R>;

    ///Exposes a mutable function to lua
    fn add_function_mut<S, A, R, F>(&mut self, name: &S, function: F)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        F: 'static + MaybeSend + FnMut(&'lua Lua, A) -> Result<R>;

    #[cfg(feature = "mlua_async")]
    ///exposes an async function to lua
    fn add_async_function<S: ?Sized, A, R, F, FR>(&mut self, name: &S, function: F)
    where
        S: AsRef<[u8]>,
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        F: 'static + MaybeSend + Fn(&'lua Lua, A) -> FR,
        FR: 'lua + std::future::Future<Output = Result<R>>;

    ///Exposes a meta method to lua [http://lua-users.org/wiki/MetatableEvents](http://lua-users.org/wiki/MetatableEvents)
    fn add_meta_method<A, R, M>(&mut self, meta: MetaMethod, method: M)
    where
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        M: 'static + MaybeSend + Fn(&'lua Lua, &T, A) -> Result<R>;
    ///Exposes a meta and mutable method to lua [http://lua-users.org/wiki/MetatableEvents](http://lua-users.org/wiki/MetatableEvents)
    fn add_meta_method_mut<A, R, M>(&mut self, meta: MetaMethod, method: M)
    where
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        M: 'static + MaybeSend + FnMut(&'lua Lua, &mut T, A) -> Result<R>;
    ///Exposes a meta function to lua [http://lua-users.org/wiki/MetatableEvents](http://lua-users.org/wiki/MetatableEvents)
    fn add_meta_function<A, R, F>(&mut self, meta: MetaMethod, function: F)
    where
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        F: 'static + MaybeSend + Fn(&'lua Lua, A) -> Result<R>;
    ///Exposes a meta and mutable function to lua [http://lua-users.org/wiki/MetatableEvents](http://lua-users.org/wiki/MetatableEvents)
    fn add_meta_function_mut<A, R, F>(&mut self, meta: MetaMethod, function: F)
    where
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        F: 'static + MaybeSend + FnMut(&'lua Lua, A) -> Result<R>;
    ///Adds documentation to the next method/function that gets added
    fn document(&mut self, documentation: &str);
    ///Adds documentation for this type itself. They will be written right above the record in the .d.tl file
    fn document_type(&mut self, documentation: &str);
    ///generates a `.help()` function on lua's/teals side, which can be used at run time to view the documentation.
    fn generate_help(&mut self);
}

///collects every instance that a type has
pub trait InstanceCollector<'lua> {
    ///adds an instance
    fn add_instance<T: TypeName + ToLua<'lua>, F: Fn(&'lua mlua::Lua) -> mlua::Result<T>>(
        &mut self,
        global_name: Cow<'static, str>,
        instance: F,
    ) -> Result<()>;
    ///Adds documentation to the next global instance
    fn document_instance(&mut self, doc: &'static str);
}

///used to export instances to lua
pub fn set_global_env<T: ExportInstances>(lua: &mlua::Lua) -> Result<()> {
    let globals = lua.globals();
    T::default().add_instances(&mut (globals, lua))?;
    Ok(())
}

impl<'lua> InstanceCollector<'lua> for (mlua::Table<'lua>, &'lua mlua::Lua) {
    fn add_instance<T: TypeName + ToLua<'lua>, F: Fn(&'lua mlua::Lua) -> Result<T>>(
        &mut self,
        global_name: Cow<'static, str>,
        instance: F,
    ) -> Result<()> {
        let instance = instance(self.1)?;
        self.0.set(global_name, instance)?;
        Ok(())
    }
    fn document_instance(&mut self, _: &'static str) {}
}

///implement this to easily document what global instances are exposed to lua
pub trait ExportInstances: Default {
    ///adds the instances
    fn add_instances<'lua, T: InstanceCollector<'lua>>(
        self,
        instance_collector: &mut T,
    ) -> Result<()>;
}
