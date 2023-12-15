use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    ops::Deref,
    string::FromUtf8Error,
};

#[cfg(feature = "rlua")]
use crate::rlu::{
    get_meta_name as get_meta_name_rlua, TealData as TealDataR, TealDataMethods as TealDataMethodsR,
};
#[cfg(feature = "rlua")]
use rlua::{
    Context, FromLua as FromLuaR, FromLuaMulti as FromLuaMultiR, MetaMethod as MetaMethodR,
    Result as ResultR, ToLuaMulti as ToLuaMultiR, UserData as UserDataR,
};
use serde::{Deserialize, Serialize};

#[cfg(feature = "mlua")]
use crate::mlu::{
    get_meta_name as get_meta_name_mlua, MaybeSend, TealData as TealDataM, TealDataFields,
    TealDataMethods as TealDataMethodsM,
};
#[cfg(feature = "mlua")]
use mlua::{
    FromLua as FromLuaM, FromLuaMulti as FromLuaMultiM, IntoLua as ToLuaM,
    IntoLuaMulti as ToLuaMultiM, Lua, MetaMethod as MetaMethodM, Result as ResultM,
    UserData as UserDataM,
};

use crate::{
    exported_function::ExportedFunction, type_parts_to_str, NamePart, ToTypename, Type, TypeName,
};

#[cfg(any(feature = "rlua", feature = "mlua"))]
use crate::TealMultiValue;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
///Simple wrapper around `Vec<u8>`
pub struct NameContainer(pub(crate) Vec<u8>);

impl Deref for NameContainer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[cfg(feature = "rlua")]
impl<'lua> FromLuaR<'lua> for NameContainer {
    fn from_lua(lua_value: rlua::Value<'lua>, lua: Context<'lua>) -> ResultR<Self> {
        Ok(<String as FromLuaR>::from_lua(lua_value, lua)?
            .into_bytes()
            .into())
    }
}
#[cfg(feature = "rlua")]
impl<'lua> crate::rlu::rlua::ToLua<'lua> for NameContainer {
    fn to_lua(self, lua: Context<'lua>) -> ResultR<rlua::Value<'lua>> {
        lua.create_string(&self.0).and_then(|x| lua.pack(x))
    }
}

impl ToTypename for NameContainer {
    fn to_typename() -> crate::Type {
        Type::new_single("string", crate::KindOfType::Builtin)
    }
}

#[cfg(feature = "mlua")]
impl<'lua> FromLuaM<'lua> for NameContainer {
    fn from_lua(lua_value: mlua::Value<'lua>, lua: &'lua Lua) -> ResultM<Self> {
        Ok(<String as FromLuaM>::from_lua(lua_value, lua)?
            .into_bytes()
            .into())
    }
}
#[cfg(feature = "mlua")]
impl<'lua> ToLuaM<'lua> for NameContainer {
    fn into_lua(self, lua: &'lua Lua) -> ResultM<mlua::Value<'lua>> {
        lua.create_string(&self.0).and_then(|x| x.into_lua(lua))
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
pub(crate) fn get_method_data<A: TealMultiValue, R: TealMultiValue, S: ?Sized + AsRef<str>>(
    name: &S,
    is_meta_method: bool,
    extra_self: Option<Type>,
) -> ExportedFunction {
    ExportedFunction::new::<A, R, _>(name, is_meta_method, extra_self)
}
///Container of all the information needed to create the `.d.tl` file for your type.
#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[cfg_attr(
    all(feature = "mlua", feature = "derive", not(feature = "rlua")),
    derive(crate::mlu::FromToLua, crate::ToTypename)
)]
#[cfg_attr(
    all(feature = "rlua", feature = "derive", not(feature = "mlua")),
    derive(crate::rlu::FromToLua, crate::ToTypename)
)]
#[cfg_attr(
    all(any(feature = "rlua", feature = "mlua"), feature = "derive", not(all(feature = "mlua", feature="rlua"))),
    tealr(tealr_name = crate)
)]
pub enum TypeGenerator {
    ///the type should be represented as a struct
    Record(
        #[cfg_attr(
        all(any(feature = "rlua", feature = "mlua"), feature = "derive",not(all(feature = "rlua", feature = "mlua"))),
        tealr(remote =  RecordGenerator))]
        Box<RecordGenerator>,
    ),
    ///the type should be represented as an enum
    Enum(EnumGenerator),
}

