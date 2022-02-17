use std::{borrow::Cow, collections::HashMap, ops::Deref, string::FromUtf8Error};

#[cfg(feature = "rlua")]
use crate::rlu::{
    get_meta_name as get_meta_name_rlua, TealData as TealDataR, TealDataMethods as TealDataMethodsR,
};
#[cfg(feature = "rlua")]
use rlua::{
    Context, FromLuaMulti as FromLuaMultiR, MetaMethod as MetaMethodR, Result as ResultR,
    ToLuaMulti as ToLuaMultiR, UserData as UserDataR,
};
use serde::{Deserialize, Serialize};

#[cfg(feature = "mlua")]
use crate::mlu::{
    get_meta_name as get_meta_name_mlua, MaybeSend, TealData as TealDataM,
    TealDataMethods as TealDataMethodsM,
};
#[cfg(feature = "mlua")]
use mlua::{
    FromLuaMulti as FromLuaMultiM, Lua, MetaMethod as MetaMethodM, Result as ResultM,
    ToLuaMulti as ToLuaMultiM, UserData as UserDataM,
};

use crate::{
    exported_function::ExportedFunction, type_parts_to_str, Direction, NamePart, TypeName,
};

#[cfg(any(feature = "rlua", feature = "mlua"))]
use crate::TealMultiValue;

#[derive(Clone, Hash, PartialEq, Eq)]
///Simple wrapper around `Vec<u8>`
pub struct NameContainer(pub(crate) Vec<u8>);

impl Deref for NameContainer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> Deserialize<'de> for NameContainer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).map(|v| v.into_bytes().into())
    }
}

impl Serialize for NameContainer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        String::from_utf8_lossy(&self.0).serialize(serializer)
    }
}

impl From<Vec<u8>> for NameContainer {
    fn from(s: Vec<u8>) -> Self {
        Self(s)
    }
}

impl From<Cow<'static, str>> for NameContainer {
    fn from(a: Cow<str>) -> Self {
        a.as_bytes().to_owned().into()
    }
}

#[cfg(any(feature = "rlua", feature = "mlua"))]
pub(crate) fn get_method_data<A: TealMultiValue, R: TealMultiValue, S: ?Sized + AsRef<[u8]>>(
    name: &S,
    is_meta_method: bool,
    extra_self: Option<Cow<'static, [NamePart]>>,
) -> ExportedFunction {
    ExportedFunction::new::<A, R, _>(name, is_meta_method, extra_self)
}

///This struct collects all the information needed to create the .d.tl file for your type.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct TypeGenerator {
    ///Represents if the type should be inlined or not.
    pub should_be_inlined: bool,
    ///Represents if the type is UserData
    pub is_user_data: bool,
    ///The name of the type in teal
    pub type_name: Cow<'static, [NamePart]>,
    ///The exposed fields and their types
    pub fields: Vec<(Cow<'static, str>, Cow<'static, str>)>,
    ///exported methods
    pub methods: Vec<ExportedFunction>,
    ///exported methods that mutate something
    pub mut_methods: Vec<ExportedFunction>,
    ///exported functions
    pub functions: Vec<ExportedFunction>,
    ///exported functions that mutate something
    pub mut_functions: Vec<ExportedFunction>,
    ///exported meta_methods
    pub meta_method: Vec<ExportedFunction>,
    ///exported meta_methods that mutate something
    pub meta_method_mut: Vec<ExportedFunction>,
    ///exported meta functions
    pub meta_function: Vec<ExportedFunction>,
    ///exported meta functions that mutate something
    pub meta_function_mut: Vec<ExportedFunction>,
    ///registered documentation
    pub documentation: HashMap<NameContainer, String>,
    ///documentation for this type itself
    pub type_doc: String,
    next_docs: Option<String>,
    ///if this type needs to get a `.help()` function
    pub should_generate_help_method: bool,
}
impl TypeGenerator {
    pub(crate) fn new<A: TypeName>(dir: Direction, should_be_inlined: bool) -> Self {
        Self {
            should_be_inlined,
            is_user_data: false,
            type_name: A::get_type_parts(dir),
            fields: Default::default(),
            methods: Default::default(),
            mut_methods: Default::default(),
            functions: Default::default(),
            mut_functions: Default::default(),
            meta_method: Default::default(),
            meta_method_mut: Default::default(),
            meta_function: Default::default(),
            meta_function_mut: Default::default(),
            documentation: Default::default(),
            should_generate_help_method: true,
            next_docs: Default::default(),
            type_doc: Default::default(),
        }
    }

