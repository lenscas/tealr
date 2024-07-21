use std::{
    collections::{BTreeMap, HashMap},
    convert::TryInto,
    ffi::{CStr, CString},
    num::TryFromIntError,
};

use rlua::{Context, Error, Function, Table, Value};

/// similar to [rlua::FromLua](rlua::FromLua). However,
/// however going through this trait you promise that the conversion to a rust value prefers failing over converting/casting
pub trait FromLuaExact<'lua>: Sized {
    ///Does the conversion, without any type conversion/casting
    fn from_lua_exact(value: rlua::Value<'lua>, lua: rlua::Context<'lua>) -> rlua::Result<Self>;
}

///Creates a new type that is a union of the types you gave.
///
///It gets translated to a [union](https://github.com/teal-language/tl/blob/master/docs/tutorial.md#union-types) type in `teal`
///and an enum on Rust.
///
///# Warning:
///`teal` has a few restrictions on what it finds a valid union types. `tealr` does ***NOT*** check if the types you put in are a valid combination
///
///# Example
///```no_run
///# use tealr::create_union_rlua;
///create_union_rlua!(pub enum YourPublicType = String | f64 | bool);
///create_union_rlua!(enum YourType = String | f64 | bool);
///```
///
/// It does the conversion by going through the list of possible types and trying to turn the lua value into a Rust type.
/// If the conversion succeeded then it is assumed that the given lua value corresponds to the given rust type
///
/// Because of this, it is _very_ important that the Lua -> Rust conversion does as little "type massaging" as possible.
/// As a result, the macro only works with types that implement [FromLuaExact] as implementing this
/// for a type should mean the conversion rather fails than to try and make it work
#[macro_export]
macro_rules! create_union_rlua {
    ($visibility:vis $(Derives($($derives:ident), +))? enum $type_name:ident = $($sub_types:ident) | +) => {
        #[derive(Clone,$($($derives ,)*)*)]
        #[allow(non_camel_case_types)]
        $visibility enum $type_name {
            $($sub_types($sub_types) ,)*
        }
        impl<'lua> $crate::rlu::rlua::ToLua<'lua> for $type_name {
            fn to_lua(self, lua: $crate::rlu::rlua::Context<'lua>) -> ::std::result::Result<$crate::rlu::rlua::Value<'lua>, $crate::rlu::rlua::Error> {
                match self {
                    $($type_name::$sub_types(x) => x.to_lua(lua),)*
                }
            }
        }
        impl<'lua> $crate::rlu::rlua::FromLua<'lua> for $type_name {
            fn from_lua(value: $crate::rlu::rlua::Value<'lua>, lua: $crate::rlu::rlua::Context<'lua>) -> ::std::result::Result<Self, $crate::rlu::rlua::Error> {
                $(match <$sub_types as $crate::rlu::FromLuaExact>::from_lua_exact(value.clone(),lua) {
                    Ok(x) => return Ok($type_name::$sub_types(x)),
                    Err($crate::rlu::rlua::Error::FromLuaConversionError{from:_,to:_,message:_}) => {}
                    Err(x) => return Err(x)
                };)*
                Err($crate::rlu::rlua::Error::FromLuaConversionError{
                    to: stringify!( $($sub_types)|* ),
                    from: $crate::rlu::get_type_name(&value),
                    message: None
                })
            }
        }
        impl<'lua> $crate::rlu::FromLuaExact<'lua> for $type_name {
            fn from_lua_exact(value: $crate::rlu::rlua::Value<'lua>, lua: $crate::rlu::rlua::Context<'lua>) -> ::std::result::Result<Self, $crate::rlu::rlua::Error> {
                <Self as $crate::rlu::rlua::FromLua>::from_lua(value,lua)
            }
        }
        impl $crate::ToTypename for $type_name {
            fn to_typename() -> $crate::Type {
                let mut types = Vec::new();
                $(
                    types.push(<$sub_types as $crate::ToTypename>::to_typename());
                )*

                $crate::Type::Or(types)
            }
        }

    };
}

impl<'lua> FromLuaExact<'lua> for String {
    fn from_lua_exact(value: rlua::Value<'lua>, _: rlua::Context<'lua>) -> rlua::Result<Self> {
        match value {
            rlua::Value::String(x) => Ok(x.to_str()?.to_owned()),
            x => Err(rlua::Error::FromLuaConversionError {
                from: x.type_name(),
                to: "String",
                message: None,
            }),
        }
    }
}

impl<'lua> FromLuaExact<'lua> for CString {
    fn from_lua_exact(value: rlua::Value<'lua>, lua: rlua::Context<'lua>) -> rlua::Result<Self> {
        let ty = value.type_name();
        let as_str = rlua::String::from_lua_exact(value, lua)?;
        match CStr::from_bytes_with_nul(as_str.as_bytes_with_nul()) {
            Ok(x) => Ok(x.to_owned()),
            Err(_) => Err(rlua::Error::FromLuaConversionError {
                from: ty,
                to: "CString",
                message: None,
            }),
        }
    }
}

macro_rules! impl_from_exact_non_failing {
    ($T:ty, $conv:pat, $bound_on:ident) => {
        impl<'lua> FromLuaExact<'lua> for $T {
            fn from_lua_exact(
                value: rlua::Value<'lua>,
                _: rlua::Context<'lua>,
            ) -> rlua::Result<Self> {
                match value {
                    $conv => Ok($bound_on.into()),
                    x => Err(rlua::Error::FromLuaConversionError {
                        from: x.type_name(),
                        to: stringify!($T),
                        message: None,
                    }),
                }
            }
        }
    };
}

macro_rules! impl_from_exact {
    ($T:ty, $conv:pat, $bound_on:ident, $error:ident) => {
        impl<'lua> FromLuaExact<'lua> for $T {
            fn from_lua_exact(
                value: rlua::Value<'lua>,
                _: rlua::Context<'lua>,
            ) -> rlua::Result<Self> {
                let ty = value.type_name();
                match value {
                    $conv => $bound_on.try_into().map_err(|x: $error| {
                        rlua::Error::FromLuaConversionError {
                            from: ty,
                            to: stringify!($T),
                            message: Some(x.to_string()),
                        }
                    }),
                    _ => Err(rlua::Error::FromLuaConversionError {
                        from: ty,
                        to: stringify!($T),
                        message: None,
                    }),
                }
            }
        }
    };
}

impl<'lua, T: FromLuaExact<'lua>> FromLuaExact<'lua> for Option<T> {
    fn from_lua_exact(value: rlua::Value<'lua>, lua: rlua::Context<'lua>) -> rlua::Result<Self> {
        match value {
            rlua::Value::Nil => Ok(None),
            x => T::from_lua_exact(x, lua).map(Some),
        }
    }
}

