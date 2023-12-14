use bstr::ByteVec;
use mlua::{
    FromLuaMulti, IntoLuaMulti as ToLuaMulti, Lua, MetaMethod, Result, UserData, UserDataFields,
    UserDataMethods,
};
use std::{collections::HashMap, marker::PhantomData};

use super::{MaybeSend, TealData, TealDataFields, TealDataMethods};
use crate::{type_generator::get_method_data, TealMultiValue, ToTypename, TypeName};

///Used to turn [UserDataMethods](mlua::UserDataMethods) into [TealDataMethods](crate::mlu::TealDataMethods).
///
///This allows you to easily implement [UserData](mlua::UserData) by wrapping the [UserDataMethods](mlua::UserDataMethods) in this struct
///and then passing it to the TealData implementation
///
pub struct UserDataWrapper<'a, 'lua, Container, T>
where
    T: UserData,
{
    cont: &'a mut Container,
    _t: std::marker::PhantomData<(&'a (), T)>,
    _x: &'lua std::marker::PhantomData<()>,
    documentation: HashMap<Vec<u8>, String>,
    type_doc: String,
    next_docs: Option<String>,
}
impl<'a, 'lua, Container, T> UserDataWrapper<'a, 'lua, Container, T>
where
    Container: UserDataMethods<'lua, T> + 'a,
    T: UserData,
{
    ///wraps the [UserDataMethods](mlua::UserDataMethods) so it can be used by [TealData](crate::mlu::TealData) to set methods.
    ///```
    ///# use std::borrow::Cow;
    ///# use mlua::{Lua, Result, UserData, UserDataMethods};
    ///# use tealr::{Type, mlu::{TealData, TealDataMethods,UserDataWrapper}, TypeWalker, ToTypename,NamePart,TealType, KindOfType};
    /// struct Example {}
    /// impl TealData for Example {}
    /// impl ToTypename for Example {
    ///     fn to_typename() -> Type {
    ///         Type::new_single("Example", tealr::KindOfType::External)
    ///     }
    /// }
    /// impl UserData for Example {
    ///     fn add_methods<'lua, T: UserDataMethods<'lua, Self>>(methods: &mut T) {
    ///         let mut x = UserDataWrapper::from_user_data_methods(methods);
    ///         <Self as TealData>::add_methods(&mut x);
    ///     }
    ///}
    ///
    ///```
    pub fn from_user_data_methods(cont: &'a mut Container) -> Self {
        Self {
            cont,
            _t: PhantomData,
            _x: &PhantomData,
            documentation: Default::default(),
            next_docs: Default::default(),
            type_doc: Default::default(),
        }
    }
}
impl<'a, 'lua, Container, T> UserDataWrapper<'a, 'lua, Container, T>
where
    Container: UserDataFields<'lua, T> + 'a,
    T: UserData,
{
    ///wraps the [UserDataMethods](mlua::UserDataFields) so it can be used by [TealData](crate::mlu::TealData) to set fields.
    ///```
    ///# use std::borrow::Cow;
    ///# use mlua::{Lua, Result, UserData, UserDataFields};
    ///# use tealr::{Type,new_type, mlu::{TealData, TealDataFields,UserDataWrapper}, TypeWalker, ToTypename,NamePart,TealType, KindOfType};
    /// struct Example {}
    /// impl TealData for Example {}
    /// impl ToTypename for Example {
    ///     fn to_typename() -> Type {
    ///         Type::new_single("Example", tealr::KindOfType::External)
    ///     }
    /// }
    /// impl UserData for Example {
    ///     fn add_fields<'lua, T: UserDataFields<'lua, Self>>(methods: &mut T) {
    ///         let mut x = UserDataWrapper::from_user_data_fields(methods);
    ///         <Self as TealData>::add_fields(&mut x);
    ///     }
    ///}
    ///
    ///```
    pub fn from_user_data_fields(cont: &'a mut Container) -> Self {
        Self {
            cont,
            _t: PhantomData,
            _x: &PhantomData,
            documentation: Default::default(),
            next_docs: Default::default(),
            type_doc: Default::default(),
        }
    }
}
impl<'a, 'lua, Container, T: ToTypename> UserDataWrapper<'a, 'lua, Container, T>
where
    T: UserData,
    //Container: UserDataMethods<'lua, T>,
{
    fn copy_method_docs<A, R>(&mut self, to: &str, self_type: bool)
    where
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
    {
        let type_def = get_method_data::<A, R, _>(to, false, self_type.then(|| T::to_typename()));
        let generated = type_def
            .generate(&Default::default())
            .map(|v| "Signature: ".to_string() + &v)
            .unwrap_or_default();
        let docs = generated + "\n\ndocs:\n" + &self.next_docs.take().unwrap_or_default();
        let documentation = &mut self.documentation;
        documentation.insert(to.as_bytes().to_owned(), docs);
    }
    fn copy_field_docs<F: ToTypename>(&mut self, name: &str) {
        let name = name.as_bytes().to_vec();
        let documentation = &mut self.documentation;
        let mut current_doc = match documentation.remove(&name) {
            Some(x) => x,
            None => {
                let mut str = crate::type_parts_to_str(F::get_type_parts()).into_owned();
                str.push_str("\n\n docs:\n");
                str
            }
        };
        current_doc.push_str(&self.next_docs.take().unwrap_or_default());
        documentation.insert(name, current_doc);
    }
    fn document(&mut self, documentation: &str) {
        match &mut self.next_docs {
            Some(x) => {
                x.push('\n');
                x.push_str(documentation)
            }
            None => self.next_docs = Some(documentation.to_owned()),
        };
    }
}

impl<'a, 'lua, Container, T: ToTypename> TealDataMethods<'lua, T>
    for UserDataWrapper<'a, 'lua, Container, T>
where
    T: UserData,
    Container: UserDataMethods<'lua, T>,
{
    #[inline(always)]
    fn add_method<S, A, R, M>(&mut self, name: &S, method: M)
    where
        S: ?Sized + AsRef<str>,
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        M: 'static + MaybeSend + Fn(&'lua Lua, &T, A) -> Result<R>,
    {
        self.copy_method_docs::<A, R>(name.as_ref(), true);
        self.cont.add_method(name, method)
    }
    #[inline(always)]
    fn add_method_mut<S, A, R, M>(&mut self, name: &S, method: M)
    where
        S: ?Sized + AsRef<str>,
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        M: 'static + MaybeSend + FnMut(&'lua Lua, &mut T, A) -> Result<R>,
    {
        self.copy_method_docs::<A, R>(name.as_ref(), true);
        self.cont.add_method_mut(name, method)
    }
    #[inline(always)]
    fn add_function<S, A, R, F>(&mut self, name: &S, function: F)
    where
        S: ?Sized + AsRef<str>,
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        F: 'static + MaybeSend + Fn(&'lua Lua, A) -> Result<R>,
    {
        self.copy_method_docs::<A, R>(name.as_ref(), false);
        self.cont.add_function(name, function)
    }
    #[inline(always)]
    fn add_function_mut<S, A, R, F>(&mut self, name: &S, function: F)
    where
        S: ?Sized + AsRef<str>,
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        F: 'static + MaybeSend + FnMut(&'lua Lua, A) -> Result<R>,
    {
        self.copy_method_docs::<A, R>(name.as_ref(), false);
        self.cont.add_function_mut(name, function)
    }
    #[inline(always)]
    fn add_meta_method<A, R, M>(&mut self, meta: MetaMethod, method: M)
    where
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        M: 'static + MaybeSend + Fn(&'lua Lua, &T, A) -> Result<R>,
    {
        self.copy_method_docs::<A, R>(meta.name(), true);
        self.cont.add_meta_method(meta, method)
    }
    #[inline(always)]
    fn add_meta_method_mut<A, R, M>(&mut self, meta: MetaMethod, method: M)
    where
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        M: 'static + MaybeSend + FnMut(&'lua Lua, &mut T, A) -> Result<R>,
    {
        self.copy_method_docs::<A, R>(meta.name(), true);
        self.cont.add_meta_method_mut(meta, method)
    }
    #[inline(always)]
    fn add_meta_function<A, R, F>(&mut self, meta: MetaMethod, function: F)
    where
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        F: 'static + MaybeSend + Fn(&'lua Lua, A) -> Result<R>,
    {
        self.copy_method_docs::<A, R>(meta.name(), false);
        self.cont.add_meta_function(meta, function)
    }
    #[inline(always)]
    fn add_meta_function_mut<A, R, F>(&mut self, meta: MetaMethod, function: F)
    where
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        F: 'static + MaybeSend + FnMut(&'lua Lua, A) -> Result<R>,
    {
        self.copy_method_docs::<A, R>(meta.name(), false);
        self.cont.add_meta_function_mut(meta, function)
    }

    #[cfg(feature = "mlua_async")]
    #[inline(always)]
    fn add_async_method<'s, S: ?Sized + AsRef<str>, A, R, M, MR>(&mut self, name: &S, method: M)
    where
        'lua: 's,
        T: 'static,
        M: Fn(&'lua Lua, &'s T, A) -> MR + MaybeSend + 'static,
        A: FromLuaMulti<'lua> + TealMultiValue,
        MR: std::future::Future<Output = Result<R>> + 's,
        R: ToLuaMulti<'lua> + TealMultiValue,
    {
        self.copy_method_docs::<A, R>(name.as_ref(), true);
        self.cont.add_async_method(name, method)
    }

    #[cfg(feature = "mlua_async")]
    #[inline(always)]
    fn add_async_function<S: ?Sized, A, R, F, FR>(&mut self, name: &S, function: F)
    where
        S: AsRef<str>,
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        F: 'static + MaybeSend + Fn(&'lua Lua, A) -> FR,
        FR: 'lua + std::future::Future<Output = Result<R>>,
    {
        self.copy_method_docs::<A, R>(name.as_ref(), false);
        self.cont.add_async_function(name, function)
    }

    fn document(&mut self, documentation: &str) -> &mut Self {
        self.document(documentation);
        self
    }
    fn generate_help(&mut self) {
        let help = self.documentation.clone();
        let type_doc = self.type_doc.clone();
        self.add_function("help", move |lua, key: Option<mlua::String>| {
            let doc = match key {
                Some(x) => help
                    .get(x.as_bytes())
                    .map(|v| v.as_bytes())
                    .map(ToOwned::to_owned)
                    .unwrap_or_else(|| {
                        b"The given key is not found. Use `.help()` to list available pages."
                            .to_vec()
                    }),
                None => {
                    let mut x = help
                        .keys()
                        .map(ToOwned::to_owned)
                        .flat_map(|mut v| {
                            v.push_char('\n');
                            v
                        })
                        .collect::<Vec<_>>();
                    let mut y = (type_doc.clone() + "\n" + "Available pages:\n").into_bytes();
                    y.append(&mut x);
                    y
                }
            };

            lua.create_string(doc)
        })
    }

    fn document_type(&mut self, documentation: &str) -> &mut Self {
        self.type_doc.push_str(documentation);
        self.type_doc.push('\n');
        self
    }
}

