#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

///traits and types specific to mlua
#[cfg(feature = "mlua")]
pub mod mlu;

mod export_instance;
mod exported_function;
mod teal_multivalue;
mod type_generator;
mod type_representation;
mod type_walker;

use std::{borrow::Cow, collections::HashSet};

pub use exported_function::ExportedFunction;
#[cfg(feature = "mlua")]
use mlu::TealDataMethods;
#[cfg(feature = "mlua")]
use mlua::UserDataRef;
use serde::{Deserialize, Serialize};
pub use teal_multivalue::{TealMultiValue, TealType};

///Implements [ToTypename](crate::ToTypename).
///
///`TypeName::get_type_name` will return the name of the rust type.
#[cfg(feature = "derive")]
pub use tealr_derive::ToTypename;

pub use type_generator::{EnumGenerator, Field, NameContainer, RecordGenerator, TypeGenerator};
pub use type_representation::{type_parts_to_str, KindOfType, NamePart, TypeBody, TypeName};
pub use type_walker::{ExtraPage, GlobalInstance, TypeWalker};

#[cfg(feature = "compile")]
pub use tealr_derive::compile_inline_teal;

#[cfg(any(
    feature = "embed_compiler_from_local",
    feature = "embed_compiler_from_download"
))]
pub use tealr_derive::embed_compiler;

/// Gets the current version of tealr.
///
/// Useful when consuming the .json files to check if it is a supported version
pub fn get_tealr_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize, Default)]
#[cfg_attr(
    all(feature = "mlua", feature = "derive"),
    derive(crate::mlu::FromToLua, crate::ToTypename)
)]
#[cfg_attr(
    all(feature = "mlua",feature = "derive"),
    tealr(tealr_name = crate)
)]
///The name of a type
pub struct Name(
    #[cfg_attr(
    all(feature = "mlua",feature = "derive"),
    tealr(remote =  String))]
    pub Cow<'static, str>,
);
impl core::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: AsRef<str>> From<T> for Name {
    fn from(value: T) -> Self {
        Name(value.as_ref().to_owned().into())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[cfg_attr(
    all(feature = "mlua", feature = "derive"),
    derive(crate::mlu::FromToLua, crate::ToTypename)
)]
#[cfg_attr(
    all(feature = "mlua",feature = "derive"),
    tealr(tealr_name = crate)
)]
///A singular type
pub struct SingleType {
    ///The name of the type
    pub name: Name,
    ///The kind of type that is being represented
    pub kind: KindOfType,
    ///If a type has generics then they are stored here
    pub generics: Vec<Type>,
}
#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[cfg_attr(
    all(feature = "mlua", feature = "derive"),
    derive(crate::mlu::FromToLua, crate::ToTypename)
)]
#[cfg_attr(
    all(feature = "mlua",feature = "derive"),
    tealr(tealr_name = crate)
)]
///A parameter for a function
pub struct FunctionParam {
    ///If the parameter has a name (will default to Param{number} if None)
    pub param_name: Option<Name>,
    ///The type of the parameter
    pub ty: Type,
}
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[cfg_attr(
    all(feature = "mlua", feature = "derive"),
    derive(crate::mlu::FromToLua, crate::ToTypename)
)]
#[cfg_attr(
    all(feature = "mlua",feature = "derive"),
    tealr(tealr_name = crate)
)]
///The representation of a function type
pub struct FunctionRepresentation {
    ///The parameters
    pub params: Vec<FunctionParam>,
    ///the return types
    pub returns: Vec<Type>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[cfg_attr(
    all(feature = "mlua", feature = "derive"),
    derive(crate::mlu::FromToLua, crate::ToTypename)
)]
#[cfg_attr(
    all(feature = "mlua",feature = "derive"),
    tealr(tealr_name = crate)
)]
///The representation of a Map<K,T> type
pub struct MapRepresentation {
    #[cfg_attr(
        all(feature = "mlua",feature = "derive"),
        tealr(remote =  Type))]
    ///The type of the key
    pub key: Box<Type>,
    #[cfg_attr(
        all(feature = "mlua",feature = "derive"),
        tealr(remote =  Type))]
    ///The type of the value
    pub value: Box<Type>,
}
#[allow(dead_code)]
type NewTypeArray = Vec<Type>;

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[cfg_attr(
    all(feature = "mlua", feature = "derive"),
    derive(crate::mlu::FromToLua, crate::ToTypename)
)]
#[cfg_attr(
    all(feature = "mlua",feature = "derive"),
    tealr(tealr_name = crate)
)]
#[cfg_attr(
    all(feature = "mlua",feature = "derive"),
    tealr(extend_methods = add_methods_to_type)
)]
///A type
pub enum Type {
    ///The type is a function
    Function(FunctionRepresentation),
    ///The type is a simple, singular type
    Single(SingleType),
    ///The type is a Map<K,V> (Think HashMap<K,V> and similar)
    Map(MapRepresentation),
    ///The type is a union (A | B)
    Or(
        #[cfg_attr(
            all(feature = "mlua",feature = "derive"),
            tealr(remote =  NewTypeArray))]
        Vec<Type>,
    ),
    ///The type is an array
    Array(
        #[cfg_attr(
            all(feature = "mlua",feature = "derive"),
            tealr(remote =  Type))]
        Box<Type>,
    ),
    ///This one doesn't really exist in lua/teal but will expand in (A,B,C)
    ///Sometimes useful for the return type or parameters but should be used _very_ carefully
    ///As it can _easily_ break things
    Tuple(
        #[cfg_attr(
            all(feature = "mlua",feature = "derive"),
            tealr(remote =  NewTypeArray))]
        Vec<Type>,
    ),
    ///Indicates that the given type is a variadic. Meaning it can be be repeated any amount of times
    Variadic(
        #[cfg_attr(
            all(feature = "mlua",feature = "derive"),
            tealr(remote =  Type))]
        Box<Type>,
    ),
}

