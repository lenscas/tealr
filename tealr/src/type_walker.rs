use std::string::FromUtf8Error;

use rlua::{Context, FromLuaMulti, MetaMethod, Result, ToLuaMulti, UserData};

use crate::{TealData, TealDataMethods, TealMultiValue, TealType};

struct ExportedFunctions {
    name: Vec<u8>,
    params: Vec<TealType>,
    returns: Vec<TealType>,
}
impl ExportedFunctions {
    fn generate(self, self_type: Option<String>) -> std::result::Result<String, FromUtf8Error> {
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

struct TypeGenerator {
    type_name: String,
    methods: Vec<ExportedFunctions>,
    mut_methods: Vec<ExportedFunctions>,
    functions: Vec<ExportedFunctions>,
    mut_functions: Vec<ExportedFunctions>,
}

impl TypeGenerator {
    fn new<A: TealData>() -> Self {
        Self {
            type_name: A::get_type_name(),
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
    ) -> ExportedFunctions {
        ExportedFunctions {
            name: name.as_ref().to_vec(),
            params: A::get_types(),
            returns: R::get_types(),
        }
    }
    fn generate(self) -> std::result::Result<String, FromUtf8Error> {
        //let head = format!("local record {}", self.type_name);
        let type_name = self.type_name.clone();
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

        let methods = Self::combine_function_names(methods, "pure Methods");
        let methods_mut = Self::combine_function_names(methods_mut, "Mutating Methods");
        let functions = Self::combine_function_names(functions, "Pure functions");
        let functions_mut = Self::combine_function_names(functions_mut, "Mutating Functions");
        Ok(format!(
            "\trecord {}\n{}{}{}{}\n\tend",
            self.type_name, methods, methods_mut, functions, functions_mut
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
    pub fn new() -> Self {
        Default::default()
    }
    ///prepares a type to have a `.d.tl` file generated, and adds it to the list of types to generate.
    pub fn proccess_type<A: 'static + TealData + UserData>(mut self) -> Self {
        let mut new_type = TypeGenerator::new::<A>();
        <A as TealData>::add_methods(&mut new_type);
        self.given_types.push(new_type);
        self
    }
    ///generates the `.d.tl` file. It outputs the string, its up to you to store it.
    ///```
    ///# use rlua::{Lua, Result, UserDataMethods};
    ///# use tealr::{TealData, TealDataMethods, TypeWalker, UserDataWrapper,UserData};
    ///#[derive(UserData)]
    ///struct Example {}
    ///impl TealData for Example {
    ///    fn get_type_name()-> String {
    ///        String::from("Example")
    ///    }
    ///}
    ///let generated_string = TypeWalker::new().proccess_type::<Example>().generate("Examples");
    ///assert_eq!(generated_string,Ok(String::from("local record Examples
    ///\trecord Example
    ///
    ///\tend
    ///end
    ///return Examples"
    ///)));
    ///```
    pub fn generate(self, outer_name: &str) -> std::result::Result<String, FromUtf8Error> {
        let v: Vec<_> = self
            .given_types
            .into_iter()
            .map(|v| v.generate())
            .collect::<std::result::Result<_, _>>()?;
        let v = v.join("\n");
        Ok(format!(
            "local record {name}\n{record}\nend\nreturn {name}",
            name = outer_name,
            record = v
        ))
    }
}
