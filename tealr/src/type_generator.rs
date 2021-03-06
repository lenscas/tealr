use std::{borrow::Cow, string::FromUtf8Error};

#[cfg(feature = "rlua")]
use crate::rlu::{
    get_meta_name as get_meta_name_rlua, TealData as TealDataR, TealDataMethods as TealDataMethodsR,
};
#[cfg(feature = "rlua")]
use rlua::{
    Context, FromLuaMulti as FromLuaMultiR, MetaMethod as MetaMethodR, Result as ResultR,
    ToLuaMulti as ToLuaMultiR, UserData as UserDataR,
};

#[cfg(feature = "mlua")]
use crate::mlu::{
    get_meta_name as get_meta_name_mlua, TealData as TealDataM, TealDataMethods as TealDataMethodsM,
};
#[cfg(feature = "mlua")]
use mlua::{
    FromLuaMulti as FromLuaMultiM, Lua, MetaMethod as MetaMethodM, Result as ResultM,
    ToLuaMulti as ToLuaMultiM, UserData as UserDataM,
};

use crate::{Direction, ExportedFunction, TypeName};

#[cfg(any(feature = "rlua", feature = "mlua"))]
use crate::TealMultiValue;

///This struct collects all the information needed to create the .d.tl file for your type.
pub struct TypeGenerator {
    ///Represents if the type is UserData
    pub is_user_data: bool,
    ///The name of the type in teal
    pub type_name: Cow<'static, str>,
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
}
impl TypeGenerator {
    pub(crate) fn new<A: TypeName>(dir: Direction) -> Self {
        Self {
            is_user_data: false,
            type_name: A::get_type_name(dir),
            fields: Default::default(),
            methods: Default::default(),
            mut_methods: Default::default(),
            functions: Default::default(),
            mut_functions: Default::default(),
            meta_method: Default::default(),
            meta_method_mut: Default::default(),
            meta_function: Default::default(),
            meta_function_mut: Default::default(),
        }
    }
    #[cfg(feature = "rlua")]
    pub(crate) fn get_method_data_rlua<
        'lua,
        A: TealMultiValue,
        R: ToLuaMultiR<'lua> + TealMultiValue,
        S: ?Sized + AsRef<[u8]>,
    >(
        name: &S,
        is_meta_method: bool,
    ) -> ExportedFunction {
        ExportedFunction {
            name: name.as_ref().to_vec(),
            params: A::get_types(Direction::FromLua),
            returns: R::get_types(Direction::ToLua),
            is_meta_method,
        }
    }
    #[cfg(feature = "mlua")]
    fn get_method_data_mlua<
        'lua,
        A: TealMultiValue,
        R: ToLuaMultiM<'lua> + TealMultiValue,
        S: ?Sized + AsRef<[u8]>,
    >(
        name: &S,
        is_meta_method: bool,
    ) -> ExportedFunction {
        ExportedFunction {
            name: name.as_ref().to_vec(),
            params: A::get_types(Direction::FromLua),
            returns: R::get_types(Direction::ToLua),
            is_meta_method,
        }
    }
    pub(crate) fn generate(self) -> std::result::Result<String, FromUtf8Error> {
        //let head = format!("local record {}", self.type_name);
        let type_name = self.type_name.clone();

        let fields: Vec<_> = self
            .fields
            .into_iter()
            .map(|(name, lua_type)| format!("{} : {}", name, lua_type))
            .collect();

        let methods: Vec<_> = self
            .methods
            .into_iter()
            .map(|v| v.generate(Some(type_name.clone())))
            .collect::<std::result::Result<_, _>>()?;

        let methods_mut: Vec<_> = self
            .mut_methods
            .into_iter()
            .map(|v| v.generate(Some(type_name.clone())))
            .collect::<std::result::Result<_, _>>()?;

        let functions: Vec<_> = self
            .functions
            .into_iter()
            .map(|f| f.generate(None))
            .collect::<std::result::Result<_, _>>()?;

        let functions_mut: Vec<_> = self
            .mut_functions
            .into_iter()
            .map(|f| f.generate(None))
            .collect::<std::result::Result<_, _>>()?;

        let meta_methods: Vec<_> = self
            .meta_method
            .into_iter()
            .map(|f| f.generate(Some(type_name.clone())))
            .collect::<std::result::Result<_, _>>()?;

        let meta_methods_mut: Vec<_> = self
            .meta_method_mut
            .into_iter()
            .map(|f| f.generate(Some(type_name.clone())))
            .collect::<std::result::Result<_, _>>()?;

        let meta_function: Vec<_> = self
            .meta_function
            .into_iter()
            .map(|f| f.generate(None))
            .collect::<std::result::Result<_, _>>()?;
        let meta_function_mut: Vec<_> = self
            .meta_function_mut
            .into_iter()
            .map(|f| f.generate(None))
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
        Ok(format!(
            "\trecord {}\n\t\t{}\n{}{}{}{}{}{}{}{}{}\n\tend",
            self.type_name,
            userdata_string,
            fields,
            methods,
            methods_mut,
            functions,
            functions_mut,
            meta_methods,
            meta_methods_mut,
            meta_functions,
            meta_functions_mut
        ))
    }
    fn combine_function_names(function_list: Vec<String>, top_doc: &str) -> String {
        if function_list.is_empty() {
            "".into()
        } else {
            let combined = function_list
                .into_iter()
                .map(|v| String::from("\t\t") + &v)
                .collect::<Vec<_>>()
                .join("\n");
            format!("\t\t-- {}\n{}\n", top_doc, combined)
        }
    }
}
#[cfg(feature = "rlua")]
impl<'lua, T> TealDataMethodsR<'lua, T> for TypeGenerator
where
    T: 'static + TealDataR + UserDataR,
{
    fn add_method<S, A, R, M>(&mut self, name: &S, _: M)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMultiR<'lua> + TealMultiValue,
        R: ToLuaMultiR<'lua> + TealMultiValue,
        M: 'static + Send + Fn(Context<'lua>, &T, A) -> ResultR<R>,
    {
        self.methods
            .push(Self::get_method_data_rlua::<A, R, _>(name, false))
    }

    fn add_method_mut<S, A, R, M>(&mut self, name: &S, _: M)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMultiR<'lua> + TealMultiValue,
        R: ToLuaMultiR<'lua> + TealMultiValue,
        M: 'static + Send + FnMut(Context<'lua>, &mut T, A) -> ResultR<R>,
    {
        self.mut_methods
            .push(Self::get_method_data_rlua::<A, R, _>(name, false))
    }

    fn add_function<S, A, R, F>(&mut self, name: &S, _: F)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMultiR<'lua> + TealMultiValue,
        R: ToLuaMultiR<'lua> + TealMultiValue,
        F: 'static + Send + Fn(Context<'lua>, A) -> ResultR<R>,
    {
        self.functions
            .push(Self::get_method_data_rlua::<A, R, _>(name, false))
    }

    fn add_function_mut<S, A, R, F>(&mut self, name: &S, _: F)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMultiR<'lua> + TealMultiValue,
        R: ToLuaMultiR<'lua> + TealMultiValue,
        F: 'static + Send + FnMut(Context<'lua>, A) -> ResultR<R>,
    {
        self.mut_functions
            .push(Self::get_method_data_rlua::<A, R, _>(name, false))
    }

    fn add_meta_method<A, R, M>(&mut self, name: MetaMethodR, _: M)
    where
        A: FromLuaMultiR<'lua> + TealMultiValue,
        R: ToLuaMultiR<'lua> + TealMultiValue,
        M: 'static + Send + Fn(Context<'lua>, &T, A) -> ResultR<R>,
    {
        self.meta_method.push(Self::get_method_data_rlua::<A, R, _>(
            get_meta_name_rlua(name),
            true,
        ))
    }

    fn add_meta_method_mut<A, R, M>(&mut self, name: MetaMethodR, _: M)
    where
        A: FromLuaMultiR<'lua> + TealMultiValue,
        R: ToLuaMultiR<'lua> + TealMultiValue,
        M: 'static + Send + FnMut(Context<'lua>, &mut T, A) -> ResultR<R>,
    {
        self.meta_method_mut
            .push(Self::get_method_data_rlua::<A, R, _>(
                get_meta_name_rlua(name),
                true,
            ))
    }

    fn add_meta_function<A, R, F>(&mut self, name: MetaMethodR, _: F)
    where
        A: FromLuaMultiR<'lua> + TealMultiValue,
        R: ToLuaMultiR<'lua> + TealMultiValue,
        F: 'static + Send + Fn(Context<'lua>, A) -> ResultR<R>,
    {
        self.meta_function
            .push(Self::get_method_data_rlua::<A, R, _>(
                get_meta_name_rlua(name),
                true,
            ))
    }

    fn add_meta_function_mut<A, R, F>(&mut self, name: MetaMethodR, _: F)
    where
        A: FromLuaMultiR<'lua> + TealMultiValue,
        R: ToLuaMultiR<'lua> + TealMultiValue,
        F: 'static + Send + FnMut(Context<'lua>, A) -> ResultR<R>,
    {
        self.meta_function_mut
            .push(Self::get_method_data_rlua::<A, R, _>(
                get_meta_name_rlua(name),
                true,
            ))
    }
}