    pub(crate) fn generate(self) -> std::result::Result<String, FromUtf8Error> {
        //let head = format!("local record {}", self.type_name);
        let type_name = type_parts_to_str(self.type_name);

        let fields: Vec<_> = self
            .fields
            .into_iter()
            .map(|(name, lua_type)| format!("{} : {}", name, lua_type))
            .collect();

        let documentation = &self.documentation;
        let methods: Vec<_> = self
            .methods
            .into_iter()
            .map(|v| v.generate(documentation)) //v.generate(Some(type_name.clone()), documentation))
            .collect::<std::result::Result<_, _>>()?;

        let methods_mut: Vec<_> = self
            .mut_methods
            .into_iter()
            .map(|v| v.generate(documentation))
            .collect::<std::result::Result<_, _>>()?;

        let functions: Vec<_> = self
            .functions
            .into_iter()
            .map(|f| f.generate(documentation))
            .collect::<std::result::Result<_, _>>()?;

        let functions_mut: Vec<_> = self
            .mut_functions
            .into_iter()
            .map(|f| f.generate(documentation))
            .collect::<std::result::Result<_, _>>()?;

        let meta_methods: Vec<_> = self
            .meta_method
            .into_iter()
            .map(|f| f.generate(documentation))
            .collect::<std::result::Result<_, _>>()?;

        let meta_methods_mut: Vec<_> = self
            .meta_method_mut
            .into_iter()
            .map(|f| f.generate(documentation))
            .collect::<std::result::Result<_, _>>()?;

        let meta_function: Vec<_> = self
            .meta_function
            .into_iter()
            .map(|f| f.generate(documentation))
            .collect::<std::result::Result<_, _>>()?;
        let meta_function_mut: Vec<_> = self
            .meta_function_mut
            .into_iter()
            .map(|f| f.generate(documentation))
            .collect::<std::result::Result<_, _>>()?;

        let fields = Self::combine_function_names(fields, "Fields");
        let methods = Self::combine_function_names(methods, "Pure methods");
        let methods_mut = Self::combine_function_names(methods_mut, "Mutating methods");
        let functions = Self::combine_function_names(functions, "Pure functions");
        let functions_mut = Self::combine_function_names(functions_mut, "Mutating functions");
        let meta_methods = Self::combine_function_names(meta_methods, "Meta methods");
        let meta_methods_mut =
            Self::combine_function_names(meta_methods_mut, "Mutating MetaMethods");

        let meta_functions = Self::combine_function_names(meta_function, "Meta functions");
        let meta_functions_mut =
            Self::combine_function_names(meta_function_mut, "Mutating meta functions");

        let userdata_string = if self.is_user_data { "userdata" } else { "" };
        let (type_header, type_end) = if self.should_be_inlined {
            (format!("-- {}\n", type_name), "")
        } else {
            (
                format!(
                    "record {}\n{}",
                    type_name,
                    userdata_string
                        .lines()
                        .map(|v| format!("\t{}\n", v))
                        .collect::<String>()
                ),
                "\tend",
            )
        };
        let type_header = type_header
            .lines()
            .map(|v| format!("\t{}\n", v))
            .collect::<String>();
        let type_docs = self
            .type_doc
            .lines()
            .map(|v| String::from("--") + v + "\n")
            .collect::<String>();
        Ok(format!(
            "{}{}\n{}{}{}{}{}{}{}{}{}\n{}",
            type_docs,
            type_header,
            fields,
            methods,
            methods_mut,
            functions,
            functions_mut,
            meta_methods,
            meta_methods_mut,
            meta_functions,
            meta_functions_mut,
            type_end
        ))
    }
    fn combine_function_names<T: AsRef<str>>(function_list: Vec<T>, top_doc: &str) -> String {
        if function_list.is_empty() {
            "".into()
        } else {
            let combined = function_list
                .into_iter()
                .map(|v| {
                    v.as_ref()
                        .lines()
                        .map(|v| String::from("\t\t") + v)
                        .map(|mut v| {
                            v.push('\n');
                            v
                        })
                        .collect::<String>()
                })
                .collect::<Vec<_>>()
                .join("\n");
            format!("\t\t-- {}\n{}\n", top_doc, combined)
        }
    }
}