impl TypeGenerator {
    pub(crate) fn generate(self) -> Result<String, FromUtf8Error> {
        match self {
            TypeGenerator::Record(x) => x.generate(),
            TypeGenerator::Enum(x) => Ok(x.generate()),
        }
    }
}

type V = Vec<NamePart>;
///contains all the information needed to create a teal enum.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[cfg_attr(
    all(feature = "mlua", feature = "derive", not(feature = "rlua")),
    derive(crate::mlu::FromToLua, crate::ToTypename)
)]
#[cfg_attr(
    all(feature = "rlua", feature = "derive", not(feature = "mlua")),
    derive(crate::rlu::FromToLua, crate::ToTypename)
)]
#[cfg_attr(
    all(any(feature = "rlua", feature = "mlua"), feature = "derive", not(all(feature = "rlua", feature = "mlua"))),
    tealr(tealr_name = crate)
)]

pub struct EnumGenerator {
    ///the name of this enum
    #[cfg_attr(
    all(any(feature = "rlua", feature = "mlua"), feature = "derive", not(all(feature = "rlua", feature = "mlua"))),
    tealr(remote = V)
)]
    pub name: Cow<'static, [NamePart]>,
    ///the variants that make up this enum.
    pub variants: Vec<NameContainer>,
    ///documentation for this enum
    pub type_doc: String,
}
impl From<EnumGenerator> for TypeGenerator {
    fn from(a: EnumGenerator) -> Self {
        TypeGenerator::Enum(a)
    }
}
impl EnumGenerator {
    ///creates a new EnumGenerator
    pub fn new<A: ToTypename>() -> Self {
        Self {
            name: A::get_type_parts(),
            variants: Default::default(),
            type_doc: Default::default(),
        }
    }
    ///Add type level documentation to this enum
    pub fn document_type(&mut self, documentation: &str) -> &mut Self {
        self.type_doc.push_str(documentation);
        self.type_doc.push('\n');
        self.type_doc.push('\n');
        self
    }
    pub(crate) fn generate(self) -> String {
        let variants = self
            .variants
            .into_iter()
            .map(|v| String::from_utf8_lossy(&v).to_string())
            .map(|v| v.replace('\\', "\\\\").replace('"', "\\\""))
            .map(|v| format!("\t\t\"{v}\""))
            .collect::<Vec<_>>()
            .join("\n");
        let name = type_parts_to_str(self.name);
        format!("\tenum {name}\n{variants}\n\tend")
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[cfg_attr(
    all(feature = "mlua", feature = "derive", not(feature = "rlua")),
    derive(crate::mlu::FromToLua, crate::ToTypename)
)]
#[cfg_attr(
    all(feature = "rlua", feature = "derive", not(feature = "mlua")),
    derive(crate::rlu::FromToLua, crate::ToTypename)
)]
#[cfg_attr(
    all(any(feature = "rlua", feature = "mlua"), feature = "derive", not(all(feature = "rlua", feature = "mlua"))),
    tealr(tealr_name = crate)
)]

///Represents a field, containing both the name and its type
pub struct Field {
    ///the name of the field
    pub name: NameContainer,

    ///the type of the field, according to the old format
    #[cfg_attr(
    all(any(feature = "rlua", feature = "mlua"), feature = "derive", not(all(feature = "rlua", feature = "mlua"))),
    tealr(remote = V)
)]
    pub teal_type: Cow<'static, [NamePart]>,
    /// the type of the field
    pub ty: Type,
}

