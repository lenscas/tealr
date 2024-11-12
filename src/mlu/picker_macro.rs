use mlua::{Error, Function, Lua, Table, Value};
use std::ops::Deref;
use std::{
    collections::{BTreeMap, HashMap},
    ffi::{CStr, CString},
    num::TryFromIntError,
};

/// similar to [mlua::FromLua](mlua::FromLua). However,
/// however going through this trait you promise that the conversion to a rust value prefers failing over converting/casting
pub trait FromLuaExact: Sized {
    ///Does the conversion, without any type conversion/casting
    fn from_lua_exact(value: Value, lua: &Lua) -> mlua::Result<Self>;
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
///# use tealr::create_union_mlua;
///create_union_mlua!(pub enum YourPublicType = String | f64 | bool);
///create_union_mlua!(enum YourType = String | f64 | bool);
///```
///
/// It does the conversion by going through the list of possible types and trying to turn the lua value into a Rust type.
/// If the conversion succeeded then it is assumed that the given lua value corresponds to the given rust type
///
/// Because of this, it is _very_ important that the Lua -> Rust conversion does as little "type massaging" as possible.
/// As a result, the macro only works with types that implement [FromLuaExact] as implementing this
/// for a type should mean the conversion rather fails than to try and make it work
#[macro_export]
macro_rules! create_union_mlua {
    ($visibility:vis $(Derives($($derives:ident), +))? enum $type_name:ident = $($sub_types:ident) | +) => {
        #[derive(Clone,$($($derives ,)*)*)]
        #[allow(non_camel_case_types)]
        $visibility enum $type_name {
            $($sub_types($sub_types) ,)*
        }
        impl $crate::mlu::mlua::IntoLua for $type_name {
            fn into_lua(self, lua: &$crate::mlu::mlua::Lua) -> ::std::result::Result<$crate::mlu::mlua::Value, $crate::mlu::mlua::Error> {
                match self {
                    $($type_name::$sub_types(x) => x.into_lua(lua),)*
                }
            }
        }
        impl $crate::mlu::mlua::FromLua for $type_name {
            fn from_lua(value: $crate::mlu::mlua::Value, lua: &$crate::mlu::mlua::Lua) -> ::std::result::Result<Self, $crate::mlu::mlua::Error> {
                $(match <$sub_types as $crate::mlu::FromLuaExact>::from_lua_exact(value.clone(),lua) {
                    Ok(x) => return Ok($type_name::$sub_types(x)),
                    Err($crate::mlu::mlua::Error::FromLuaConversionError{from:_,to:_,message:_}) => {}
                    Err(x) => return Err(x)
                };)*
                Err($crate::mlu::mlua::Error::FromLuaConversionError{
                    to: stringify!( $($sub_types)|* ).to_string(),
                    from: value.type_name(),
                    message: None
                })
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

impl FromLuaExact for String {
    fn from_lua_exact(value: Value, _: &Lua) -> mlua::Result<Self> {
        match value {
            Value::String(x) => Ok(x.to_str()?.to_owned()),
            x => Err(Error::FromLuaConversionError {
                from: x.type_name(),
                to: "String".to_string(),
                message: None,
            }),
        }
    }
}

impl FromLuaExact for CString {
    fn from_lua_exact(value: Value, lua: &Lua) -> mlua::Result<Self> {
        let ty = value.type_name();
        let as_str = mlua::String::from_lua_exact(value, lua)?;
        match CStr::from_bytes_with_nul(as_str.as_bytes_with_nul().deref()) {
            Ok(x) => Ok(x.to_owned()),
            Err(_) => Err(Error::FromLuaConversionError {
                from: ty,
                to: "CString".to_string(),
                message: None,
            }),
        }
    }
}

macro_rules! impl_from_exact_non_failing {
    ($T:ty, $conv:pat, $bound_on:ident) => {
        impl FromLuaExact for $T {
            fn from_lua_exact(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
                match value {
                    $conv => Ok($bound_on.into()),
                    x => Err(mlua::Error::FromLuaConversionError {
                        from: x.type_name(),
                        to: stringify!($T).to_string(),
                        message: None,
                    }),
                }
            }
        }
    };
}

macro_rules! impl_from_exact {
    ($T:ty, $conv:pat, $bound_on:ident, $error:ident) => {
        impl FromLuaExact for $T {
            fn from_lua_exact(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
                let ty = value.type_name();
                match value {
                    $conv => $bound_on.try_into().map_err(|x: $error| {
                        mlua::Error::FromLuaConversionError {
                            from: ty,
                            to: stringify!($T).to_string(),
                            message: Some(x.to_string()),
                        }
                    }),
                    _ => Err(mlua::Error::FromLuaConversionError {
                        from: ty,
                        to: stringify!($T).to_string(),
                        message: None,
                    }),
                }
            }
        }
    };
}

impl<T: FromLuaExact> FromLuaExact for Option<T> {
    fn from_lua_exact(value: Value, lua: &Lua) -> mlua::Result<Self> {
        match value {
            Value::Nil => Ok(None),
            x => T::from_lua_exact(x, lua).map(Some),
        }
    }
}

impl<T: FromLuaExact> FromLuaExact for Vec<T> {
    fn from_lua_exact(value: Value, lua: &Lua) -> mlua::Result<Self> {
        match value {
            Value::Table(x) => x
                .sequence_values()
                .map(|x| match x {
                    Ok(x) => T::from_lua_exact(x, lua),
                    Err(x) => Err(x),
                })
                .collect(),
            x => Err(Error::FromLuaConversionError {
                from: x.type_name(),
                to: "Vec".to_string(),
                message: None,
            }),
        }
    }
}

impl<
        K: Eq + std::hash::Hash + FromLuaExact,
        V: FromLuaExact,
        S: std::hash::BuildHasher + Default,
    > FromLuaExact for HashMap<K, V, S>
{
    fn from_lua_exact(value: Value, lua: &Lua) -> mlua::Result<Self> {
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
                to: "HashMap".to_string(),
                message: Some("expected table".to_string()),
            })
        }
    }
}

impl<K: Ord + FromLuaExact, V: FromLuaExact> FromLuaExact for BTreeMap<K, V> {
    fn from_lua_exact(value: Value, lua: &Lua) -> mlua::Result<Self> {
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
                to: "BTreeMap".to_string(),
                message: Some("expected table".to_string()),
            })
        }
    }
}