impl TypeGenerator {
    fn copy_docs(&mut self, to: &[u8]) {
        if let Some(x) = self.next_docs.take() {
            self.documentation.insert(to.to_owned().into(), x);
        }
    }
}

#[cfg(feature = "rlua")]
impl<'lua, T> TealDataMethodsR<'lua, T> for TypeGenerator
where
    T: 'static + TealDataR + UserDataR + TypeName,
{
    fn add_method<S, A, R, M>(&mut self, name: &S, _: M)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMultiR<'lua> + TealMultiValue,
        R: ToLuaMultiR<'lua> + TealMultiValue,
        M: 'static + Send + Fn(Context<'lua>, &T, A) -> ResultR<R>,
    {
        self.copy_docs(name.as_ref());
        self.methods.push(get_method_data::<A, R, _>(
            name,
            false,
            Some(T::get_type_parts(Direction::FromLua)),
        ))
    }

    fn add_method_mut<S, A, R, M>(&mut self, name: &S, _: M)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMultiR<'lua> + TealMultiValue,
        R: ToLuaMultiR<'lua> + TealMultiValue,
        M: 'static + Send + FnMut(Context<'lua>, &mut T, A) -> ResultR<R>,
    {
        self.copy_docs(name.as_ref());
        self.mut_methods.push(get_method_data::<A, R, _>(
            name,
            false,
            Some(T::get_type_parts(Direction::FromLua)),
        ))
    }

    fn add_function<S, A, R, F>(&mut self, name: &S, _: F)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMultiR<'lua> + TealMultiValue,
        R: ToLuaMultiR<'lua> + TealMultiValue,
        F: 'static + Send + Fn(Context<'lua>, A) -> ResultR<R>,
    {
        self.copy_docs(name.as_ref());
        self.functions
            .push(get_method_data::<A, R, _>(name, false, None))
    }

    fn add_function_mut<S, A, R, F>(&mut self, name: &S, _: F)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMultiR<'lua> + TealMultiValue,
        R: ToLuaMultiR<'lua> + TealMultiValue,
        F: 'static + Send + FnMut(Context<'lua>, A) -> ResultR<R>,
    {
        self.copy_docs(name.as_ref());
        self.mut_functions
            .push(get_method_data::<A, R, _>(name, false, None))
    }

    fn add_meta_method<A, R, M>(&mut self, name: MetaMethodR, _: M)
    where
        A: FromLuaMultiR<'lua> + TealMultiValue,
        R: ToLuaMultiR<'lua> + TealMultiValue,
        M: 'static + Send + Fn(Context<'lua>, &T, A) -> ResultR<R>,
    {
        self.copy_docs(get_meta_name_rlua(name).as_bytes());
        self.meta_method.push(get_method_data::<A, R, _>(
            get_meta_name_rlua(name),
            false,
            Some(T::get_type_parts(Direction::FromLua)),
        ))
    }

    fn add_meta_method_mut<A, R, M>(&mut self, name: MetaMethodR, _: M)
    where
        A: FromLuaMultiR<'lua> + TealMultiValue,
        R: ToLuaMultiR<'lua> + TealMultiValue,
        M: 'static + Send + FnMut(Context<'lua>, &mut T, A) -> ResultR<R>,
    {
        self.copy_docs(get_meta_name_rlua(name).as_bytes());
        self.meta_method_mut.push(get_method_data::<A, R, _>(
            get_meta_name_rlua(name),
            false,
            Some(T::get_type_parts(Direction::FromLua)),
        ))
    }

    fn add_meta_function<A, R, F>(&mut self, name: MetaMethodR, _: F)
    where
        A: FromLuaMultiR<'lua> + TealMultiValue,
        R: ToLuaMultiR<'lua> + TealMultiValue,
        F: 'static + Send + Fn(Context<'lua>, A) -> ResultR<R>,
    {
        self.copy_docs(get_meta_name_rlua(name).as_bytes());
        self.meta_function.push(get_method_data::<A, R, _>(
            get_meta_name_rlua(name),
            false,
            None,
        ))
    }

    fn add_meta_function_mut<A, R, F>(&mut self, name: MetaMethodR, _: F)
    where
        A: FromLuaMultiR<'lua> + TealMultiValue,
        R: ToLuaMultiR<'lua> + TealMultiValue,
        F: 'static + Send + FnMut(Context<'lua>, A) -> ResultR<R>,
    {
        self.copy_docs(get_meta_name_rlua(name).as_bytes());
        self.meta_function_mut.push(get_method_data::<A, R, _>(
            get_meta_name_rlua(name),
            false,
            None,
        ))
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
        self.functions
            .push(get_method_data::<Option<String>, String, _>(
                "help", false, None,
            ))
    }

    fn document_type(&mut self, documentation: &str) {
        self.type_doc.push_str(documentation);
        self.type_doc.push('\n');
        self.type_doc.push('\n');
    }
}

