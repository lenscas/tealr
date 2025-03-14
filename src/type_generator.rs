use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    ops::Deref,
    string::FromUtf8Error,
};

#[cfg(feature = "mlua")]
use crate::mlu::{
    get_meta_name as get_meta_name_mlua,
    mlua::{
        FromLua as FromLuaM, FromLuaMulti as FromLuaMultiM, IntoLua as ToLuaM,
        IntoLuaMulti as ToLuaMultiM, Lua, MetaMethod as MetaMethodM, Result as ResultM,
        UserData as UserDataM,
    },
    MaybeSend, TealData as TealDataM, TealDataFields, TealDataMethods as TealDataMethodsM,
};
use serde::{Deserialize, Serialize};

use crate::{
    exported_function::ExportedFunction, type_parts_to_str, NamePart, ToTypename, Type, TypeName,
};

use crate::TealMultiValue;

#[derive(Clone, Hash, PartialEq, Eq)]
///Simple wrapper around `Vec<u8>`
pub struct NameContainer(pub(crate) Vec<u8>);

impl Debug for NameContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <_ as Debug>::fmt(&String::from_utf8_lossy(&self.0), f)
    }
}

impl Deref for NameContainer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for NameContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.0))
    }
}

impl ToTypename for NameContainer {
    fn to_typename() -> Type {
        Type::new_single("string", crate::KindOfType::Builtin)
    }
}

#[cfg(feature = "mlua")]
impl FromLuaM for NameContainer {
    fn from_lua(lua_value: mlua::Value, lua: &Lua) -> ResultM<Self> {
        Ok(<String as FromLuaM>::from_lua(lua_value, lua)?
            .into_bytes()
            .into())
    }
}
#[cfg(feature = "mlua")]
impl ToLuaM for NameContainer {
    fn into_lua(self, lua: &Lua) -> ResultM<mlua::Value> {
        lua.create_string(self.0).and_then(|x| x.into_lua(lua))
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
impl PartialEq<&str> for NameContainer {
    fn eq(&self, other: &&str) -> bool {
        self.0 == other.as_bytes()
    }
}

#[allow(dead_code)]
pub(crate) fn get_method_data<A: TealMultiValue, R: TealMultiValue, S: ToString + AsRef<str>>(
    name: S,
    is_meta_method: bool,
    extra_self: Option<Type>,
) -> ExportedFunction {
    ExportedFunction::new::<A, R, _>(name, is_meta_method, extra_self)
}
///Container of all the information needed to create the `.d.tl` file for your type.
#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[cfg_attr(
    all(feature = "mlua", feature = "derive"),
    derive(crate::mlu::FromToLua, crate::ToTypename)
)]
#[cfg_attr(
    all(feature = "mlua", feature = "derive"),
    tealr(tealr_name = crate)
)]
#[cfg_attr(
    all(feature = "mlua", feature = "derive"),
    tealr(extend_methods = add_lua_funcs_to_type_gen)
)]

pub enum TypeGenerator {
    ///the type should be represented as a struct
    Record(
        #[cfg_attr(
        all(feature = "mlua", feature = "derive"),
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
    ///returns the name of the current type
    pub fn type_name(&self) -> &Type {
        match self {
            TypeGenerator::Record(record_generator) => &record_generator.ty,
            TypeGenerator::Enum(enum_generator) => &enum_generator.ty,
        }
    }
    /// returns true if `self` is an TypeGenerator::Record(x) and x.should_be_inlined is true
    pub fn is_inlined(&self) -> bool {
        match self {
            TypeGenerator::Record(record_generator) => record_generator.should_be_inlined,
            TypeGenerator::Enum(_) => false,
        }
    }
    ///returns the RecordGenerator if self is `TypeGenerator::Record(x)` otherwise returns None
    pub fn record(&self) -> Option<&RecordGenerator> {
        match self {
            TypeGenerator::Record(record_generator) => Some(record_generator),
            TypeGenerator::Enum(_) => None,
        }
    }
}

#[cfg(feature = "mlua")]
fn add_lua_funcs_to_type_gen<A: TealDataMethodsM<TypeGenerator>>(a: &mut A) {
    a.add_method("type_name", |_, x, ()| Ok(x.type_name().to_owned()));
    a.add_method("is_inlined", |_, x, ()| Ok(x.is_inlined()));
}

#[allow(dead_code)]
type V = Vec<NamePart>;
///contains all the information needed to create a teal enum.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[cfg_attr(
    all(feature = "mlua", feature = "derive"),
    derive(crate::mlu::FromToLua, crate::ToTypename)
)]
#[cfg_attr(
    all(feature = "mlua", feature = "derive"),
    tealr(tealr_name = crate)
)]