impl<T: FromLuaExact, const N: usize> FromLuaExact for [T; N] {
    fn from_lua_exact(value: Value, lua: &Lua) -> mlua::Result<Self> {
        let as_vec = Vec::<T>::from_lua_exact(value, lua).map_err(|x| match x {
            Error::FromLuaConversionError {
                from,
                to: _,
                message: _,
            } => Error::FromLuaConversionError {
                from,
                to: "Array".to_string(),
                message: Some(format!("Expected array of exactly length {}", N)),
            },
            x => x,
        })?;
        let len = as_vec.len();
        if len != N {
            return Err(Error::FromLuaConversionError {
                from: "Table",
                to: "Array".to_string(),
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

impl_from_exact_non_failing!(bool, mlua::Value::Boolean(x), x);

impl_from_exact_non_failing!(Function, mlua::Value::Function(x), x);

impl_from_exact_non_failing!(Table, mlua::Value::Table(x), x);

impl_from_exact_non_failing!(mlua::String, mlua::Value::String(x), x);

impl_from_exact!(i8, mlua::Value::Integer(x), x, TryFromIntError);
impl_from_exact!(u8, mlua::Value::Integer(x), x, TryFromIntError);
impl_from_exact!(i16, mlua::Value::Integer(x), x, TryFromIntError);
impl_from_exact!(u16, mlua::Value::Integer(x), x, TryFromIntError);

#[cfg(any(target_pointer_width = "32", feature = "mlua_luau"))]
impl_from_exact_non_failing!(i32, mlua::Value::Integer(x), x);
#[cfg(all(target_pointer_width = "64", not(feature = "mlua_luau")))]
impl_from_exact!(i32, mlua::Value::Integer(x), x, TryFromIntError);

impl_from_exact!(u32, mlua::Value::Integer(x), x, TryFromIntError);
impl_from_exact_non_failing!(i64, mlua::Value::Integer(x), x);
impl_from_exact!(u64, mlua::Value::Integer(x), x, TryFromIntError);
impl_from_exact_non_failing!(i128, mlua::Value::Integer(x), x);
impl_from_exact!(u128, mlua::Value::Integer(x), x, TryFromIntError);
impl_from_exact!(isize, mlua::Value::Integer(x), x, TryFromIntError);
impl_from_exact!(usize, mlua::Value::Integer(x), x, TryFromIntError);

impl_from_exact_non_failing!(f64, mlua::Value::Number(x), x);

impl FromLuaExact for f32 {
    fn from_lua_exact(value: Value, lua: &Lua) -> mlua::Result<Self> {
        f64::from_lua_exact(value, lua).map(|x| x as f32)
    }
}
