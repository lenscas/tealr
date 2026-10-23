#![cfg_attr(docsrs, feature(doc_cfg))]

//! Core library for tealr_doc_gen
//!
//! It contains various helper functions to nicer transform the collected types, functions for basic warnings, etc.
use std::{
    error::Error,
    fmt::Display,
    fs::create_dir_all,
    io,
    path::{Path, PathBuf},
};

use tealr::{ExportedFunction, KindOfType, Name, SingleType, Type, TypeGenerator, TypeWalker};

pub mod configuration;
#[cfg(feature = "definitions")]
pub mod definition_files;
pub mod find_users;
#[cfg(feature = "html")]
pub mod html;
#[cfg(feature = "lua_addon")]
pub mod lua_addon;
#[cfg(feature = "warnings")]
pub mod warnings;

/// Checks if two types are compatible
/// Types are compatible if they have the same amount of generics and the same name
pub fn is_type_compatible(ty: &tealr::SingleType, other: tealr::SingleType) -> bool {
    let other = SingleTypeNoConsume::from(other);
    is_type_compatible_helper(ty, &other)
}

fn is_type_compatible_helper(ty: &tealr::SingleType, other: &SingleTypeNoConsume) -> bool {
    ty.name == other.name && ty.generics.len() == other.generics.len()
}

#[cfg(feature = "lua_support")]
use tealr::mlu::{
    mlua,
    mlua::{FromLua, IntoLua, Table, UserDataRef},
};

use crate::configuration::Paths;
/// A stand in for UserDataRef is `lua_support` is not enabled
#[cfg(not(feature = "lua_support"))]
pub struct UserDataRef<T>(T);

/// Some functions in Rust that take a [SingleType](tealr::SingleType) need to also be called from lua
/// as well _without_ consuming the generics. This type is part of [SingleTypeNoConsume] to make this possible.
pub enum OwnedOrNot {
    Owned(Vec<Type>),
    Reference(Vec<UserDataRef<Type>>),
}
impl OwnedOrNot {
    fn len(&self) -> usize {
        match self {
            OwnedOrNot::Owned(user_data_refs) => user_data_refs.len(),
            OwnedOrNot::Reference(user_data_refs) => user_data_refs.len(),
        }
    }
}
/// A version of SingleType that does not consume the generics.
/// Thus keeping it valid when called from lua
pub struct SingleTypeNoConsume {
    pub name: Name,
    pub kind: KindOfType,
    pub generics: OwnedOrNot,
}
#[cfg(feature = "lua_support")]
impl FromLua for SingleTypeNoConsume {
    fn from_lua(value: mlua::Value, lua: &mlua::Lua) -> mlua::Result<Self> {
        let table = Table::from_lua(value, lua)?;
        let generics = Vec::<UserDataRef<Type>>::from_lua(table.get("generics")?, lua)?;
        let name = Name::from_lua(table.get("name")?, lua)?;
        let kind = KindOfType::from_lua(table.get("kind")?, lua)?;
        Ok(Self {
            generics: OwnedOrNot::Reference(generics),
            name,
            kind,
        })
    }
}
#[cfg(feature = "lua_support")]
impl IntoLua for SingleTypeNoConsume {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        let table = lua.create_table()?;
        let generics = match self.generics {
            OwnedOrNot::Owned(user_data_refs) => user_data_refs
                .into_iter()
                .map(|x| x.clone().into_lua(lua))
                .collect::<Result<Vec<_>, _>>()?,
            OwnedOrNot::Reference(user_data_refs) => user_data_refs
                .into_iter()
                .map(|x| x.clone().into_lua(lua))
                .collect::<Result<Vec<_>, _>>()?,
        };
        table.set("generics", generics)?;
        table.set("name", self.name)?;
        table.set("kind", self.kind)?;
        table.into_lua(lua)
    }
}
#[cfg(feature = "lua_support")]
impl tealr::TypeBody for SingleTypeNoConsume {
    fn get_type_body() -> TypeGenerator {
        SingleType::get_type_body()
    }
}

