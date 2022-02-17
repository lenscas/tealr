use std::{collections::HashMap, marker::PhantomData};

use bstr::ByteVec;
use mlua::{FromLuaMulti, Lua, MetaMethod, Result, ToLuaMulti, UserData, UserDataMethods};

use super::{MaybeSend, TealDataMethods};
use crate::{type_generator::get_method_data, Direction, TealMultiValue, TypeName};

///Used to turn [UserDataMethods](mlua::UserDataMethods) into [TealDataMethods](crate::mlu::TealDataMethods).
///
///This allows you to easily implement [UserData](mlua::UserData) by wrapping the [UserDataMethods](mlua::UserDataMethods) in this struct
///and then passing it to the TealData implementation
///
pub struct UserDataWrapper<'a, 'lua, Container, T>
where
    Container: UserDataMethods<'lua, T>,
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
    ///wraps it.
    ///```
    ///# use std::borrow::Cow;
    ///# use mlua::{Lua, Result, UserData, UserDataMethods};
    ///# use tealr::{mlu::{TealData, TealDataMethods,UserDataWrapper}, TypeWalker, Direction, TypeName,NamePart,TealType, KindOfType};
    /// struct Example {}
    /// impl TealData for Example {}
    /// impl TypeName for Example {
    ///     fn get_type_parts(dir: Direction) -> Cow<'static, [NamePart]> {
    ///         Cow::Borrowed(&[
    ///             NamePart::Type(TealType{
    ///                 name: Cow::Borrowed("Example"),
    ///                 type_kind: KindOfType::External,
    ///                 generics: None
    ///             })
    ///         ])
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
            documentation: HashMap::new(),
            next_docs: Default::default(),
            type_doc: Default::default(),
        }
    }
}
impl<'a, 'lua, Container, T: TypeName> UserDataWrapper<'a, 'lua, Container, T>
where
    T: UserData,
    Container: UserDataMethods<'lua, T>,
{
    fn copy_docs<A, R>(&mut self, to: &[u8], self_type: bool)
    where
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
    {
        let type_def = get_method_data::<A, R, _>(
            to,
            false,
            self_type.then(|| T::get_type_parts(Direction::FromLua)),
        );
        let generated = type_def
            .generate(&Default::default())
            .map(|v| "Signature: ".to_string() + &v)
            .unwrap_or_default();
        let docs = generated + "\n\ndocs:\n" + &self.next_docs.take().unwrap_or_default();
        self.documentation.insert(to.to_owned(), docs);
    }
}

impl<'a, 'lua, Container, T: TypeName> TealDataMethods<'lua, T>
    for UserDataWrapper<'a, 'lua, Container, T>
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
        M: 'static + MaybeSend + Fn(&'lua Lua, &T, A) -> Result<R>,
    {
        self.copy_docs::<A, R>(name.as_ref(), true);
        self.cont.add_method(name, method)
    }
    #[inline(always)]
    fn add_method_mut<S, A, R, M>(&mut self, name: &S, method: M)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        M: 'static + MaybeSend + FnMut(&'lua Lua, &mut T, A) -> Result<R>,
    {
        self.copy_docs::<A, R>(name.as_ref(), true);
        self.cont.add_method_mut(name, method)
    }
    #[inline(always)]
    fn add_function<S, A, R, F>(&mut self, name: &S, function: F)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        F: 'static + MaybeSend + Fn(&'lua Lua, A) -> Result<R>,
    {
        self.copy_docs::<A, R>(name.as_ref(), false);
        self.cont.add_function(name, function)
    }
    #[inline(always)]
    fn add_function_mut<S, A, R, F>(&mut self, name: &S, function: F)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        F: 'static + MaybeSend + FnMut(&'lua Lua, A) -> Result<R>,
    {
        self.copy_docs::<A, R>(name.as_ref(), false);
        self.cont.add_function_mut(name, function)
    }
    #[inline(always)]
    fn add_meta_method<A, R, M>(&mut self, meta: MetaMethod, method: M)
    where
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        M: 'static + MaybeSend + Fn(&'lua Lua, &T, A) -> Result<R>,
    {
        self.copy_docs::<A, R>(meta.name().as_bytes(), true);
        self.cont.add_meta_method(meta, method)
    }
    #[inline(always)]
    fn add_meta_method_mut<A, R, M>(&mut self, meta: MetaMethod, method: M)
    where
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        M: 'static + MaybeSend + FnMut(&'lua Lua, &mut T, A) -> Result<R>,
    {
        self.copy_docs::<A, R>(meta.name().as_bytes(), true);
        self.cont.add_meta_method_mut(meta, method)
    }
    #[inline(always)]
    fn add_meta_function<A, R, F>(&mut self, meta: MetaMethod, function: F)
    where
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        F: 'static + MaybeSend + Fn(&'lua Lua, A) -> Result<R>,
    {
        self.copy_docs::<A, R>(meta.name().as_bytes(), false);
        self.cont.add_meta_function(meta, function)
    }
    #[inline(always)]
    fn add_meta_function_mut<A, R, F>(&mut self, meta: MetaMethod, function: F)
    where
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        F: 'static + MaybeSend + FnMut(&'lua Lua, A) -> Result<R>,
    {
        self.copy_docs::<A, R>(meta.name().as_bytes(), false);
        self.cont.add_meta_function_mut(meta, function)
    }

    #[cfg(feature = "mlua_async")]
    #[inline(always)]
    fn add_async_method<S: ?Sized, A, R, M, MR>(&mut self, name: &S, method: M)
    where
        T: Clone,
        S: AsRef<[u8]>,
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        M: 'static + MaybeSend + Fn(&'lua Lua, T, A) -> MR,
        MR: 'lua + std::future::Future<Output = Result<R>>,
    {
        self.copy_docs::<A, R>(name.as_ref(), true);
        self.cont.add_async_method(name, method)
    }

    #[cfg(feature = "mlua_async")]
    #[inline(always)]
    fn add_async_function<S: ?Sized, A, R, F, FR>(&mut self, name: &S, function: F)
    where
        S: AsRef<[u8]>,
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        F: 'static + MaybeSend + Fn(&'lua Lua, A) -> FR,
        FR: 'lua + std::future::Future<Output = Result<R>>,
    {
        self.copy_docs::<A, R>(name.as_ref(), false);
        self.cont.add_async_function(name, function)
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

            lua.create_string(&doc)
        })
    }

    fn document_type(&mut self, documentation: &str) {
        self.type_doc.push_str(documentation);
        self.type_doc.push('\n')
    }
}