impl From<(NameContainer, Type)> for Field {
    fn from((name, ty): (NameContainer, Type)) -> Self {
        #[allow(deprecated)]
        let teal_type = crate::new_type_to_old(ty.clone(), false);
        Self {
            name,
            teal_type,
            ty,
        }
    }
}
impl Field {
    ///creates a new field
    pub fn new<A: ToTypename>(name: impl Into<Cow<'static, str>>) -> Self {
        (NameContainer::from(name.into()), A::to_typename()).into()
    }
}

impl From<Field> for (NameContainer, Cow<'static, [NamePart]>) {
    fn from(x: Field) -> Self {
        (x.name, x.teal_type)
    }
}

///contains all the information needed to create a record
#[derive(Default, Clone, serde::Serialize, serde::Deserialize)]
#[cfg_attr(
    all(feature = "mlua", feature = "derive", not(feature = "rlua")),
    derive(crate::mlu::FromToLua, crate::ToTypename)
)]
#[cfg_attr(
    all(feature = "rlua", feature = "derive", not(feature = "mlua")),
    derive(crate::rlu::FromToLua, crate::ToTypename)
)]
#[cfg_attr(
    all(any(feature = "rlua", feature = "mlua"), feature = "derive", not(all(feature = "rlua", feature = "mlua"))),
    tealr(tealr_name = crate)
)]

pub struct RecordGenerator {
    ///Represents if the type should be inlined or not.
    pub should_be_inlined: bool,
    ///Represents if the type is UserData
    pub is_user_data: bool,
    ///The name of the type in teal
    #[cfg_attr(
        all(any(feature = "rlua", feature = "mlua"), feature = "derive", not(all(feature = "rlua", feature = "mlua"))),
        tealr(remote = V)
    )]
    pub type_name: Cow<'static, [NamePart]>,
    ///The exposed fields and their types
    pub fields: Vec<Field>,
    ///The exposed static fields and their types
    pub static_fields: Vec<Field>,
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
    pub(crate) next_docs: Option<String>,
    ///if this type needs to get a `.help()` function
    pub should_generate_help_method: bool,
}

impl From<RecordGenerator> for TypeGenerator {
    fn from(a: RecordGenerator) -> Self {
        TypeGenerator::Record(Box::new(a))
    }
}

impl From<Box<RecordGenerator>> for RecordGenerator {
    fn from(x: Box<RecordGenerator>) -> Self {
        *x
    }
}

impl RecordGenerator {
    ///creates a new RecordGenerator
    pub fn new<A: ToTypename>(should_be_inlined: bool) -> Self {
        Self {
            should_be_inlined,
            is_user_data: false,
            type_name: A::get_type_parts(),
            should_generate_help_method: true,
            ..Default::default()
        }
    }

    pub(crate) fn generate(self) -> std::result::Result<String, FromUtf8Error> {
        //let head = format!("local record {}", self.type_name);
        let type_name = type_parts_to_str(self.type_name);
        let mut duplicates = HashSet::new();
        let documentation = &self.documentation;
        let fields: Vec<_> = self
            .fields
            .into_iter()
            .chain(self.static_fields)
            .filter(|field| duplicates.insert(field.name.0.clone()))
            .map(|field| {
                let name = field.name;
                let lua_type = field.teal_type;
                let doc = match documentation.get(&name) {
                    Some(x) => x
                        .lines()
                        .map(|v| {
                            let mut str = "--".to_string();
                            str.push_str(v);
                            str.push('\n');
                            str
                        })
                        .collect::<String>(),
                    None => String::from(""),
                };
                (name, lua_type, doc)
            })
            .map(|(name, lua_type, doc)| {
                format!(
                    "{doc}{} : {}",
                    String::from_utf8_lossy(&name),
                    crate::type_parts_to_str(lua_type)
                )
            })
            .collect();

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
                        .map(|v| {
                            let mut str = "\t".to_string();
                            str.push_str(v);
                            str.push('\n');
                            str
                        })
                        .collect::<String>()
                ),
                "\tend",
            )
        };
        let type_header = type_header
            .lines()
            .map(|v| {
                let mut str = "\t".to_string();
                str.push_str(v);
                str.push('\n');
                str
            })
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