impl<'a, 'lua, Container, T: ToTypename + TealData> TealDataFields<'lua, T>
    for UserDataWrapper<'a, 'lua, Container, T>
where
    T: UserData,
    Container: UserDataFields<'lua, T>,
{
    fn add_field_method_get<S, R, M>(&mut self, name: &S, method: M)
    where
        S: AsRef<str> + ?Sized,
        R: mlua::IntoLua<'lua> + ToTypename,
        M: 'static + MaybeSend + Fn(&'lua Lua, &T) -> mlua::Result<R>,
    {
        self.copy_field_docs::<R>(name.as_ref());
        self.cont.add_field_method_get(name, method)
    }

    fn add_field_method_set<S, A, M>(&mut self, name: &S, method: M)
    where
        S: AsRef<str> + ?Sized,
        A: mlua::FromLua<'lua> + ToTypename,
        M: 'static + MaybeSend + FnMut(&'lua Lua, &mut T, A) -> mlua::Result<()>,
    {
        self.copy_field_docs::<A>(name.as_ref());
        self.cont.add_field_method_set(name, method)
    }

    fn add_field_function_get<S, R, F>(&mut self, name: &S, function: F)
    where
        S: AsRef<str> + ?Sized,
        R: mlua::IntoLua<'lua> + ToTypename,
        F: 'static + MaybeSend + Fn(&'lua Lua, mlua::AnyUserData<'lua>) -> mlua::Result<R>,
    {
        self.copy_field_docs::<R>(name.as_ref());
        self.cont.add_field_function_get(name, function)
    }

    fn add_field_function_set<S, A, F>(&mut self, name: &S, function: F)
    where
        S: AsRef<str> + ?Sized,
        A: mlua::FromLua<'lua> + ToTypename,
        F: 'static + MaybeSend + FnMut(&'lua Lua, mlua::AnyUserData<'lua>, A) -> mlua::Result<()>,
    {
        self.copy_field_docs::<A>(name.as_ref());
        self.cont.add_field_function_set(name, function)
    }

    fn add_meta_field_with<R, F>(&mut self, meta: MetaMethod, f: F)
    where
        F: 'static + MaybeSend + Fn(&'lua Lua) -> mlua::Result<R>,
        R: mlua::IntoLua<'lua> + ToTypename,
    {
        self.copy_field_docs::<R>(meta.name());
        self.cont.add_meta_field_with(meta, f)
    }

    fn document(&mut self, documentation: &str) {
        self.document(documentation)
    }
}
