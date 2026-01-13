#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

///traits and types specific to mlua
#[cfg(feature = "mlua")]
pub mod mlu;

mod export_instance;
mod exported_function;
mod macro_expressions;
mod teal_multivalue;
mod type_generator;
mod type_representation;
mod type_walker;

use std::{borrow::Cow, collections::HashSet};

pub use exported_function::ExportedFunction;
#[cfg(feature = "self_to_lua")]
use mlu::TealDataMethods;
#[cfg(feature = "self_to_lua")]
use mlua::UserDataRef;
use serde::{Deserialize, Serialize};
pub use teal_multivalue::{TealMultiValue, TealType};

///Implements [ToTypename].
///
///`TypeName::get_type_name` will return the name of the rust type.
#[cfg(feature = "derive")]
pub use tealr_derive::ToTypename;

pub use macro_expressions::MacroExpr;
pub use type_generator::{EnumGenerator, Field, NameContainer, RecordGenerator, TypeGenerator};
pub use type_representation::{KindOfType, NamePart, TypeBody};
pub use type_walker::{ExtraPage, GlobalInstance, TypeWalker};

#[cfg(feature = "compile")]
pub use tealr_derive::compile_inline_teal;

#[cfg(any(
    feature = "embed_compiler_from_local",
    feature = "embed_compiler_from_download"
))]
pub use tealr_derive::embed_compiler;

#[cfg(feature = "self_to_lua")]
use crate::mlu::teal_data_macros::TealDataMacros;

/// Gets the current version of tealr.
///
/// Useful when consuming the .json files to check if it is a supported version
pub fn get_tealr_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize, Default)]
#[cfg_attr(
    feature = "self_to_lua",
    derive(crate::mlu::FromToLua, crate::ToTypename)
)]
#[cfg_attr(
    feature = "self_to_lua",
    tealr(tealr_name = crate)
)]
///The name of a type
pub struct Name(
    #[cfg_attr(
    feature = "self_to_lua",
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
    feature = "self_to_lua",
    derive(crate::mlu::FromToLua, crate::ToTypename)
)]
#[cfg_attr(
    feature = "self_to_lua",
    tealr(tealr_name = crate)
)]
#[cfg_attr(feature = "self_to_lua", tealr(tag = "tealType"))]
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
    feature = "self_to_lua",
    derive(crate::mlu::FromToLua, crate::ToTypename)
)]
#[cfg_attr(
    feature = "self_to_lua",
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
    feature = "self_to_lua",
    derive(crate::mlu::FromToLua, crate::ToTypename)
)]
#[cfg_attr(
    feature = "self_to_lua",
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
    feature = "self_to_lua",
    derive(crate::mlu::FromToLua, crate::ToTypename)
)]
#[cfg_attr(
    feature = "self_to_lua",
    tealr(tealr_name = crate)
)]
///The representation of a Map<K,T> type
pub struct MapRepresentation {
    #[cfg_attr(
        feature = "self_to_lua",
        tealr(remote =  Type))]
    ///The type of the key
    pub key: Box<Type>,
    #[cfg_attr(
        feature = "self_to_lua",
        tealr(remote =  Type))]
    ///The type of the value
    pub value: Box<Type>,
}
#[allow(dead_code)]
type NewTypeArray = Vec<Type>;

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[cfg_attr(
    feature = "self_to_lua",
    derive(crate::mlu::FromToLua, crate::ToTypename)
)]
#[cfg_attr(feature = "self_to_lua",
    tealr(tealr_name = crate)
)]
#[cfg_attr(feature = "self_to_lua",
tealr(extend_methods = add_methods_to_type)
)]
#[cfg_attr(feature = "self_to_lua",
tealr(extend_macros = add_macros_to_type)
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
            feature = "self_to_lua",
            tealr(remote =  NewTypeArray))]
        Vec<Type>,
    ),
    ///The type is an array
    Array(
        #[cfg_attr(
            feature = "self_to_lua",
            tealr(remote =  Type))]
        Box<Type>,
    ),
    ///This one doesn't really exist in lua/teal but will expand in (A,B,C)
    ///Sometimes useful for the return type or parameters but should be used _very_ carefully
    ///As it can _easily_ break things
    Tuple(
        #[cfg_attr(
            feature = "self_to_lua",
            tealr(remote =  NewTypeArray))]
        Vec<Type>,
    ),
    ///Indicates that the given type is a variadic. Meaning it can be be repeated any amount of times
    Variadic(
        #[cfg_attr(
            feature = "self_to_lua",
            tealr(remote =  Type))]
        Box<Type>,
    ),
}

#[cfg(feature = "self_to_lua")]
fn add_methods_to_type<T: TealDataMethods<Type>>(methods: &mut T) {
    methods.add_meta_method(mlua::MetaMethod::Eq, |_, this, b: UserDataRef<Type>| {
        let a: &Type = &b;
        Ok(this == a)
    });
    methods.add_meta_method(mlua::MetaMethod::ToString, |_, this, ()| {
        Ok(format!("{:?}", this))
    })
}