pub struct EnumGenerator {
    ///the type of this enum
    pub ty: Type,
    ///the name of this enum
    #[cfg_attr(
        all(feature = "mlua", feature = "derive"),
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
            ty: A::to_typename(),
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

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[cfg_attr(
    all(feature = "mlua", feature = "derive"),
    derive(crate::mlu::FromToLua, crate::ToTypename)
)]
#[cfg_attr(
    all(feature = "mlua", feature = "derive"),
    tealr(tealr_name = crate)
)]

///Represents a field, containing both the name and its type
pub struct Field {
    ///the name of the field
    pub name: NameContainer,

    ///the type of the field, according to the old format
    #[cfg_attr(
        all(feature = "mlua", feature = "derive"),
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
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[cfg_attr(
    all(feature = "mlua", feature = "derive"),
    derive(crate::mlu::FromToLua, crate::ToTypename)
)]
#[cfg_attr(
    all(feature = "mlua", feature = "derive"),
    tealr(tealr_name = crate)
)]

pub struct RecordGenerator {
    ///Represents if the type should be inlined or not.
    pub should_be_inlined: bool,
    ///Represents if the type is UserData
    pub is_user_data: bool,
    ///the type produced by typename
    pub ty: Type,
    ///The name of the type in teal.
    ///This uses the old system and thus should ideally not be used.
    #[cfg_attr(
        all(feature = "mlua", feature = "derive"),
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
    #[doc(hidden)]
    pub next_docs: Option<String>,
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
            documentation: Default::default(),
            ty: A::to_typename(),
            fields: Default::default(),
            static_fields: Default::default(),
            methods: Default::default(),
            mut_methods: Default::default(),
            functions: Default::default(),
            mut_functions: Default::default(),
            meta_method: Default::default(),
            meta_method_mut: Default::default(),
            meta_function: Default::default(),
            meta_function_mut: Default::default(),
            type_doc: Default::default(),
            next_docs: Default::default(),
        }
    }
    /// creates an iterator that goes over the various method and function fields
    pub fn all_functions(&self) -> impl Iterator<Item = &ExportedFunction> {
        self.methods
            .iter()
            .chain(self.mut_methods.iter())
            .chain(self.functions.iter())
            .chain(self.mut_functions.iter())
            .chain(self.meta_method.iter())
            .chain(self.meta_method_mut.iter())
            .chain(self.meta_function.iter())
            .chain(self.meta_function_mut.iter())
    }

    pub(crate) fn generate(self) -> Result<String, FromUtf8Error> {
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
                    type_parts_to_str(lua_type)
                )
            })
            .collect();

        let methods: Vec<_> = self
            .methods
            .into_iter()
            .map(|v| v.generate(documentation)) //v.generate(Some(type_name.clone()), documentation))
            .collect::<Result<_, _>>()?;

        let methods_mut: Vec<_> = self
            .mut_methods
            .into_iter()
            .map(|v| v.generate(documentation))
            .collect::<Result<_, _>>()?;

        let functions: Vec<_> = self
            .functions
            .into_iter()
            .map(|f| f.generate(documentation))
            .collect::<Result<_, _>>()?;

        let functions_mut: Vec<_> = self
            .mut_functions
            .into_iter()
            .map(|f| f.generate(documentation))
            .collect::<Result<_, _>>()?;

        let meta_methods: Vec<_> = self
            .meta_method
            .into_iter()
            .map(|f| f.generate(documentation))
            .collect::<Result<_, _>>()?;

        let meta_methods_mut: Vec<_> = self
            .meta_method_mut
            .into_iter()
            .map(|f| f.generate(documentation))
            .collect::<Result<_, _>>()?;

        let meta_function: Vec<_> = self
            .meta_function
            .into_iter()
            .map(|f| f.generate(documentation))
            .collect::<Result<_, _>>()?;
        let meta_function_mut: Vec<_> = self
            .meta_function_mut
            .into_iter()
            .map(|f| f.generate(documentation))
            .collect::<Result<_, _>>()?;

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