#[cfg(feature = "mlua")]
impl<'lua, T> TealDataMethodsM<'lua, T> for TypeGenerator
where
    T: 'static + TealDataM + UserDataM,
{
    fn add_method<S, A, R, M>(&mut self, name: &S, _: M)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMultiM<'lua> + TealMultiValue,
        R: ToLuaMultiM<'lua> + TealMultiValue,
        M: 'static + Send + Fn(&'lua Lua, &T, A) -> ResultM<R>,
    {
        self.methods
            .push(Self::get_method_data_mlua::<A, R, _>(name, false))
    }

    fn add_method_mut<S, A, R, M>(&mut self, name: &S, _: M)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMultiM<'lua> + TealMultiValue,
        R: ToLuaMultiM<'lua> + TealMultiValue,
        M: 'static + Send + FnMut(&'lua Lua, &mut T, A) -> ResultM<R>,
    {
        self.mut_methods
            .push(Self::get_method_data_mlua::<A, R, _>(name, false))
    }

    fn add_function<S, A, R, F>(&mut self, name: &S, _: F)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMultiM<'lua> + TealMultiValue,
        R: ToLuaMultiM<'lua> + TealMultiValue,
        F: 'static + Send + Fn(&'lua Lua, A) -> ResultM<R>,
    {
        self.functions
            .push(Self::get_method_data_mlua::<A, R, _>(name, false))
    }

    fn add_function_mut<S, A, R, F>(&mut self, name: &S, _: F)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMultiM<'lua> + TealMultiValue,
        R: ToLuaMultiM<'lua> + TealMultiValue,
        F: 'static + Send + FnMut(&'lua Lua, A) -> ResultM<R>,
    {
        self.mut_functions
            .push(Self::get_method_data_mlua::<A, R, _>(name, false))
    }

    fn add_meta_method<A, R, M>(&mut self, name: MetaMethodM, _: M)
    where
        A: FromLuaMultiM<'lua> + TealMultiValue,
        R: ToLuaMultiM<'lua> + TealMultiValue,
        M: 'static + Send + Fn(&'lua Lua, &T, A) -> ResultM<R>,
    {
        self.meta_method.push(Self::get_method_data_mlua::<A, R, _>(
            get_meta_name_mlua(name),
            true,
        ))
    }

    fn add_meta_method_mut<A, R, M>(&mut self, name: MetaMethodM, _: M)
    where
        A: FromLuaMultiM<'lua> + TealMultiValue,
        R: ToLuaMultiM<'lua> + TealMultiValue,
        M: 'static + Send + FnMut(&'lua Lua, &mut T, A) -> ResultM<R>,
    {
        self.meta_method_mut
            .push(Self::get_method_data_mlua::<A, R, _>(
                get_meta_name_mlua(name),
                true,
            ))
    }

    fn add_meta_function<A, R, F>(&mut self, name: MetaMethodM, _: F)
    where
        A: FromLuaMultiM<'lua> + TealMultiValue,
        R: ToLuaMultiM<'lua> + TealMultiValue,
        F: 'static + Send + Fn(&'lua Lua, A) -> ResultM<R>,
    {
        self.meta_function
            .push(Self::get_method_data_mlua::<A, R, _>(
                get_meta_name_mlua(name),
                true,
            ))
    }

    fn add_meta_function_mut<A, R, F>(&mut self, name: MetaMethodM, _: F)
    where
        A: FromLuaMultiM<'lua> + TealMultiValue,
        R: ToLuaMultiM<'lua> + TealMultiValue,
        F: 'static + Send + FnMut(&'lua Lua, A) -> ResultM<R>,
    {
        self.meta_function_mut
            .push(Self::get_method_data_mlua::<A, R, _>(
                get_meta_name_mlua(name),
                true,
            ))
    }
}
