use std::{borrow::Cow, string::FromUtf8Error};

use rlua::{Context, FromLuaMulti, MetaMethod, Result, ToLuaMulti, UserData};

use crate::{
    rlu::{TealData, TealDataMethods},
    Direction, TealMultiValue, TealType, TypeBody, TypeName,
};

///Contains the data needed to write down the type of a function
pub struct ExportedFunction {
    name: Vec<u8>,
    params: Vec<TealType>,
    returns: Vec<TealType>,
}
impl ExportedFunction {
    ///Creates an ExportedFunction with the given name, Parameters and return value
    ///```no_run
    ///# use tealr::ExportedFunction;
    ///# use std::borrow::Cow;
    ///ExportedFunction::new::<(String,String),String>(Cow::from("concat"));
    ///```
    pub fn new<
        'lua,
        Params: ToLuaMulti<'lua> + TealMultiValue,
        Response: FromLuaMulti<'lua> + TealMultiValue,
    >(
        name: Cow<'static, str>,
    ) -> Self {
        Self {
            name: name.as_bytes().to_owned(),
            params: Params::get_types(Direction::FromLua),
            returns: Response::get_types(Direction::ToLua),
        }
    }
    fn generate(
        self,
        self_type: Option<Cow<'static, str>>,
    ) -> std::result::Result<String, FromUtf8Error> {
        let params = self_type
            .iter()
            .map(|v| v.to_owned())
            .chain(self.params.iter().map(|v| v.name.to_owned()))
            .collect::<Vec<_>>()
            .join(", ");

        let returns = self
            .returns
            .iter()
            .map(|v| v.name.to_owned())
            .collect::<Vec<_>>()
            .join(", ");

        Ok(format!(
            "{}: function({}):({})",
            String::from_utf8(self.name)?,
            params,
            returns
        ))
    }
}

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
}

impl TypeGenerator {
    fn new<A: TypeName>(dir: Direction) -> Self {
        Self {
            is_user_data: false,
            type_name: A::get_type_name(dir),
            fields: Default::default(),
            methods: Default::default(),
            mut_methods: Default::default(),
            functions: Default::default(),
            mut_functions: Default::default(),
        }
    }
    fn get_method_data<
        'lua,
        A: TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        S: ?Sized + AsRef<[u8]>,
    >(
        name: &S,
    ) -> ExportedFunction {
        ExportedFunction {
            name: name.as_ref().to_vec(),
            params: A::get_types(Direction::FromLua),
            returns: R::get_types(Direction::ToLua),
        }
    }
    fn generate(self) -> std::result::Result<String, FromUtf8Error> {
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

        let fields = Self::combine_function_names(fields, "fields");
        let methods = Self::combine_function_names(methods, "pure Methods");
        let methods_mut = Self::combine_function_names(methods_mut, "Mutating Methods");
        let functions = Self::combine_function_names(functions, "Pure functions");
        let functions_mut = Self::combine_function_names(functions_mut, "Mutating Functions");

        let userdata_string = if self.is_user_data { "userdata" } else { "" };
        Ok(format!(
            "\trecord {}\n\t\t{}\n{}{}{}{}{}\n\tend",
            self.type_name, userdata_string, fields, methods, methods_mut, functions, functions_mut
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

impl<'lua, T> TealDataMethods<'lua, T> for TypeGenerator
where
    T: 'static + TealData + UserData,
{
    fn add_method<S, A, R, M>(&mut self, name: &S, _: M)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        M: 'static + Send + Fn(Context<'lua>, &T, A) -> Result<R>,
    {
        self.methods.push(Self::get_method_data::<A, R, _>(name))
    }

    fn add_method_mut<S, A, R, M>(&mut self, name: &S, _: M)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        M: 'static + Send + FnMut(Context<'lua>, &mut T, A) -> Result<R>,
    {
        self.mut_methods
            .push(Self::get_method_data::<A, R, _>(name))
    }

    fn add_function<S, A, R, F>(&mut self, name: &S, _: F)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        F: 'static + Send + Fn(Context<'lua>, A) -> Result<R>,
    {
        self.functions.push(Self::get_method_data::<A, R, _>(name))
    }

    fn add_function_mut<S, A, R, F>(&mut self, name: &S, _: F)
    where
        S: ?Sized + AsRef<[u8]>,
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        F: 'static + Send + FnMut(Context<'lua>, A) -> Result<R>,
    {
        self.mut_functions
            .push(Self::get_method_data::<A, R, _>(name))
    }