#[cfg(feature = "mlua")]
impl<T> TealDataMethodsM<T> for RecordGenerator
where
    T: 'static + TealDataM + UserDataM + ToTypename,
{
    fn add_method<S, A, R, M>(&mut self, name: S, _: M)
    where
        S: ToString + AsRef<str>,
        A: FromLuaMultiM + TealMultiValue,
        R: ToLuaMultiM + TealMultiValue,
        M: 'static + MaybeSend + Fn(&Lua, &T, A) -> ResultM<R>,
    {
        self.add_method::<S, A, R, T>(name);
    }

    fn add_method_mut<S, A, R, M>(&mut self, name: S, _: M)
    where
        S: ToString + AsRef<str>,
        A: FromLuaMultiM + TealMultiValue,
        R: ToLuaMultiM + TealMultiValue,
        M: 'static + MaybeSend + FnMut(&Lua, &mut T, A) -> ResultM<R>,
    {
        self.add_method_mut::<S, A, R, T>(name);
    }

    #[cfg(feature = "mlua_async")]
    fn add_async_method<S: ToString + AsRef<str>, A, R, M, MR>(&mut self, name: S, _: M)
    where
        T: 'static,
        M: Fn(Lua, mlua::UserDataRef<T>, A) -> MR + MaybeSend + 'static,
        A: FromLuaMultiM + TealMultiValue,
        MR: std::future::Future<Output = ResultM<R>> + MaybeSend + 'static,
        R: ToLuaMultiM + TealMultiValue,
    {
        self.copy_docs(name.as_ref().as_bytes());
        self.methods.push(get_method_data::<A, R, _>(
            name,
            false,
            Some(T::to_typename()),
        ))
    }

    fn add_function<S, A, R, F>(&mut self, name: S, _: F)
    where
        S: ToString + AsRef<str>,
        A: FromLuaMultiM + TealMultiValue,
        R: ToLuaMultiM + TealMultiValue,
        F: 'static + MaybeSend + Fn(&Lua, A) -> ResultM<R>,
    {
        self.add_function::<S, A, R>(name);
    }

    fn add_function_mut<S, A, R, F>(&mut self, name: S, _: F)
    where
        S: ToString + AsRef<str>,
        A: FromLuaMultiM + TealMultiValue,
        R: ToLuaMultiM + TealMultiValue,
        F: 'static + MaybeSend + FnMut(&Lua, A) -> ResultM<R>,
    {
        self.add_function_mut::<S, A, R>(name);
    }

    #[cfg(feature = "mlua_async")]
    fn add_async_function<S, A, R, F, FR>(&mut self, name: S, _: F)
    where
        S: AsRef<str> + ToString,
        A: FromLuaMultiM + TealMultiValue,
        R: ToLuaMultiM + TealMultiValue,
        F: Fn(Lua, A) -> FR + MaybeSend + 'static,
        FR: std::future::Future<Output = ResultM<R>>,
    {
        self.copy_docs(name.as_ref().as_bytes());
        self.functions
            .push(get_method_data::<A, R, _>(name, false, None))
    }

    fn add_meta_method<A, R, M>(&mut self, name: MetaMethodM, _: M)
    where
        A: FromLuaMultiM + TealMultiValue,
        R: ToLuaMultiM + TealMultiValue,
        M: 'static + MaybeSend + Fn(&Lua, &T, A) -> ResultM<R>,
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
        A: FromLuaMultiM + TealMultiValue,
        R: ToLuaMultiM + TealMultiValue,
        M: 'static + MaybeSend + FnMut(&Lua, &mut T, A) -> ResultM<R>,
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
        A: FromLuaMultiM + TealMultiValue,
        R: ToLuaMultiM + TealMultiValue,
        F: 'static + MaybeSend + Fn(&Lua, A) -> ResultM<R>,
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
        A: FromLuaMultiM + TealMultiValue,
        R: ToLuaMultiM + TealMultiValue,
        F: 'static + MaybeSend + FnMut(&Lua, A) -> ResultM<R>,
    {
        self.copy_docs(name.name().as_bytes());
        self.meta_function_mut.push(get_method_data::<A, R, _>(
            &get_meta_name_mlua(name),
            true,
            None,
        ))
    }

    fn document(&mut self, documentation: &str) -> &mut Self {
        self.document(documentation);
        self
    }
    fn document_type(&mut self, documentation: &str) -> &mut Self {
        self.document_type(documentation)
    }
    fn generate_help(&mut self) {
        self.functions
            .push(get_method_data::<Option<String>, String, _>(
                "help", false, None,
            ))
    }
}