impl<'lua, T: FromLuaExact<'lua>> FromLuaExact<'lua> for Vec<T> {
    fn from_lua_exact(value: rlua::Value<'lua>, lua: rlua::Context<'lua>) -> rlua::Result<Self> {
        match value {
            rlua::Value::Table(x) => x
                .sequence_values()
                .map(|x| match x {
                    Ok(x) => T::from_lua_exact(x, lua),
                    Err(x) => Err(x),
                })
                .collect(),
            x => Err(rlua::Error::FromLuaConversionError {
                from: x.type_name(),
                to: "Vec",
                message: None,
            }),
        }
    }
}

impl<
        'lua,
        K: Eq + std::hash::Hash + FromLuaExact<'lua>,
        V: FromLuaExact<'lua>,
        S: std::hash::BuildHasher + Default,
    > FromLuaExact<'lua> for HashMap<K, V, S>
{
    fn from_lua_exact(value: rlua::Value<'lua>, lua: rlua::Context<'lua>) -> rlua::Result<Self> {
        if let rlua::Value::Table(table) = value {
            table
                .pairs()
                .map(|x| match x {
                    Ok((key, value)) => K::from_lua_exact(key, lua)
                        .and_then(|key| V::from_lua_exact(value, lua).map(|value| (key, value))),
                    Err(x) => Err(x),
                })
                .collect()
        } else {
            Err(rlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "HashMap",
                message: Some("expected table".to_string()),
            })
        }
    }
}

impl<'lua, K: Ord + FromLuaExact<'lua>, V: FromLuaExact<'lua>> FromLuaExact<'lua>
    for BTreeMap<K, V>
{
    fn from_lua_exact(value: Value<'lua>, lua: Context<'lua>) -> rlua::Result<Self> {
        if let Value::Table(table) = value {
            table
                .pairs()
                .map(|x| match x {
                    Ok((key, value)) => K::from_lua_exact(key, lua)
                        .and_then(|key| V::from_lua_exact(value, lua).map(|value| (key, value))),
                    Err(x) => Err(x),
                })
                .collect()
        } else {
            Err(Error::FromLuaConversionError {
                from: value.type_name(),
                to: "BTreeMap",
                message: Some("expected table".to_string()),
            })
        }
    }
}

