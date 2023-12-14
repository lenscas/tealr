use std::{collections::HashMap, marker::PhantomData};

use bstr::ByteVec;
use rlua::{Context, FromLuaMulti, MetaMethod, Result, ToLuaMulti, UserData, UserDataMethods};

use super::{get_meta_name, TealDataMethods};
use crate::TealMultiValue;

///Used to turn [UserDataMethods](rlua::UserDataMethods) into [TealDataMethods](crate::rlu::TealDataMethods).
///
///This allows you to easily implement [UserData](rlua::UserData) by wrapping the [UserDataMethods](rlua::UserDataMethods) in this struct
///and then passing it to the TealData implementation
///
pub struct UserDataWrapper<'a, 'lua, Container, T>
where
    Container: UserDataMethods<'lua, T>,
    T: UserData,
{
    cont: &'a mut Container,
    _t: PhantomData<(&'a (), T)>,
    _x: &'lua PhantomData<()>,
    documentation: HashMap<Vec<u8>, String>,
    type_doc: String,
    next_docs: Option<String>,
}
impl<'a, 'lua, Container, T> UserDataWrapper<'a, 'lua, Container, T>
where
    Container: UserDataMethods<'lua, T> + 'a,
    T: UserData,
{
    ///wraps it.
    ///```
    ///# use rlua::{Lua, Result, UserData, UserDataMethods};
    ///# use tealr::{rlu::{TealData, TealDataMethods,UserDataWrapper}, TypeWalker,  TypeName,};
    /// struct Example {}
    /// impl TealData for Example {}
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
            _t: std::marker::PhantomData,
            _x: &std::marker::PhantomData,
            documentation: Default::default(),
            type_doc: Default::default(),
            next_docs: Default::default(),
        }
    }
}
impl<'a, 'lua, Container, T> UserDataWrapper<'a, 'lua, Container, T>
where
    T: UserData,
    Container: UserDataMethods<'lua, T>,
{
    fn copy_docs(&mut self, to: &[u8]) {
        if let Some(x) = self.next_docs.take() {
            self.documentation.insert(to.to_owned(), x);
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
        S: ?Sized + AsRef<str>,
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        M: 'static + Send + Fn(Context<'lua>, &T, A) -> Result<R>,
    {
        self.copy_docs(name.as_ref().as_bytes());
        self.cont.add_method(name.as_ref().as_bytes(), method)
    }
    #[inline(always)]
    fn add_method_mut<S, A, R, M>(&mut self, name: &S, method: M)
    where
        S: ?Sized + AsRef<str>,
        A: FromLuaMulti<'lua>,
        R: ToLuaMulti<'lua>,
        M: 'static + Send + FnMut(Context<'lua>, &mut T, A) -> Result<R>,
    {
        self.copy_docs(name.as_ref().as_bytes());
        self.cont.add_method_mut(name.as_ref().as_bytes(), method)
    }
    #[inline(always)]
    fn add_function<S, A, R, F>(&mut self, name: &S, function: F)
    where
        S: ?Sized + AsRef<str>,
        A: FromLuaMulti<'lua>,
        R: ToLuaMulti<'lua>,
        F: 'static + Send + Fn(Context<'lua>, A) -> Result<R>,
    {
        self.copy_docs(name.as_ref().as_bytes());
        self.cont.add_function(name.as_ref().as_bytes(), function)
    }
    #[inline(always)]
    fn add_function_mut<S, A, R, F>(&mut self, name: &S, function: F)
    where
        S: ?Sized + AsRef<str>,
        A: FromLuaMulti<'lua>,
        R: ToLuaMulti<'lua>,
        F: 'static + Send + FnMut(Context<'lua>, A) -> Result<R>,
    {
        self.copy_docs(name.as_ref().as_bytes());
        self.cont
            .add_function_mut(name.as_ref().as_bytes(), function)
    }
    #[inline(always)]
    fn add_meta_method<A, R, M>(&mut self, meta: MetaMethod, method: M)
    where
        A: FromLuaMulti<'lua>,
        R: ToLuaMulti<'lua>,
        M: 'static + Send + Fn(Context<'lua>, &T, A) -> Result<R>,
    {
        self.copy_docs(get_meta_name(meta).as_bytes());
        self.cont.add_meta_method(meta, method)
    }
    #[inline(always)]
    fn add_meta_method_mut<A, R, M>(&mut self, meta: MetaMethod, method: M)
    where
        A: FromLuaMulti<'lua>,
        R: ToLuaMulti<'lua>,
        M: 'static + Send + FnMut(Context<'lua>, &mut T, A) -> Result<R>,
    {
        self.copy_docs(get_meta_name(meta).as_bytes());
        self.cont.add_meta_method_mut(meta, method)
    }
    #[inline(always)]
    fn add_meta_function<A, R, F>(&mut self, meta: MetaMethod, function: F)
    where
        A: FromLuaMulti<'lua>,
        R: ToLuaMulti<'lua>,
        F: 'static + Send + Fn(Context<'lua>, A) -> Result<R>,
    {
        self.copy_docs(get_meta_name(meta).as_bytes());
        self.cont.add_meta_function(meta, function)
    }
    #[inline(always)]
    fn add_meta_function_mut<A, R, F>(&mut self, meta: MetaMethod, function: F)
    where
        A: FromLuaMulti<'lua>,
        R: ToLuaMulti<'lua>,
        F: 'static + Send + FnMut(Context<'lua>, A) -> Result<R>,
    {
        self.copy_docs(get_meta_name(meta).as_bytes());
        self.cont.add_meta_function_mut(meta, function)
    }
    fn document(&mut self, documentation: &str) -> &mut Self {
        match &mut self.next_docs {
            Some(x) => {
                x.push('\n');
                x.push_str(documentation)
            }
            None => self.next_docs = Some(documentation.to_owned()),
        };
        self
    }
    fn generate_help(&mut self) {
        let help = self.documentation.clone();
        let type_doc = self.type_doc.clone();
        self.add_function("help", move |lua, key: Option<rlua::String>| {
            let doc = match key {
                Some(x) => help
                    .get(x.as_bytes())
                    .map(|v| v.as_bytes())
                    .map(ToOwned::to_owned)
                    .unwrap_or_else(|| {
                        b"The given key is not found. Use `.help()` to list available keys."
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
            lua.create_string(&doc)
        })
    }

    fn document_type(&mut self, documentation: &str) -> &mut Self {
        self.type_doc.push_str(documentation);
        self.type_doc.push('\n');
        self
    }
}
