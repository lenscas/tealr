use bstr::{ByteSlice, ByteVec};
#[cfg(feature = "mlua_async")]
use mlua::UserDataRef;
use mlua::{
    FromLuaMulti, IntoLuaMulti as ToLuaMulti, Lua, MetaMethod, Result, UserData, UserDataFields,
    UserDataMethods,
};
use std::{collections::HashMap, marker::PhantomData};

use super::{MaybeSend, TealData, TealDataFields, TealDataMethods};
use crate::{type_generator::get_method_data, type_to_string, TealMultiValue, ToTypename};

///Used to turn [UserDataMethods](mlua::UserDataMethods) into [TealDataMethods](crate::mlu::TealDataMethods).
///
///This allows you to easily implement [UserData](mlua::UserData) by wrapping the [UserDataMethods](mlua::UserDataMethods) in this struct
///and then passing it to the TealData implementation
///
pub struct UserDataWrapper<'a, Container, T>
where
    T: UserData,
{
    cont: &'a mut Container,
    _t: PhantomData<T>,
    documentation: HashMap<Vec<u8>, String>,
    type_doc: String,
    next_docs: Option<String>,
}
impl<'a, Container, T> UserDataWrapper<'a, Container, T>
where
    Container: UserDataMethods<T>,
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
    ///     fn add_methods<T: UserDataMethods<Self>>(methods: &mut T) {
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
            documentation: Default::default(),
            next_docs: Default::default(),
            type_doc: Default::default(),
        }
    }
}
impl<'a, Container, T> UserDataWrapper<'a, Container, T>
where
    Container: UserDataFields<T>,
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
    ///     fn add_fields<T: UserDataFields<Self>>(methods: &mut T) {
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
            documentation: Default::default(),
            next_docs: Default::default(),
            type_doc: Default::default(),
        }
    }
}
impl<Container, T: ToTypename> UserDataWrapper<'_, Container, T>
where
    T: UserData,
    //Container: UserDataMethods<T>,
{
    fn copy_method_docs<A, R>(&mut self, to: &str, self_type: bool)
    where
        A: FromLuaMulti + TealMultiValue,
        R: ToLuaMulti + TealMultiValue,
    {
        let type_def = get_method_data::<A, R, _>(to, false, self_type.then(|| T::to_typename()));
        let generated = type_to_string(&type_def.into_type(), false);

        let docs = generated + "\n\ndocs:\n" + &self.next_docs.take().unwrap_or_default();
        let documentation = &mut self.documentation;
        documentation.insert(to.as_bytes().to_owned(), docs);
    }
    fn copy_field_docs<F: ToTypename>(&mut self, name: &str) {
        let name = name.as_bytes().to_vec();
        let documentation = &mut self.documentation;
        let mut current_doc = documentation.remove(&name).unwrap_or_else(|| {
            let mut str = type_to_string(&F::to_typename(), false);
            str.push_str("\n\n docs:\n");
            str
        });
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

impl<Container, T: ToTypename> TealDataMethods<T> for UserDataWrapper<'_, Container, T>
where
    T: UserData,
    Container: UserDataMethods<T>,
{
    #[inline(always)]
    fn add_method<S, A, R, M>(&mut self, name: S, method: M)
    where
        S: ToString + AsRef<str>,
        A: FromLuaMulti + TealMultiValue,
        R: ToLuaMulti + TealMultiValue,
        M: 'static + MaybeSend + Fn(&Lua, &T, A) -> Result<R>,
    {
        self.copy_method_docs::<A, R>(name.as_ref(), true);
        self.cont.add_method(name, method)
    }
    #[inline(always)]
    fn add_method_mut<S, A, R, M>(&mut self, name: S, method: M)
    where
        S: ToString + AsRef<str>,
        A: FromLuaMulti + TealMultiValue,
        R: ToLuaMulti + TealMultiValue,
        M: 'static + MaybeSend + FnMut(&Lua, &mut T, A) -> Result<R>,
    {
        self.copy_method_docs::<A, R>(name.as_ref(), true);
        self.cont.add_method_mut(name, method)
    }
    #[cfg(feature = "mlua_async")]
    #[inline(always)]
    fn add_async_method<S: ToString + AsRef<str>, A, R, M, MR>(&mut self, name: S, method: M)
    where
        T: 'static,
        M: Fn(Lua, UserDataRef<T>, A) -> MR + MaybeSend + 'static,
        A: FromLuaMulti + TealMultiValue,
        MR: std::future::Future<Output = Result<R>> + mlua::MaybeSend + 'static,
        R: ToLuaMulti + TealMultiValue,
    {
        self.copy_method_docs::<A, R>(name.as_ref(), true);
        self.cont.add_async_method(name, method)
    }
    #[inline(always)]
    fn add_function<S, A, R, F>(&mut self, name: S, function: F)
    where
        S: ToString + AsRef<str>,
        A: FromLuaMulti + TealMultiValue,
        R: ToLuaMulti + TealMultiValue,
        F: 'static + MaybeSend + Fn(&Lua, A) -> Result<R>,
    {
        self.copy_method_docs::<A, R>(name.as_ref(), false);
        self.cont.add_function(name, function)
    }
    #[inline(always)]
    fn add_function_mut<S, A, R, F>(&mut self, name: S, function: F)
    where
        S: ToString + AsRef<str>,
        A: FromLuaMulti + TealMultiValue,
        R: ToLuaMulti + TealMultiValue,
        F: 'static + MaybeSend + FnMut(&Lua, A) -> Result<R>,
    {
        self.copy_method_docs::<A, R>(name.as_ref(), false);
        self.cont.add_function_mut(name, function)
    }
    #[cfg(feature = "mlua_async")]
    #[inline(always)]
    fn add_async_function<S, A, R, F, FR>(&mut self, name: S, function: F)
    where
        S: AsRef<str> + ToString,
        A: FromLuaMulti + TealMultiValue,
        R: ToLuaMulti + TealMultiValue,
        F: Fn(Lua, A) -> FR + mlua::MaybeSend + 'static,
        FR: std::future::Future<Output = Result<R>> + mlua::MaybeSend + 'static,
    {
        self.copy_method_docs::<A, R>(name.as_ref(), false);
        self.cont.add_async_function(name, function)
    }
    #[inline(always)]
    fn add_meta_method<A, R, M>(&mut self, meta: MetaMethod, method: M)
    where
        A: FromLuaMulti + TealMultiValue,
        R: ToLuaMulti + TealMultiValue,
        M: 'static + MaybeSend + Fn(&Lua, &T, A) -> Result<R>,
    {
        self.copy_method_docs::<A, R>(meta.name(), true);
        self.cont.add_meta_method(meta, method)
    }
    #[inline(always)]
    fn add_meta_method_mut<A, R, M>(&mut self, meta: MetaMethod, method: M)
    where
        A: FromLuaMulti + TealMultiValue,
        R: ToLuaMulti + TealMultiValue,
        M: 'static + MaybeSend + FnMut(&Lua, &mut T, A) -> Result<R>,
    {
        self.copy_method_docs::<A, R>(meta.name(), true);
        self.cont.add_meta_method_mut(meta, method)
    }

    #[inline(always)]
    fn add_meta_function<A, R, F>(&mut self, meta: MetaMethod, function: F)
    where
        A: FromLuaMulti + TealMultiValue,
        R: ToLuaMulti + TealMultiValue,
        F: 'static + MaybeSend + Fn(&Lua, A) -> Result<R>,
    {
        self.copy_method_docs::<A, R>(meta.name(), false);
        self.cont.add_meta_function(meta, function)
    }

    #[inline(always)]
    fn add_meta_function_mut<A, R, F>(&mut self, meta: MetaMethod, function: F)
    where
        A: FromLuaMulti + TealMultiValue,
        R: ToLuaMulti + TealMultiValue,
        F: 'static + MaybeSend + FnMut(&Lua, A) -> Result<R>,
    {
        self.copy_method_docs::<A, R>(meta.name(), false);
        self.cont.add_meta_function_mut(meta, function)
    }

    fn document(&mut self, documentation: &str) -> &mut Self {
        self.document(documentation);
        self
    }
    fn document_type(&mut self, documentation: &str) -> &mut Self {
        self.type_doc.push_str(documentation);
        self.type_doc.push('\n');
        self
    }

    fn generate_help(&mut self) {
        let help = self.documentation.clone();
        let type_doc = self.type_doc.clone();
        self.add_function("help", move |lua, key: Option<mlua::String>| {
            let doc = match key {
                Some(x) => help
                    .get(x.as_bytes().as_bytes())
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
}

impl<Container, T: ToTypename + TealData> TealDataFields<T> for UserDataWrapper<'_, Container, T>
where
    T: UserData,
    Container: UserDataFields<T>,
{
    fn document(&mut self, documentation: &str) {
        self.document(documentation)
    }

    fn add_field_method_get<S, R, M>(&mut self, name: S, method: M)
    where
        S: AsRef<str> + ToString,
        R: mlua::IntoLua + ToTypename,
        M: 'static + MaybeSend + Fn(&Lua, &T) -> mlua::Result<R>,
    {
        self.copy_field_docs::<R>(name.as_ref());
        self.cont.add_field_method_get(name, method)
    }

    fn add_field_method_set<S, A, M>(&mut self, name: S, method: M)
    where
        S: AsRef<str> + ToString,
        A: mlua::FromLua + ToTypename,
        M: 'static + MaybeSend + FnMut(&Lua, &mut T, A) -> mlua::Result<()>,
    {
        self.copy_field_docs::<A>(name.as_ref());
        self.cont.add_field_method_set(name, method)
    }

    fn add_field_function_get<S, R, F>(&mut self, name: S, function: F)
    where
        S: AsRef<str> + ToString,
        R: mlua::IntoLua + ToTypename,
        F: 'static + MaybeSend + Fn(&Lua, mlua::AnyUserData) -> mlua::Result<R>,
    {
        self.copy_field_docs::<R>(name.as_ref());
        self.cont.add_field_function_get(name, function)
    }

    fn add_field_function_set<S, A, F>(&mut self, name: S, function: F)
    where
        S: AsRef<str> + ToString,
        A: mlua::FromLua + ToTypename,
        F: 'static + MaybeSend + FnMut(&Lua, mlua::AnyUserData, A) -> mlua::Result<()>,
    {
        self.copy_field_docs::<A>(name.as_ref());
        self.cont.add_field_function_set(name, function)
    }

    fn add_meta_field_with<R, F>(&mut self, meta: MetaMethod, f: F)
    where
        F: 'static + MaybeSend + Fn(&Lua) -> mlua::Result<R>,
        R: mlua::IntoLua + ToTypename,
    {
        self.copy_field_docs::<R>(meta.name());
        self.cont.add_meta_field_with(meta, f)
    }
}