impl<'lua, T: FromLuaExact<'lua>, const N: usize> FromLuaExact<'lua> for [T; N] {
    fn from_lua_exact(value: Value<'lua>, lua: Context<'lua>) -> rlua::Result<Self> {
        let as_vec = Vec::<T>::from_lua_exact(value, lua).map_err(|x| match x {
            Error::FromLuaConversionError {
                from,
                to: _,
                message: _,
            } => Error::FromLuaConversionError {
                from,
                to: "Array",
                message: Some(format!("Expected array of exactly length {}", N)),
            },
            x => x,
        })?;
        let len = as_vec.len();
        if len != N {
            return Err(Error::FromLuaConversionError {
                from: "Table",
                to: "Array",
                message: Some(format!(
                    "Expected array of exactly length {}, got {}",
                    N, len
                )),
            });
        }
        match as_vec.try_into() {
            Ok(array) => Ok(array),
            Err(_) => unreachable!(),
        }
    }
}

impl_from_exact_non_failing!(bool, rlua::Value::Boolean(x), x);

impl_from_exact_non_failing!(Function<'lua>, rlua::Value::Function(x), x);

impl_from_exact_non_failing!(Table<'lua>, rlua::Value::Table(x), x);

impl_from_exact_non_failing!(rlua::String<'lua>, rlua::Value::String(x), x);

#[cfg(all(
    not(feature = "rlua_builtin-lua51"),
    not(feature = "rlua_system-lua51")
))]
impl_from_exact!(i8, rlua::Value::Integer(x), x, TryFromIntError);
#[cfg(all(
    not(feature = "rlua_builtin-lua51"),
    not(feature = "rlua_system-lua51")
))]
impl_from_exact!(u8, rlua::Value::Integer(x), x, TryFromIntError);
#[cfg(all(
    not(feature = "rlua_builtin-lua51"),
    not(feature = "rlua_system-lua51")
))]
impl_from_exact!(i16, rlua::Value::Integer(x), x, TryFromIntError);
#[cfg(all(
    not(feature = "rlua_builtin-lua51"),
    not(feature = "rlua_system-lua51")
))]
impl_from_exact!(u16, rlua::Value::Integer(x), x, TryFromIntError);
#[cfg(all(
    target_pointer_width = "32",
    not(feature = "rlua_builtin-lua51"),
    not(feature = "rlua_system-lua51")
))]
impl_from_exact_non_failing!(i32, rlua::Value::Integer(x), x);
#[cfg(all(
    target_pointer_width = "64",
    not(feature = "rlua_builtin-lua51"),
    not(feature = "rlua_system-lua51")
))]
impl_from_exact!(i32, rlua::Value::Integer(x), x, TryFromIntError);
#[cfg(all(
    not(feature = "rlua_builtin-lua51"),
    not(feature = "rlua_system-lua51")
))]
impl_from_exact!(u32, rlua::Value::Integer(x), x, TryFromIntError);
#[cfg(all(
    not(feature = "rlua_builtin-lua51"),
    not(feature = "rlua_system-lua51")
))]
impl_from_exact_non_failing!(i64, rlua::Value::Integer(x), x);
#[cfg(all(
    not(feature = "rlua_builtin-lua51"),
    not(feature = "rlua_system-lua51")
))]
impl_from_exact!(u64, rlua::Value::Integer(x), x, TryFromIntError);
#[cfg(all(
    not(feature = "rlua_builtin-lua51"),
    not(feature = "rlua_system-lua51")
))]
impl_from_exact_non_failing!(i128, rlua::Value::Integer(x), x);
#[cfg(all(
    not(feature = "rlua_builtin-lua51"),
    not(feature = "rlua_system-lua51")
))]
impl_from_exact!(u128, rlua::Value::Integer(x), x, TryFromIntError);
#[cfg(all(
    not(feature = "rlua_builtin-lua51"),
    not(feature = "rlua_system-lua51")
))]
impl_from_exact!(isize, rlua::Value::Integer(x), x, TryFromIntError);
#[cfg(all(
    not(feature = "rlua_builtin-lua51"),
    not(feature = "rlua_system-lua51")
))]
impl_from_exact!(usize, rlua::Value::Integer(x), x, TryFromIntError);

impl_from_exact_non_failing!(f64, rlua::Value::Number(x), x);

impl<'lua> FromLuaExact<'lua> for f32 {
    fn from_lua_exact(value: rlua::Value<'lua>, lua: rlua::Context<'lua>) -> rlua::Result<Self> {
        f64::from_lua_exact(value, lua).map(|x| x as f32)
    }
}