    fn add_meta_method<A, R, M>(&mut self, _: MetaMethod, _: M)
    where
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        M: 'static + Send + Fn(Context<'lua>, &T, A) -> Result<R>,
    {
    }

    fn add_meta_method_mut<A, R, M>(&mut self, _: MetaMethod, _: M)
    where
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        M: 'static + Send + FnMut(Context<'lua>, &mut T, A) -> Result<R>,
    {
    }

    fn add_meta_function<A, R, F>(&mut self, _: MetaMethod, _: F)
    where
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        F: 'static + Send + Fn(Context<'lua>, A) -> Result<R>,
    {
    }

    fn add_meta_function_mut<A, R, F>(&mut self, _: MetaMethod, _: F)
    where
        A: FromLuaMulti<'lua> + TealMultiValue,
        R: ToLuaMulti<'lua> + TealMultiValue,
        F: 'static + Send + FnMut(Context<'lua>, A) -> Result<R>,
    {
    }
}

///This generates the .d.tl files
#[derive(Default)]
pub struct TypeWalker {
    given_types: Vec<TypeGenerator>,
}
impl TypeWalker {
    ///creates the TypeWalker
    pub fn new() -> Self {
        Default::default()
    }
    ///prepares a type to have a `.d.tl` file generated, and adds it to the list of types to generate.
    pub fn process_type<A: 'static + TypeName + TypeBody>(mut self, dir: Direction) -> Self {
        let mut new_type = TypeGenerator::new::<A>(dir);
        <A as TypeBody>::get_type_body(dir, &mut new_type);
        //<A as TealData>::add_methods(&mut new_type);
        self.given_types.push(new_type);
        self
    }
    ///generates the `.d.tl` file. It outputs the string, its up to you to store it.
    ///```
    ///# use rlua::{Lua, Result, UserDataMethods};
    ///# use tealr::{rlu::{TealData, TealDataMethods,UserDataWrapper}, TypeWalker, UserData,TypeName};
    ///#[derive(UserData,TypeName)]
    ///struct Example {}
    ///impl TealData for Example {}
    ///let generated_string = TypeWalker::new().process_type::<Example>(tealr::Direction::ToLua).generate("Examples",true);
    ///assert_eq!(generated_string,Ok(String::from("global record Examples
    ///\trecord Example
    ///\t\tuserdata
    ///
    ///\tend
    ///end
    ///return Examples"
    ///)));
    ///```
    pub fn generate(
        self,
        outer_name: &str,
        is_global: bool,
    ) -> std::result::Result<String, FromUtf8Error> {
        let v: Vec<_> = self
            .given_types
            .into_iter()
            .map(|v| v.generate())
            .collect::<std::result::Result<_, _>>()?;
        let v = v.join("\n");
        let scope = if is_global { "global" } else { "local" };
        Ok(format!(
            "{} record {name}\n{record}\nend\nreturn {name}",
            scope,
            name = outer_name,
            record = v
        ))
    }
    ///Same as calling [Typewalker::generate(outer_name,true)](crate::TypeWalker::generate).
    ///```
    ///# use rlua::{Lua, Result, UserDataMethods};
    ///# use tealr::{rlu::{TealData, TealDataMethods,UserDataWrapper}, TypeWalker, UserData,TypeName};
    ///#[derive(UserData,TypeName)]
    ///struct Example {}
    ///impl TealData for Example {}
    ///let generated_string = TypeWalker::new().process_type::<Example>(tealr::Direction::ToLua).generate_global("Examples");
    ///assert_eq!(generated_string,Ok(String::from("global record Examples
    ///\trecord Example
    ///\t\tuserdata
    ///
    ///\tend
    ///end
    ///return Examples"
    ///)));
    ///```
    pub fn generate_global(self, outer_name: &str) -> std::result::Result<String, FromUtf8Error> {
        self.generate(outer_name, true)
    }
    ///Same as calling [Typewalker::generate(outer_name,false)](crate::TypeWalker::generate).
    ///```
    ///# use rlua::{Lua, Result, UserDataMethods};
    ///# use tealr::{rlu::{TealData, TealDataMethods,UserDataWrapper}, TypeWalker, UserData,TypeName};
    ///#[derive(UserData,TypeName)]
    ///struct Example {}
    ///impl TealData for Example {}
    ///let generated_string = TypeWalker::new().process_type::<Example>(tealr::Direction::ToLua).generate_local("Examples");
    ///assert_eq!(generated_string,Ok(String::from("local record Examples
    ///\trecord Example
    ///\t\tuserdata
    ///
    ///\tend
    ///end
    ///return Examples"
    ///)));
    ///```
    pub fn generate_local(self, outer_name: &str) -> std::result::Result<String, FromUtf8Error> {
        self.generate(outer_name, false)
    }
}