#[cfg(feature = "self_to_lua")]
fn add_macros_to_type<T: TealDataMacros<Type>>(macros: &mut T) {
    type Foo = crate::mlu::TypedFunction<FunctionRepresentation, crate::mlu::generics::R>;
    let x = Field::new::<Foo>("func");
    macros.add_macro::<crate::mlu::generics::R>(
        "on_function",
        vec![Field::new::<Type>("self"), x],
        "return self:isFunction() and func(self.FunctionOrNil())",
    );
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
///Turns a Type into a readable string based on Teal's syntax
pub fn type_to_string(a: &Type, is_callback: bool) -> String {
    type_to_teal_parts(a, is_callback)
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join("")
}

///Turns a Type into a representation that is closer to how it should be displayed while keeping type information intact
pub fn type_to_teal_parts(a: &Type, is_callback: bool) -> Cow<'static, [NamePart]> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum AsPartOf {
        Param,
        Return,
        Other,
    }
    fn type_to_teal_parts_helper(
        a: &Type,
        is_callback: bool,
        as_part_of: AsPartOf,
    ) -> Cow<'static, [NamePart]> {
        match a {
            Type::Single(a) => {
                let mut name_parts = vec![NamePart::Type(TealType {
                    name: a.name.0.clone(),
                    type_kind: a.kind.clone(),
                    generics: None,
                })];
                if !a.generics.is_empty() {
                    name_parts.push("<".into());
                    a.generics
                        .iter()
                        .map(|x| type_to_teal_parts_helper(x, is_callback, AsPartOf::Other))
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
                parts.extend(
                    type_to_teal_parts_helper(x, true, AsPartOf::Other)
                        .iter()
                        .cloned(),
                );
                parts.push(NamePart::symbol("}"));
                parts.into()
            }
            Type::Map(MapRepresentation { key, value }) => {
                let mut parts = Vec::with_capacity(5);
                parts.push(NamePart::symbol("{"));
                parts.extend(
                    type_to_teal_parts_helper(key, true, AsPartOf::Other)
                        .iter()
                        .cloned(),
                );
                parts.push(NamePart::symbol(" : "));
                parts.extend(
                    type_to_teal_parts_helper(value, true, AsPartOf::Other)
                        .iter()
                        .cloned(),
                );
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
                    parts.extend(
                        type_to_teal_parts_helper(part, true, as_part_of)
                            .iter()
                            .cloned(),
                    );
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
                    parts.extend(
                        type_to_teal_parts_helper(part, true, as_part_of)
                            .iter()
                            .cloned(),
                    );
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
                    let name = if matches!(&param.ty, Type::Variadic(_)) {
                        &Some("...".into())
                    } else {
                        &param.param_name
                    };
                    if let Some(name) = name {
                        parts.push(NamePart::Symbol(name.0.clone()));
                        parts.push(NamePart::symbol(":"));
                    }
                    parts.extend(
                        type_to_teal_parts_helper(&param.ty, true, AsPartOf::Param)
                            .iter()
                            .cloned(),
                    );
                    parts.push(NamePart::symbol(" , "));
                }
                if has_params {
                    parts.pop();
                }
                parts.push(NamePart::symbol(")"));
                if !returns.is_empty() {
                    parts.push(NamePart::symbol(":("));
                    for ret in returns {
                        parts.extend(
                            type_to_teal_parts_helper(ret, true, AsPartOf::Return)
                                .iter()
                                .cloned(),
                        );
                        if matches!(&ret, Type::Variadic(_)) {
                            parts.push(NamePart::symbol("..."));
                        }
                        parts.push(NamePart::symbol(" , "))
                    }
                    parts.pop();
                    parts.push(NamePart::symbol(")"));
                }

                Cow::Owned(parts)
            }
            Type::Variadic(x) => {
                if as_part_of != AsPartOf::Param && as_part_of != AsPartOf::Return {
                    eprintln!(
                        "An NewType::Variadic found that is not a param or return. This should _not_ happen");
                }
                let mut full_result = Vec::new();
                let res = type_to_teal_parts_helper(x, is_callback, as_part_of);
                full_result.extend(res.iter());
                if as_part_of == AsPartOf::Return {
                    full_result.push(&NamePart::symbol("..."));
                }
                res
            }
        }
    }
    type_to_teal_parts_helper(a, is_callback, AsPartOf::Other)
}
///Gets the names of generics of any given type
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
///Gets the generics of any given type
pub fn get_generic_types(to_check: &Type) -> HashSet<Type> {
    match to_check {
        Type::Function(FunctionRepresentation { params, returns }) => params
            .iter()
            .map(|v| &v.ty)
            .chain(returns.iter())
            .flat_map(get_generic_types)
            .collect(),
        Type::Array(x) => get_generic_types(x.as_ref()),
        Type::Single(x) => {
            let mut set = HashSet::new();
            if x.kind == KindOfType::Generic {
                set.insert(Type::Single(x.clone()));
            }
            set
        }
        Type::Or(x) | Type::Tuple(x) => x.iter().flat_map(get_generic_types).collect(),
        Type::Map(MapRepresentation { key, value }) => {
            let mut generics = get_generic_types(key);
            generics.extend(get_generic_types(value));
            generics
        }
        Type::Variadic(x) => get_generic_types(x.as_ref()),
    }
}

//debug macro
