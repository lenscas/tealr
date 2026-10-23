mod extendable;
///this module holds some pre made types that can be used to create generics.
pub mod generics;
mod named_parameters;
mod picker_macro;
pub(crate) mod teal_data;
mod teal_data_fields;
pub(crate) mod teal_data_macros;
pub(crate) mod teal_data_methods;
mod typed_function;
/// Module containing functionality to do with user data proxies
pub mod user_data_proxy;
pub(crate) mod user_data_wrapper;
mod variadics;
use std::ops::ControlFlow;

pub use self::{
    extendable::{BackMerger, Extend, Extendable, Extender},
    picker_macro::FromLuaExact,
    teal_data::TealData,
    teal_data_methods::{ExportInstances, InstanceCollector, TealDataMethods, set_global_env},
    typed_function::TypedFunction,
    user_data_proxy::UserDataProxy,
    user_data_wrapper::UserDataWrapper,
};
use crate::{ToTypename, Type, TypeGenerator, TypeWalker, type_to_string};
pub use crate::{
    create_generic_mlua as create_generic, create_union_mlua as create_union,
    mlua_create_named_parameters as create_named_parameters,
};

use itertools::Itertools;
pub use mlua;
use mlua::{UserDataRef, UserDataRefMut};
pub use teal_data_fields::TealDataFields;

#[cfg(feature = "mlua_send")]
///used by the `mlua_send` feature
pub trait MaybeSend: Send {}
#[cfg(feature = "mlua_send")]
impl<T: Send> MaybeSend for T {}

#[cfg(not(feature = "mlua_send"))]
///used by the `mlua_send` feature
pub trait MaybeSend {}
#[cfg(not(feature = "mlua_send"))]
impl<T> MaybeSend for T {}

#[doc = include_str!("mlu/to_from_macro_doc.md")]
#[cfg(feature = "derive")]
pub use tealr_derive::MluaFromToLua as FromToLua;

///Implement both [mlua::UserData](mlua::UserData) and [TypeName](crate::ToTypename).
///
///Look at [tealr_derive::MluaUserData](tealr_derive::MluaUserData) and [tealr_derive::TypeName](tealr_derive::TypeName)
///for more information on how the implemented traits will behave.
#[cfg(feature = "derive")]
pub use tealr_derive::MluaTealDerive as TealDerive;

///Implements [UserData](mlua::UserData) and [TypeBody](crate::TypeBody)
///
///It wraps the [mlua::UserDataMethods](mlua::UserDataMethods) into [UserDataWrapper](crate::mlu::UserDataWrapper)
///and then passes it to `crate::TealData::add_methods`.
///
///Type body is implemented in a similar way, where it uses the [TealData](crate::mlu::TealData) implementation to get the types
#[cfg(feature = "derive")]
pub use tealr_derive::MluaUserData as UserData;

impl<T: ToTypename> ToTypename for UserDataRef<T> {
    fn to_typename() -> Type {
        T::to_typename()
    }
    fn to_function_param() -> Vec<crate::FunctionParam> {
        T::to_function_param()
    }
}

impl<T: ToTypename> ToTypename for UserDataRefMut<T> {
    fn to_typename() -> Type {
        T::to_typename()
    }
    fn to_function_param() -> Vec<crate::FunctionParam> {
        T::to_function_param()
    }
}

