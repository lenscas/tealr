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
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        M: 'static + Send + Fn(Context<'lua>, &T, A) -> Result<R>,
    {
        self.copy_docs(name.as_ref());
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
        self.copy_docs(name.as_ref());
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
        self.copy_docs(name.as_ref());
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
        self.copy_docs(name.as_ref());
        self.cont.add_function_mut(name, function)
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
    fn document(&mut self, documentation: &str) {
        match &mut self.next_docs {
            Some(x) => {
                x.push('\n');
                x.push_str(documentation)
            }
            None => self.next_docs = Some(documentation.to_owned()),
        };
    }
    fn generate_help(&mut self) {
        let help = self.documentation.clone();
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
                    let mut y = (b"Available keys:\n").to_vec();
                    y.append(&mut x);
                    y
                }
            };
            lua.create_string(&doc)
        })
    }
}
// impl<'a, 'lua, Container, T> DocumentationCollector for UserDataWrapper<'a, 'lua, Container, T>
// where
//     Container: UserDataMethods<'lua, T> + 'a,
//     T: UserData,
// {
//     fn document_function(&mut self, name: impl Into<String>, documentation: impl Into<String>) {
//         self.documentation.insert(name.into(), documentation.into());
//     }

//     fn should_generate_help_method(&mut self, should_generate: bool) {
//         self.should_generate_help = should_generate;
//     }
// }
// impl<'a, 'lua, Container, T> HelpMethodGenerator for UserDataWrapper<'a, 'lua, Container, T>
// where
//     Container: UserDataMethods<'lua, T> + 'a,
//     T: UserData,
// {
//     fn generate_help(&mut self) {
//         if !self.should_generate_help {
//             return;
//         }
//         let help = self.documentation.clone();
//         self.add_function("help", move |_, key: Option<String>| {
//             Ok(match key {
//                 Some(x) => help.get(&x).map(ToOwned::to_owned).unwrap_or_else(|| {
//                     "The given key is not found. Use `.help()` to list available keys.".into()
//                 }),
//                 None => {
//                     format!(
//                         "Available keys:\n{}",
//                         help.keys()
//                             .map(ToOwned::to_owned)
//                             .collect::<Vec<_>>()
//                             .join("\n")
//                     )
//                 }
//             })
//         })
//     }
// }