impl RecordGenerator {
    /// copies the documentation stored in "next_docs" to be linked to the given name
    /// Shouldn't be called manually unless you add the fields/methods by hand rather than using the functions for this.
    pub fn copy_docs(&mut self, to: &[u8]) {
        if let Some(docs) = self.next_docs.take() {
            match self.documentation.entry(to.to_owned().into()) {
                std::collections::hash_map::Entry::Vacant(x) => {
                    x.insert(docs);
                }
                std::collections::hash_map::Entry::Occupied(mut x) => {
                    let current_docs = x.get_mut();
                    current_docs.push('\n');
                    current_docs.push('\n');
                    current_docs.push_str(&docs);
                }
            }
        }
    }
    ///adds documentation to the next field.
    pub fn document(&mut self, documentation: &str) {
        match &mut self.next_docs {
            Some(x) => {
                x.push('\n');
                x.push('\n');
                x.push_str(documentation)
            }
            None => self.next_docs = Some(documentation.to_owned()),
        };
    }
    ///adds documentation to the type itself
    pub fn document_type(&mut self, documentation: &str) -> &mut Self {
        self.type_doc.push_str(documentation);
        self.type_doc.push('\n');
        self.type_doc.push('\n');
        self
    }
}

#[cfg(feature = "rlua")]
impl<'lua, T> TealDataMethodsR<'lua, T> for RecordGenerator
where
    T: 'static + TealDataR + UserDataR + ToTypename,
{
    fn add_method<S, A, R, M>(&mut self, name: &S, _: M)
    where
        S: ?Sized + AsRef<str>,
        A: FromLuaMultiR<'lua> + TealMultiValue,
        R: ToLuaMultiR<'lua> + TealMultiValue,
        M: 'static + Send + Fn(Context<'lua>, &T, A) -> ResultR<R>,
    {
        self.copy_docs(name.as_ref().as_bytes());
        self.methods.push(get_method_data::<A, R, _>(
            name,
            false,
            Some(T::to_typename()),
        ))
    }

    fn add_method_mut<S, A, R, M>(&mut self, name: &S, _: M)
    where
        S: ?Sized + AsRef<str>,
        A: FromLuaMultiR<'lua> + TealMultiValue,
        R: ToLuaMultiR<'lua> + TealMultiValue,
        M: 'static + Send + FnMut(Context<'lua>, &mut T, A) -> ResultR<R>,
    {
        self.copy_docs(name.as_ref().as_bytes());
        self.mut_methods.push(get_method_data::<A, R, _>(
            name,
            false,
            Some(T::to_typename()),
        ))
    }

    fn add_function<S, A, R, F>(&mut self, name: &S, _: F)
    where
        S: ?Sized + AsRef<str>,
        A: FromLuaMultiR<'lua> + TealMultiValue,
        R: ToLuaMultiR<'lua> + TealMultiValue,
        F: 'static + Send + Fn(Context<'lua>, A) -> ResultR<R>,
    {
        self.copy_docs(name.as_ref().as_bytes());
        self.functions
            .push(get_method_data::<A, R, _>(name, false, None))
    }

    fn add_function_mut<S, A, R, F>(&mut self, name: &S, _: F)
    where
        S: ?Sized + AsRef<str>,
        A: FromLuaMultiR<'lua> + TealMultiValue,
        R: ToLuaMultiR<'lua> + TealMultiValue,
        F: 'static + Send + FnMut(Context<'lua>, A) -> ResultR<R>,
    {
        self.copy_docs(name.as_ref().as_bytes());
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
            Some(T::to_typename()),
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
            Some(T::to_typename()),
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
    fn document(&mut self, documentation: &str) -> &mut Self {
        self.document(documentation);
        self
    }
    fn generate_help(&mut self) {
        self.functions
            .push(get_method_data::<Option<String>, String, _>(
                "help", false, None,
            ))
    }

    fn document_type(&mut self, documentation: &str) -> &mut Self {
        self.document_type(documentation)
    }
}