create_named_parameters!(HelpParams with
    ty: Option<String>,
    method: Option<String>,
);
///Generates a help function for a specific type.
pub fn generate_help_for_type(
    lua: &mlua::Lua,
    ty: TypeGenerator,
) -> mlua::Result<TypedFunction<Option<String>, mlua::String>> {
    TypedFunction::from_rust(
        move |lua, method: Option<String>| {
            let res = match (method, &ty) {
                (None, crate::TypeGenerator::Record(record_generator)) => {
                    let mut to_show = record_generator.type_doc.to_string();
                    to_show.push_str("Methods:\n");
                    record_generator.all_functions().for_each(|v| {
                        to_show.push_str(&v.name.to_string());
                        to_show.push_str(" : ");
                        to_show.push_str(&type_to_string(&v.into_type(), false));
                        to_show.push('\n');
                    });
                    to_show.push_str("Fields:\n");
                    record_generator.fields.iter().for_each(|v| {
                        to_show.push_str(&v.name.to_string());
                        to_show.push_str(" : ");
                        to_show.push_str(&type_to_string(&v.ty, false));
                    });
                    to_show
                }
                (None | Some(_), crate::TypeGenerator::Enum(enum_generator)) => {
                    let mut to_show = enum_generator.type_doc.to_string();
                    to_show.push_str("Variants:\n");
                    enum_generator.variants.iter().for_each(|v| {
                        to_show.push_str(&v.to_string());
                        to_show.push('\n');
                    });
                    to_show
                }
                (Some(method), crate::TypeGenerator::Record(record_generator)) => record_generator
                    .all_functions()
                    .map(|v| (&v.name, v.get_all_as_types().collect_vec()))
                    .chain(
                        record_generator
                            .fields
                            .iter()
                            .map(|v| (&v.name, vec![v.ty.clone()])),
                    )
                    .find(|v| v.0.to_string() == method)
                    .map(|v| {
                        let mut to_show = v.0.to_string();
                        to_show.push_str(" : ");
                        let length = to_show.len();
                        let start = " ".repeat(length);
                        to_show.reserve(start.len() * (v.1.len() * 15 /* function() takes up 10 characters, so 15 per type is probably a good enough guess */));
                        v.1.into_iter().enumerate().for_each(|(v,i)| {
                            if v > 0 {
                                to_show.push('\n');
                                to_show.push_str(&start);
                            }
                            to_show.push_str(&type_to_string(&i, false));
                        });
                        if let Some(x) = record_generator.documentation.get(v.0) {
                            to_show.push('\n');
                            to_show.push_str(x);
                        }
                        to_show
                    })
                    .unwrap_or_else(|| {
                        let ty = type_to_string(&record_generator.ty, false);
                        format!("{method} is not a part of {ty}")
                    }),
            };
            lua.create_string(res)
        },
        lua,
    )
}
/// Generates a function that is capable of returning the documentation of any type.
///
/// The first argument is the type you want to get the documentation for.
///
/// The second argument is the method/field you want to get the documentation for.
///
/// Both arguments can be nil. If the first is nil then it will print the documentation for the type that is marked as inlined
///
/// If the second argument is nil then it will print the documentation of the found type rather than a specific method/field
///
/// Ignores the second argument when the type is an enum.
pub fn generate_help(
    lua: &mlua::Lua,
    types: TypeWalker,
) -> mlua::Result<TypedFunction<HelpParams, mlua::String>> {
    TypedFunction::from_rust(
        move |lua, params: HelpParams| {
            let mut found_types = types
                .iter()
                .filter(|x| match &params.ty {
                    None => x.is_inlined(),
                    Some(searching_for) => {
                        crate::type_to_string(x.type_name(), false).starts_with(searching_for)
                    }
                })
                .collect::<Vec<_>>();
            found_types.sort_by_cached_key(|v| match &params.ty {
                None => 10,
                Some(searching_for) => {
                    let ty_name = crate::type_to_string(v.type_name(), false);
                    let res =
                        searching_for
                            .as_bytes()
                            .iter()
                            .enumerate()
                            .try_fold(0, |acc, (i, v)| {
                                if ty_name.as_bytes()[i] == *v {
                                    ControlFlow::Continue(acc + 1)
                                } else {
                                    ControlFlow::Break(acc)
                                }
                            });
                    match res {
                        ControlFlow::Break(x) => x,
                        ControlFlow::Continue(x) => x,
                    }
                }
            });

            match found_types.first() {
                None => {
                    let types = types
                        .iter()
                        .map(|v| type_to_string(v.type_name(), false))
                        .join("\n");
                    lua.create_string(
                        "Could not find type for help.\nAvailable types:\n".to_owned() + &types,
                    )
                }
                Some(found_type) => {
                    generate_help_for_type(lua, (*found_type).to_owned())?.call(params.method)
                }
            }
        },
        lua,
    )
}
