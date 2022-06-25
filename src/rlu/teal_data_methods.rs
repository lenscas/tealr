use std::borrow::Cow;

use rlua::{Context, FromLuaMulti, MetaMethod, Result, ToLua, ToLuaMulti};

use crate::{TealMultiValue, TypeName};

///The teal version of [UserDataMethods](rlua::UserDataMethods)
///
///The meaning of every method is the same, and so is its use.
///Look at  [rlua](rlua::UserDataMethods) for documentation
///
///The only difference is that [TealDataMethods](crate::rlu::TealDataMethods) have an extra type bound on `A` and `R`.
///These are to get the type names when generating the `.d.tl` file
pub trait TealDataMethods<'lua, T> {
    ///Exposes a method to lua
    fn add_method<S, A, R, M>(&mut self, name: &S, method: M)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        M: 'static + Send + Fn(Context<'lua>, &T, A) -> Result<R>;
    ///Exposes a method to lua that has a mutable reference to Self
    fn add_method_mut<S, A, R, M>(&mut self, name: &S, method: M)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        M: 'static + Send + FnMut(Context<'lua>, &mut T, A) -> Result<R>;

    ///Exposes a function to lua (its a method that does not take Self)
    fn add_function<S, A, R, F>(&mut self, name: &S, function: F)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        F: 'static + Send + Fn(Context<'lua>, A) -> Result<R>;

    ///Exposes a mutable function to lua
    fn add_function_mut<S, A, R, F>(&mut self, name: &S, function: F)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        F: 'static + Send + FnMut(Context<'lua>, A) -> Result<R>;

    ///Exposes a meta method to lua <http://lua-users.org/wiki/MetatableEvents>
    fn add_meta_method<A, R, M>(&mut self, meta: MetaMethod, method: M)
    where
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        M: 'static + Send + Fn(Context<'lua>, &T, A) -> Result<R>;
    ///Exposes a meta and mutable method to lua <http://lua-users.org/wiki/MetatableEvents>
    fn add_meta_method_mut<A, R, M>(&mut self, meta: MetaMethod, method: M)
    where
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        M: 'static + Send + FnMut(Context<'lua>, &mut T, A) -> Result<R>;
    ///Exposes a meta function to lua <http://lua-users.org/wiki/MetatableEvents>
    fn add_meta_function<A, R, F>(&mut self, meta: MetaMethod, function: F)
    where
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        F: 'static + Send + Fn(Context<'lua>, A) -> Result<R>;
    ///Exposes a meta and mutable function to lua <http://lua-users.org/wiki/MetatableEvents>
    fn add_meta_function_mut<A, R, F>(&mut self, meta: MetaMethod, function: F)
    where
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        F: 'static + Send + FnMut(Context<'lua>, A) -> Result<R>;
    ///Adds documentation to the next method/function that gets added
    fn document(&mut self, documentation: &str);
    ///Adds documentation for this type itself. They will be written right above the record in the .d.tl file
    fn document_type(&mut self, documentation: &str);
    ///generates an `instance.help()` function on lua's/teals side, which can be used at run time to view the documentation.
    fn generate_help(&mut self);
}

///collets every instance that needs to be exposed to lua
pub trait InstanceCollector<'lua> {
    ///adds an instance
    fn add_instance<T: TypeName + ToLua<'lua>, F: Fn(Context<'lua>) -> rlua::Result<T>>(
        &mut self,
        global_name: Cow<'static, str>,
        instance: F,
    ) -> Result<()>;
    ///adds documentation to this instance
    fn document_instance(&mut self, doc: &'static str);
}
///used to export instances to lua
pub fn set_global_env<T: ExportInstances>(context: rlua::Context) -> rlua::Result<()> {
    let globals = context.globals();
    T::add_instances::<_>(&mut (globals, context))?;
    Ok(())
}

impl<'lua> InstanceCollector<'lua> for (rlua::Table<'lua>, rlua::Context<'lua>) {
    fn add_instance<T: TypeName + ToLua<'lua>, F: Fn(Context<'lua>) -> rlua::Result<T>>(
        &mut self,
        global_name: Cow<'static, str>,
        instance: F,
    ) -> Result<()> {
        let instance = instance(self.1)?;
        self.0.set(global_name.to_string(), instance)?;
        Ok(())
    }
    fn document_instance(&mut self, _: &'static str) {}
}

///implement this to easily document what global instances are exposed to lua
pub trait ExportInstances {
    ///adds the instances
    fn add_instances<'lua, T: InstanceCollector<'lua>>(instance_collector: &mut T) -> Result<()>;
}
