use std::{
    collections::{BTreeMap, HashMap},
    ffi::{CStr, CString},
    num::TryFromIntError,
};

use mlua::{Error, Function, Lua, Table, Value};

/// similar to [mlua::FromLua](mlua::FromLua). However,
/// however going through this trait you promise that the conversion to a rust value prefers failing over converting/casting
pub trait FromLuaExact<'lua>: Sized {
    ///Does the conversion, without any type conversion/casting
    fn from_lua_exact(value: mlua::Value<'lua>, lua: &'lua mlua::Lua) -> mlua::Result<Self>;
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
        impl<'lua> $crate::mlu::mlua::ToLua<'lua> for $type_name {
            fn to_lua(self, lua: &'lua $crate::mlu::mlua::Lua) -> ::std::result::Result<$crate::mlu::mlua::Value<'lua>, $crate::mlu::mlua::Error> {
                match self {
                    $($type_name::$sub_types(x) => x.to_lua(lua),)*
                }
            }
        }
        impl<'lua> $crate::mlu::mlua::FromLua<'lua> for $type_name {
            fn from_lua(value: $crate::mlu::mlua::Value<'lua>, lua: &'lua $crate::mlu::mlua::Lua) -> ::std::result::Result<Self, $crate::mlu::mlua::Error> {
                $(match <$sub_types as $crate::mlu::FromLuaExact>::from_lua_exact(value.clone(),lua) {
                    Ok(x) => return Ok($type_name::$sub_types(x)),
                    Err($crate::mlu::mlua::Error::FromLuaConversionError{from:_,to:_,message:_}) => {}
                    Err(x) => return Err(x)
                };)*
                Err($crate::mlu::mlua::Error::FromLuaConversionError{
                    to: stringify!( $($sub_types)|* ),
                    from: value.type_name(),
                    message: None
                })
            }
        }
        impl $crate::TypeName for $type_name {
            fn get_type_parts() -> ::std::borrow::Cow<'static,[$crate::NamePart]> {
                let mut name = Vec::new();
                $(
                    name.append(&mut $sub_types::get_type_parts().to_vec());
                    name.push(" | ".into());
                )*
                name.pop();
                std::borrow::Cow::Owned(name)
            }
            fn collect_children(v: &mut Vec<$crate::TealType>) {
                use $crate::TealMultiValue;
                $(
                    v.extend(
                        ($sub_types::get_types()
                        .into_iter()
                        ).filter_map(|v| {
                            if let $crate::NamePart::Type(x) = v {
                                Some(x)
                            } else {
                                None
                            }
                        })
                    );
                )*
            }
            fn get_type_kind() -> $crate::KindOfType {
                $crate::KindOfType::Builtin
            }
        }
    };
}

impl<'lua> FromLuaExact<'lua> for String {
    fn from_lua_exact(value: mlua::Value<'lua>, _: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::String(x) => Ok(x.to_str()?.to_owned()),
            x => Err(mlua::Error::FromLuaConversionError {
                from: x.type_name(),
                to: "String",
                message: None,
            }),
        }
    }
}

impl<'lua> FromLuaExact<'lua> for CString {
    fn from_lua_exact(value: mlua::Value<'lua>, lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        let ty = value.type_name();
        let as_str = mlua::String::from_lua_exact(value, lua)?;
        match CStr::from_bytes_with_nul(as_str.as_bytes_with_nul()) {
            Ok(x) => Ok(x.to_owned()),
            Err(_) => Err(mlua::Error::FromLuaConversionError {
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
            fn from_lua_exact(value: mlua::Value<'lua>, _: &'lua mlua::Lua) -> mlua::Result<Self> {
                match value {
                    $conv => Ok($bound_on.into()),
                    x => Err(mlua::Error::FromLuaConversionError {
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
            fn from_lua_exact(value: mlua::Value<'lua>, _: &'lua mlua::Lua) -> mlua::Result<Self> {
                let ty = value.type_name();
                match value {
                    $conv => $bound_on.try_into().map_err(|x: $error| {
                        mlua::Error::FromLuaConversionError {
                            from: ty,
                            to: stringify!($T),
                            message: Some(x.to_string()),
                        }
                    }),
                    _ => Err(mlua::Error::FromLuaConversionError {
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
    fn from_lua_exact(value: mlua::Value<'lua>, lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::Nil => Ok(None),
            x => T::from_lua_exact(x, lua).map(Some),
        }
    }
}

impl<'lua, T: FromLuaExact<'lua>> FromLuaExact<'lua> for Vec<T> {
    fn from_lua_exact(value: mlua::Value<'lua>, lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::Table(x) => x
                .sequence_values()
                .map(|x| match x {
                    Ok(x) => T::from_lua_exact(x, lua),
                    Err(x) => Err(x),
                })
                .collect(),
            x => Err(mlua::Error::FromLuaConversionError {
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
    fn from_lua_exact(value: mlua::Value<'lua>, lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        if let mlua::Value::Table(table) = value {
            table
                .pairs()
                .map(|x| match x {
                    Ok((key, value)) => K::from_lua_exact(key, lua)
                        .and_then(|key| V::from_lua_exact(value, lua).map(|value| (key, value))),
                    Err(x) => Err(x),
                })
                .collect()
        } else {
            Err(mlua::Error::FromLuaConversionError {
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
    fn from_lua_exact(value: Value<'lua>, lua: &'lua Lua) -> mlua::Result<Self> {
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
    fn from_lua_exact(value: Value<'lua>, lua: &'lua mlua::Lua) -> mlua::Result<Self> {
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

impl_from_exact_non_failing!(bool, mlua::Value::Boolean(x), x);

impl_from_exact_non_failing!(Function<'lua>, mlua::Value::Function(x), x);

impl_from_exact_non_failing!(Table<'lua>, mlua::Value::Table(x), x);

impl_from_exact_non_failing!(mlua::String<'lua>, mlua::Value::String(x), x);

impl_from_exact!(i8, mlua::Value::Integer(x), x, TryFromIntError);
impl_from_exact!(u8, mlua::Value::Integer(x), x, TryFromIntError);
impl_from_exact!(i16, mlua::Value::Integer(x), x, TryFromIntError);
impl_from_exact!(u16, mlua::Value::Integer(x), x, TryFromIntError);
impl_from_exact!(i32, mlua::Value::Integer(x), x, TryFromIntError);
impl_from_exact!(u32, mlua::Value::Integer(x), x, TryFromIntError);
impl_from_exact_non_failing!(i64, mlua::Value::Integer(x), x);
impl_from_exact!(u64, mlua::Value::Integer(x), x, TryFromIntError);
impl_from_exact_non_failing!(i128, mlua::Value::Integer(x), x);
impl_from_exact!(u128, mlua::Value::Integer(x), x, TryFromIntError);
impl_from_exact!(isize, mlua::Value::Integer(x), x, TryFromIntError);
impl_from_exact!(usize, mlua::Value::Integer(x), x, TryFromIntError);

impl_from_exact_non_failing!(f64, mlua::Value::Number(x), x);

impl<'lua> FromLuaExact<'lua> for f32 {
    fn from_lua_exact(value: mlua::Value<'lua>, lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        f64::from_lua_exact(value, lua).map(|x| x as f32)
    }
}