#[cfg(feature = "mlua")]
fn add_methods_to_type<T: TealDataMethods<Type>>(methods: &mut T) {
    methods.add_meta_method(mlua::MetaMethod::Eq, |_, this, b: UserDataRef<Type>| {
        let a: &Type = &b;
        Ok(this == a)
    });
}

impl From<Box<Type>> for Type {
    fn from(value: Box<Type>) -> Self {
        *value
    }
}
impl Type {
    ///Creates a new singular type
    pub fn new_single(name: impl AsRef<str>, kind: KindOfType) -> Self {
        Self::new_single_with_generics(name, kind, vec![])
    }
    ///Same as `new_single` but with generics
    pub fn new_single_with_generics(
        name: impl AsRef<str>,
        kind: KindOfType,
        generics: Vec<Type>,
    ) -> Self {
        Self::Single(SingleType {
            name: name.into(),
            kind,
            generics,
        })
    }

    ///returns Some(X) if Self is `Single`. Otherwise None
    pub fn single(&self) -> Option<&SingleType> {
        if let Self::Single(x) = self {
            Some(x)
        } else {
            None
        }
    }
}
///This trait turns a A into a type representation for Lua/Teal
pub trait ToTypename {
    /// Used to get the old representation.
    /// Should basically never be used or implemented manually
    ///
    /// Note: While the result of this function can be used to get a pretty readable version of the type,
    /// It will likely miss some information that is otherwise included.
    /// Think for example of variadics, as those don't have a singular "true" way to be represented in teal.
    /// This results in that information being lost when using this function to get a readable type name.
    #[deprecated]
    fn to_old_type_parts() -> Cow<'static, [NamePart]> {
        #[allow(deprecated)]
        new_type_to_old(Self::to_typename(), false)
    }
    ///generates the type representation
    fn to_typename() -> Type;
    ///generates the type representation when used as a parameter
    ///By default will assume no name was given
    ///
    ///This is useful when the type you made is _specifically_ made to add more
    ///context to function parameters.
    fn to_function_param() -> Vec<FunctionParam> {
        vec![FunctionParam {
            param_name: None,
            ty: Self::to_typename(),
        }]
    }
}
impl<T: ToTypename> TypeName for T {
    fn get_type_parts() -> Cow<'static, [NamePart]> {
        #[allow(deprecated)]
        Self::to_old_type_parts()
    }
}
///Turns a type in the new representation into the old representation
#[deprecated]
pub fn new_type_to_old(a: Type, is_callback: bool) -> Cow<'static, [NamePart]> {
    match a {
        Type::Single(a) => {
            let mut name_parts = vec![NamePart::Type(TealType {
                name: a.name.0,
                type_kind: a.kind,
                generics: None,
            })];
            if !a.generics.is_empty() {
                name_parts.push("<".into());
                a.generics
                    .iter()
                    .map(|x| new_type_to_old(x.clone(), is_callback))
                    .enumerate()
                    .for_each(|(key, v)| {
                        if key > 0 {
                            name_parts.push(",".into());
                        }
                        name_parts.extend(v.iter().map(|x| x.to_owned()))
                    });

                name_parts.push(">".into());
            }
            Cow::Owned(name_parts)
        }
        Type::Array(x) => {
            let mut parts = Vec::with_capacity(3);
            parts.push(NamePart::symbol("{"));
            parts.extend(new_type_to_old(*x, true).iter().cloned());
            parts.push(NamePart::symbol("}"));
            parts.into()
        }
        Type::Map(MapRepresentation { key, value }) => {
            let mut parts = Vec::with_capacity(5);
            parts.push(NamePart::symbol("{"));
            parts.extend(new_type_to_old(*key, true).iter().cloned());
            parts.push(NamePart::symbol(" : "));
            parts.extend(new_type_to_old(*value, true).iter().cloned());
            parts.push(NamePart::symbol("}"));
            parts.into()
        }
        Type::Or(x) => {
            if x.is_empty() {
                eprintln!("An NewType::Or found with empty contents. Skipping");
                return Vec::new().into();
            }
            let mut parts = Vec::with_capacity(x.len());
            parts.push(NamePart::symbol("("));

            for part in x {
                parts.extend(new_type_to_old(part, true).iter().cloned());
                parts.push(NamePart::symbol(" | "))
            }
            parts.pop();
            parts.push(NamePart::symbol(")"));
            parts.into()
        }
        Type::Tuple(x) => {
            if x.is_empty() {
                eprintln!("An NewType::Tuple found with empty contents. Skipping");
                return Vec::new().into();
            }
            let mut parts = Vec::with_capacity(x.len());
            parts.push(NamePart::symbol("("));
            for part in x {
                parts.extend(new_type_to_old(part, true).iter().cloned());
                parts.push(NamePart::symbol(" , "))
            }
            parts.pop();
            parts.push(NamePart::symbol(")"));
            parts.into()
        }
        Type::Function(FunctionRepresentation { params, returns }) => {
            let mut parts = Vec::with_capacity(params.len() + returns.len());
            parts.push(NamePart::symbol("function"));
            let generics: HashSet<_> = params
                .iter()
                .map(|v| &v.ty)
                .chain(returns.iter())
                .flat_map(get_generics)
                .collect();
            let generic_amount = generics.len();
            if (!is_callback) && generic_amount > 0 {
                parts.push(NamePart::Symbol("<".into()));
                for generic in generics {
                    parts.push(NamePart::Type(TealType {
                        name: generic.0.clone(),
                        type_kind: KindOfType::Generic,
                        generics: None,
                    }));
                    parts.push(NamePart::symbol(","));
                }
                parts.pop();
                parts.push(NamePart::symbol(">"));
            }
            parts.push(NamePart::symbol("("));
            let has_params = !params.is_empty();
            for param in params {
                if let Some(name) = param.param_name {
                    parts.push(NamePart::Symbol(name.0.clone()));
                    parts.push(NamePart::symbol(":"));
                }
                parts.extend(new_type_to_old(param.ty, true).iter().cloned());
                parts.push(NamePart::symbol(" , "));
            }
            if has_params {
                parts.pop();
            }
            parts.push(NamePart::symbol(")"));
            if !returns.is_empty() {
                parts.push(NamePart::symbol(":("));
                for ret in returns {
                    parts.extend(new_type_to_old(ret, true).iter().cloned());
                    parts.push(NamePart::symbol(" , "))
                }
                parts.pop();
                parts.push(NamePart::symbol(")"));
            }

            Cow::Owned(parts)
        }
        Type::Variadic(x) => new_type_to_old(*x, is_callback),
    }
}
///Gets the generics of any given type
pub fn get_generics(to_check: &Type) -> HashSet<&Name> {
    match to_check {
        Type::Function(FunctionRepresentation { params, returns }) => params
            .iter()
            .map(|v| &v.ty)
            .chain(returns.iter())
            .flat_map(get_generics)
            .collect(),
        Type::Array(x) => get_generics(x.as_ref()),
        Type::Single(x) => {
            let mut set = HashSet::new();
            if x.kind == KindOfType::Generic {
                set.insert(&x.name);
            }
            set
        }
        Type::Or(x) | Type::Tuple(x) => x.iter().flat_map(get_generics).collect(),
        Type::Map(MapRepresentation { key, value }) => {
            let mut generics = get_generics(key);
            generics.extend(get_generics(value));
            generics
        }
        Type::Variadic(x) => get_generics(x.as_ref()),
    }
}