#[cfg(feature = "mlua")]
impl<'lua, T> TealDataMethodsM<'lua, T> for RecordGenerator
where
    T: 'static + TealDataM + UserDataM + ToTypename,
{
    fn add_method<S, A, R, M>(&mut self, name: &S, _: M)
    where
        S: ?Sized + AsRef<str>,
        A: FromLuaMultiM<'lua> + TealMultiValue,
        R: ToLuaMultiM<'lua> + TealMultiValue,
        M: 'static + MaybeSend + Fn(&'lua Lua, &T, A) -> ResultM<R>,
    {
        self.copy_docs(name.as_ref().as_bytes());
        self.methods.push(get_method_data::<A, R, _>(
            name,
            false,
            Some(T::to_typename()),
        ))
    }

    fn add_method_mut<S, A, R, M>(&mut self, name: &S, _: M)
    where
        S: ?Sized + AsRef<str>,
        A: FromLuaMultiM<'lua> + TealMultiValue,
        R: ToLuaMultiM<'lua> + TealMultiValue,
        M: 'static + MaybeSend + FnMut(&'lua Lua, &mut T, A) -> ResultM<R>,
    {
        self.copy_docs(name.as_ref().as_bytes());
        self.mut_methods.push(get_method_data::<A, R, _>(
            name,
            false,
            Some(T::to_typename()),
        ))
    }

    fn add_function<S, A, R, F>(&mut self, name: &S, _: F)
    where
        S: ?Sized + AsRef<str>,
        A: FromLuaMultiM<'lua> + TealMultiValue,
        R: ToLuaMultiM<'lua> + TealMultiValue,
        F: 'static + MaybeSend + Fn(&'lua Lua, A) -> ResultM<R>,
    {
        self.copy_docs(name.as_ref().as_bytes());
        self.functions
            .push(get_method_data::<A, R, _>(name, false, None))
    }

    fn add_function_mut<S, A, R, F>(&mut self, name: &S, _: F)
    where
        S: ?Sized + AsRef<str>,
        A: FromLuaMultiM<'lua> + TealMultiValue,
        R: ToLuaMultiM<'lua> + TealMultiValue,
        F: 'static + MaybeSend + FnMut(&'lua Lua, A) -> ResultM<R>,
    {
        self.copy_docs(name.as_ref().as_bytes());
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
            &get_meta_name_mlua(name),
            true,
            Some(T::to_typename()),
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
            &get_meta_name_mlua(name),
            true,
            Some(T::to_typename()),
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
            get_meta_name_mlua(name).as_ref(),
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
            &get_meta_name_mlua(name),
            true,
            None,
        ))
    }
    #[cfg(feature = "mlua_async")]
    fn add_async_method<'s, S: ?Sized + AsRef<str>, A, R, M, MR>(&mut self, name: &S, _: M)
    where
        'lua: 's,
        T: 'static,
        M: Fn(&'lua Lua, &'s T, A) -> MR + MaybeSend + 'static,
        A: FromLuaMultiM<'lua> + TealMultiValue,
        MR: std::future::Future<Output = ResultM<R>> + 's,
        R: ToLuaMultiM<'lua> + TealMultiValue,
    {
        self.copy_docs(name.as_ref().as_bytes());
        self.methods.push(get_method_data::<A, R, _>(
            name,
            false,
            Some(T::to_typename()),
        ))
    }

    #[cfg(feature = "mlua_async")]
    fn add_async_function<S: ?Sized, A, R, F, FR>(&mut self, name: &S, _: F)
    where
        S: AsRef<str>,
        A: FromLuaMultiM<'lua> + TealMultiValue,
        R: ToLuaMultiM<'lua> + TealMultiValue,
        F: 'static + MaybeSend + Fn(&'lua Lua, A) -> FR,
        FR: 'lua + std::future::Future<Output = ResultM<R>>,
    {
        self.copy_docs(name.as_ref().as_bytes());
        self.functions
            .push(get_method_data::<A, R, _>(name, false, None))
    }

    fn generate_help(&mut self) {
        self.functions
            .push(get_method_data::<Option<String>, String, _>(
                "help", false, None,
            ))
    }
    fn document(&mut self, documentation: &str) -> &mut Self {
        self.document(documentation);
        self
    }
    fn document_type(&mut self, documentation: &str) -> &mut Self {
        self.document_type(documentation)
    }
}
#[cfg(feature = "mlua")]
impl<'lua, T> TealDataFields<'lua, T> for RecordGenerator
where
    T: 'static + TealDataM + UserDataM + ToTypename,
{
    fn add_field_method_get<S, R, M>(&mut self, name: &S, _: M)
    where
        S: AsRef<str> + ?Sized,
        R: mlua::IntoLua<'lua> + ToTypename,
        M: 'static + MaybeSend + Fn(&'lua Lua, &T) -> mlua::Result<R>,
    {
        self.copy_docs(name.as_ref().as_bytes());
        self.fields
            .push((name.as_ref().as_bytes().to_vec().into(), R::to_typename()).into());
    }

    fn add_field_method_set<S, A, M>(&mut self, name: &S, _: M)
    where
        S: AsRef<str> + ?Sized,
        A: mlua::FromLua<'lua> + ToTypename,
        M: 'static + MaybeSend + FnMut(&'lua Lua, &mut T, A) -> mlua::Result<()>,
    {
        self.copy_docs(name.as_ref().as_bytes());
        self.fields
            .push((name.as_ref().as_bytes().to_vec().into(), A::to_typename()).into());
    }

    fn add_field_function_get<S, R, F>(&mut self, name: &S, _: F)
    where
        S: AsRef<str> + ?Sized,
        R: mlua::IntoLua<'lua> + ToTypename,
        F: 'static + MaybeSend + Fn(&'lua Lua, mlua::AnyUserData<'lua>) -> mlua::Result<R>,
    {
        self.copy_docs(name.as_ref().as_bytes());
        self.static_fields
            .push((name.as_ref().as_bytes().to_vec().into(), R::to_typename()).into());
    }

    fn add_field_function_set<S, A, F>(&mut self, name: &S, _: F)
    where
        S: AsRef<str> + ?Sized,
        A: mlua::FromLua<'lua> + ToTypename,
        F: 'static + MaybeSend + FnMut(&'lua Lua, mlua::AnyUserData<'lua>, A) -> mlua::Result<()>,
    {
        self.copy_docs(name.as_ref().as_bytes());
        self.static_fields
            .push((name.as_ref().as_bytes().to_vec().into(), A::to_typename()).into());
    }

    fn add_meta_field_with<R, F>(&mut self, meta: MetaMethodM, _: F)
    where
        F: 'static + MaybeSend + Fn(&'lua Lua) -> mlua::Result<R>,
        R: mlua::IntoLua<'lua> + ToTypename,
    {
        let x = Into::<MetaMethodM>::into(meta);
        let name: Cow<'_, str> = Cow::Owned(x.name().to_string());
        self.copy_docs(name.as_bytes());
        self.static_fields
            .push((NameContainer::from(name), R::to_typename()).into());
    }

    fn document(&mut self, documentation: &str) {
        self.document(documentation)
    }
}