#[cfg(feature = "mlua")]
impl<T> TealDataFields<T> for RecordGenerator
where
    T: 'static + TealDataM + UserDataM + ToTypename,
{
    fn document(&mut self, documentation: &str) {
        self.document(documentation)
    }

    fn add_field_method_get<S, R, M>(&mut self, name: S, _: M)
    where
        S: AsRef<str> + ToString,
        R: mlua::IntoLua + ToTypename,
        M: 'static + MaybeSend + Fn(&Lua, &T) -> mlua::Result<R>,
    {
        self.add_field::<S, R>(name);
    }

    fn add_field_method_set<S, A, M>(&mut self, name: S, _: M)
    where
        S: AsRef<str> + ToString,
        A: mlua::FromLua + ToTypename,
        M: 'static + MaybeSend + FnMut(&Lua, &mut T, A) -> mlua::Result<()>,
    {
        self.add_field::<S, A>(name);
    }

    fn add_field_function_get<S, R, F>(&mut self, name: S, _: F)
    where
        S: AsRef<str> + ToString,
        R: mlua::IntoLua + ToTypename,
        F: 'static + MaybeSend + Fn(&Lua, mlua::AnyUserData) -> mlua::Result<R>,
    {
        self.add_field::<S, R>(name);
    }

    fn add_field_function_set<S, A, F>(&mut self, name: S, _: F)
    where
        S: AsRef<str> + ToString,
        A: mlua::FromLua + ToTypename,
        F: 'static + MaybeSend + FnMut(&Lua, mlua::AnyUserData, A) -> mlua::Result<()>,
    {
        self.add_field::<S, A>(name);
    }

    fn add_meta_field_with<R, F>(&mut self, meta: MetaMethodM, _: F)
    where
        F: 'static + MaybeSend + Fn(&Lua) -> mlua::Result<R>,
        R: mlua::IntoLua + ToTypename,
    {
        let x = Into::<MetaMethodM>::into(meta);
        let name: Cow<'_, str> = Cow::Owned(x.name().to_string());
        self.copy_docs(name.as_bytes());
        self.static_fields
            .push((NameContainer::from(name), R::to_typename()).into());
    }
}

#[cfg(feature = "mlua")]
impl RecordGenerator {
    ///documents that this type has a field of the given type and name when exposed to lua
    pub fn add_field<S, R>(&mut self, name: S)
    where
        S: AsRef<str> + ToString,
        R: ToTypename,
    {
        self.copy_docs(name.as_ref().as_bytes());
        self.fields
            .push((name.as_ref().as_bytes().to_vec().into(), R::to_typename()).into());
    }
    /// documents that this type has a method of the given type and name when exposed to lua
    pub fn add_method<
        S: ToString + AsRef<str>,
        A: TealMultiValue,
        R: TealMultiValue,
        T: ToTypename,
    >(
        &mut self,
        name: S,
    ) {
        self.copy_docs(name.as_ref().as_bytes());
        self.methods.push(get_method_data::<A, R, _>(
            name,
            false,
            Some(T::to_typename()),
        ))
    }
    /// documents that this type has a method of the given type and name when exposed to lua
    pub fn add_method_mut<
        S: ToString + AsRef<str>,
        A: TealMultiValue,
        R: TealMultiValue,
        T: ToTypename,
    >(
        &mut self,
        name: S,
    ) {
        self.copy_docs(name.as_ref().as_bytes());
        self.mut_methods.push(get_method_data::<A, R, _>(
            name,
            false,
            Some(T::to_typename()),
        ))
    }
    /// documents that this type has a function of the given type and name when exposed to lua
    pub fn add_function<S: ToString + AsRef<str>, A: TealMultiValue, R: TealMultiValue>(
        &mut self,
        name: S,
    ) {
        self.copy_docs(name.as_ref().as_bytes());
        self.functions
            .push(get_method_data::<A, R, _>(name, false, None))
    }
    /// documents that this type has a function of the given type and name when exposed to lua
    pub fn add_function_mut<S: ToString + AsRef<str>, A: TealMultiValue, R: TealMultiValue>(
        &mut self,
        name: S,
    ) {
        self.copy_docs(name.as_ref().as_bytes());
        self.mut_functions
            .push(get_method_data::<A, R, _>(name, false, None))
    }
}