impl From<SingleType> for SingleTypeNoConsume {
    fn from(value: SingleType) -> Self {
        Self {
            generics: OwnedOrNot::Owned(value.generics),
            kind: value.kind,
            name: value.name,
        }
    }
}
#[cfg(feature = "lua_support")]
impl tealr::ToTypename for SingleTypeNoConsume {
    fn to_typename() -> Type {
        SingleType::to_typename()
    }

    fn to_function_param() -> Vec<tealr::FunctionParam> {
        SingleType::to_function_param()
    }
}

/// Helper to see if a type is marked as needing to be "inlined"
/// Inlined meaning that it is part of the index page.
///
/// Mostly used when creating lua modules
pub fn type_should_be_inlined(v: &TypeGenerator) -> bool {
    match v {
        TypeGenerator::Record(x) => x.should_be_inlined,
        TypeGenerator::Enum(_) => false,
    }
}

/// Gets the folder where the definition files should be written to.
pub fn get_definition_file_path(paths: &Paths) -> PathBuf {
    Path::new(&paths.build_dir)
        .join(&paths.root)
        .join("definitions")
}

/// Ensures every directory is created, the functions to create the files do not do this.
/// As such you have to call this function yourself first or ensure the directories exist in other means.
pub fn create_all_dirs(paths: &Paths) -> Result<(), std::io::Error> {
    let write_path = get_definition_file_path(paths);
    create_dir_all(&write_path)?;
    Ok(())
}

/// Goes through the Type and find all generics that make it up.
///
/// If a SingleType got given that is just the generics of that type.
/// For a Function it will recurse to find all the generics of both the parameters and the return value.
/// For map it goes over the keys and values, etc.
///
pub fn find_all_generics(ty: &tealr::Type) -> Vec<Type> {
    let mut generics = Vec::new();
    match ty {
        tealr::Type::Single(single_type) => {
            if single_type.kind.is_generic() {
                generics.push(Type::Single(single_type.clone()));
            }
            single_type
                .generics
                .iter()
                .map(find_all_generics)
                .for_each(|x| {
                    generics.extend(x);
                });
        }
        tealr::Type::Map(map_representation) => {
            generics.extend(find_all_generics(&map_representation.key));
            generics.extend(find_all_generics(&map_representation.value));
        }
        tealr::Type::Tuple(items) | tealr::Type::Or(items) => {
            generics.extend(items.iter().flat_map(find_all_generics));
        }
        tealr::Type::Variadic(x) | tealr::Type::Array(x) => {
            generics.extend(find_all_generics(x));
        }
        tealr::Type::Function(function_representation) => {
            let x = ExportedFunction {
                name: std::borrow::Cow::from("foo").into(),
                params: function_representation.params.to_owned(),
                returns: function_representation.returns.to_owned(),
                is_meta_method: false,
                is_hidden: false,
                overloads: Vec::new(),
            };
            generics.extend(x.get_generic_types().into_iter().map(|v| v.to_owned()));
        }
    }
    generics
}

/// Convenience method to load stored typewalkers without having to depend on serde directly
///
/// Will parse the given json file and produce a TypeWalker.
///
/// Same as `serde_json::from_reader::<_,TypeWalker>(reader)`
pub fn type_walker_from_json<R: io::Read>(reader: R) -> Result<TypeWalker, serde_json::Error> {
    serde_json::from_reader(reader)
}

#[derive(Debug)]
pub enum LoadError {
    Io(std::io::Error),
    Deserialize(serde_json::Error),
}

impl From<io::Error> for LoadError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
impl From<serde_json::Error> for LoadError {
    fn from(value: serde_json::Error) -> Self {
        Self::Deserialize(value)
    }
}

impl Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::Io(error) => error.fmt(f),
            LoadError::Deserialize(error) => error.fmt(f),
        }
    }
}

impl Error for LoadError {}

/// Convenience method to load stored typewalkers without having to depend on serde directly
/// Takes a path to a .json file, parses it into a TypeWalker and returns the result.
pub fn load_typewalker_from_json(path: &Path) -> Result<TypeWalker, LoadError> {
    let x = std::fs::File::open(path)?;
    let x = std::io::BufReader::new(x);
    type_walker_from_json(x).map_err(Into::into)
}