#[cfg(feature = "mlua")]
impl<'lua, T> TealDataMethodsM<'lua, T> for TypeGenerator
where
    T: 'static + TealDataM + UserDataM + TypeName,
{
    fn add_method<S, A, R, M>(&mut self, name: &S, _: M)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMultiM<'lua> + TealMultiValue,
        R: ToLuaMultiM<'lua> + TealMultiValue,
        M: 'static + MaybeSend + Fn(&'lua Lua, &T, A) -> ResultM<R>,
    {
        self.copy_docs(name.as_ref());
        self.methods.push(get_method_data::<A, R, _>(
            name,
            false,
            Some(T::get_type_parts(Direction::FromLua)),
        ))
    }

    fn add_method_mut<S, A, R, M>(&mut self, name: &S, _: M)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMultiM<'lua> + TealMultiValue,
        R: ToLuaMultiM<'lua> + TealMultiValue,
        M: 'static + MaybeSend + FnMut(&'lua Lua, &mut T, A) -> ResultM<R>,
    {
        self.copy_docs(name.as_ref());
        self.mut_methods.push(get_method_data::<A, R, _>(
            name,
            false,
            Some(T::get_type_parts(Direction::FromLua)),
        ))
    }

    fn add_function<S, A, R, F>(&mut self, name: &S, _: F)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMultiM<'lua> + TealMultiValue,
        R: ToLuaMultiM<'lua> + TealMultiValue,
        F: 'static + MaybeSend + Fn(&'lua Lua, A) -> ResultM<R>,
    {
        self.copy_docs(name.as_ref());
        self.functions
            .push(get_method_data::<A, R, _>(name, false, None))
    }

    fn add_function_mut<S, A, R, F>(&mut self, name: &S, _: F)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMultiM<'lua> + TealMultiValue,
        R: ToLuaMultiM<'lua> + TealMultiValue,
        F: 'static + MaybeSend + FnMut(&'lua Lua, A) -> ResultM<R>,
    {
        self.copy_docs(name.as_ref());
        self.mut_functions
            .push(get_method_data::<A, R, _>(name, false, None))
    }

    fn add_meta_method<A, R, M>(&mut self, name: MetaMethodM, _: M)
    where
        A: FromLuaMultiM<'lua> + TealMultiValue,
        R: ToLuaMultiM<'lua> + TealMultiValue,
        M: 'static + MaybeSend + Fn(&'lua Lua, &T, A) -> ResultM<R>,
    {
        self.copy_docs(name.name().as_bytes());
        self.meta_method.push(get_method_data::<A, R, _>(
            &get_meta_name_mlua(name).as_bytes(),
            true,
            Some(T::get_type_parts(Direction::FromLua)),
        ))
    }

    fn add_meta_method_mut<A, R, M>(&mut self, name: MetaMethodM, _: M)
    where
        A: FromLuaMultiM<'lua> + TealMultiValue,
        R: ToLuaMultiM<'lua> + TealMultiValue,
        M: 'static + MaybeSend + FnMut(&'lua Lua, &mut T, A) -> ResultM<R>,
    {
        self.copy_docs(name.name().as_bytes());
        self.meta_method_mut.push(get_method_data::<A, R, _>(
            &get_meta_name_mlua(name).as_bytes(),
            true,
            Some(T::get_type_parts(Direction::FromLua)),
        ))
    }

    fn add_meta_function<A, R, F>(&mut self, name: MetaMethodM, _: F)
    where
        A: FromLuaMultiM<'lua> + TealMultiValue,
        R: ToLuaMultiM<'lua> + TealMultiValue,
        F: 'static + MaybeSend + Fn(&'lua Lua, A) -> ResultM<R>,
    {
        self.copy_docs(name.name().as_bytes());
        self.meta_function.push(get_method_data::<A, R, _>(
            get_meta_name_mlua(name).as_bytes(),
            true,
            None,
        ))
    }

    fn add_meta_function_mut<A, R, F>(&mut self, name: MetaMethodM, _: F)
    where
        A: FromLuaMultiM<'lua> + TealMultiValue,
        R: ToLuaMultiM<'lua> + TealMultiValue,
        F: 'static + MaybeSend + FnMut(&'lua Lua, A) -> ResultM<R>,
    {
        self.copy_docs(name.name().as_bytes());
        self.meta_function_mut.push(get_method_data::<A, R, _>(
            &get_meta_name_mlua(name).as_bytes(),
            true,
            None,
        ))
    }
    #[cfg(feature = "mlua_async")]
    fn add_async_method<S: ?Sized, A, R, M, MR>(&mut self, name: &S, _: M)
    where
        T: Clone,
        S: AsRef<[u8]>,
        A: FromLuaMultiM<'lua> + TealMultiValue,
        R: ToLuaMultiM<'lua> + TealMultiValue,
        M: 'static + MaybeSend + Fn(&'lua Lua, T, A) -> MR,
        MR: 'lua + std::future::Future<Output = ResultM<R>>,
    {
        self.copy_docs(name.as_ref());
        self.methods.push(get_method_data::<A, R, _>(
            name,
            false,
            Some(T::get_type_parts(Direction::FromLua)),
        ))
    }

    #[cfg(feature = "mlua_async")]
    fn add_async_function<S: ?Sized, A, R, F, FR>(&mut self, name: &S, _: F)
    where
        S: AsRef<[u8]>,
        A: FromLuaMultiM<'lua> + TealMultiValue,
        R: ToLuaMultiM<'lua> + TealMultiValue,
        F: 'static + MaybeSend + Fn(&'lua Lua, A) -> FR,
        FR: 'lua + std::future::Future<Output = ResultM<R>>,
    {
        self.copy_docs(name.as_ref());
        self.functions
            .push(get_method_data::<A, R, _>(name, false, None))
    }

    fn document(&mut self, documentation: &str) {
        match &mut self.next_docs {
            Some(x) => {
                x.push('\n');
                x.push('\n');
                x.push_str(documentation)
            }
            None => self.next_docs = Some(documentation.to_owned()),
        };
    }
    fn generate_help(&mut self) {
        self.functions
            .push(get_method_data::<Option<String>, String, _>(
                "help", false, None,
            ))
    }

    fn document_type(&mut self, documentation: &str) {
        self.type_doc.push_str(documentation);
        self.type_doc.push('\n');
        self.type_doc.push('\n');
    }
}
