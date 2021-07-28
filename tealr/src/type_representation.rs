///The direction that the rust <-> lua conversion goes to.
///This is needed in case the FromLua and ToLua traits aren't perfect opposites of each other.

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    ///Represents that the Rust type is being exposed to Lua
    FromLua,
    ///Represents that the Rust type is being constructed from Lua
    ToLua,
}

macro_rules! impl_type_name {
    ($teal_type:literal $current_type:ty) => {
        impl TypeName for $current_type {
            fn get_type_name(_ :Direction) -> Cow<'static, str> {
                Cow::from($teal_type)
            }
            fn is_external() -> bool {
                false
            }
        }
    };
    ($teal_type:literal $current_type:ty,$($types:ty),*) => {
        impl_type_name!($teal_type $current_type);
        impl_type_name!($teal_type $($types),+);
    };
}

///A trait to collect the required type information like the name of the type.
pub trait TypeName {
    ///returns the type name as how it should show up in the generated `.d.tl` file
    fn get_type_name(dir: Direction) -> Cow<'static, str>;
    ///This method tells the generator that the type will always be available
    ///This is pretty much only the case for native lua/teal types like `number`
    ///
    ///***DO NOT*** overwrite it unless you are ***ABSOLUTELY*** sure you need to.
    ///The default is correct for 99.999% of the cases.
    fn is_external() -> bool {
        true
    }
}

use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
};

use crate::TypeGenerator;

impl_type_name!("boolean" bool);
impl_type_name!("string" String,std::ffi::CString,bstr::BString ,&str,&std::ffi::CStr,&bstr::BStr);
impl_type_name!("number" f32,f64);
impl_type_name!("integer" i8,u8,u16,i16,u32,i32,u64,i64,u128,i128,isize,usize);

#[cfg(feature = "rlua")]
impl<'lua> TypeName for rlua::Thread<'lua> {
    fn get_type_name(_: Direction) -> Cow<'static, str> {
        Cow::from("thread")
    }
    fn is_external() -> bool {
        false
    }
}

#[cfg(feature = "mlua")]
impl<'lua> TypeName for mlua::Thread<'lua> {
    fn get_type_name(_: Direction) -> Cow<'static, str> {
        Cow::from("thread")
    }
    fn is_external() -> bool {
        false
    }
}

#[cfg(feature = "mlua_async")]
impl<'lua, R> TypeName for mlua::AsyncThread<'lua, R> {
    fn get_type_name(_: Direction) -> Cow<'static, str> {
        Cow::from("thread")
    }
    fn is_external() -> bool {
        false
    }
}

#[cfg(feature = "rlua")]
impl<'lua> TypeName for rlua::Value<'lua> {
    fn get_type_name(_: Direction) -> Cow<'static, str> {
        Cow::from("any")
    }
    fn is_external() -> bool {
        false
    }
}

#[cfg(feature = "mlua")]
impl<'lua> TypeName for mlua::Value<'lua> {
    fn get_type_name(_: Direction) -> Cow<'static, str> {
        Cow::from("any")
    }
    fn is_external() -> bool {
        false
    }
}

#[cfg(feature = "rlua")]
impl<'lua> TypeName for rlua::Table<'lua> {
    fn get_type_name(_: Direction) -> Cow<'static, str> {
        Cow::from("{any:any}")
    }
    fn is_external() -> bool {
        false
    }
}

#[cfg(feature = "mlua")]
impl<'lua> TypeName for mlua::Table<'lua> {
    fn get_type_name(_: Direction) -> Cow<'static, str> {
        Cow::from("{any:any}")
    }
    fn is_external() -> bool {
        false
    }
}

#[cfg(feature = "rlua")]
impl<'lua> TypeName for rlua::String<'lua> {
    fn get_type_name(_: Direction) -> Cow<'static, str> {
        Cow::from("string")
    }
    fn is_external() -> bool {
        false
    }
}

#[cfg(feature = "mlua")]
impl<'lua> TypeName for mlua::String<'lua> {
    fn get_type_name(_: Direction) -> Cow<'static, str> {
        Cow::from("string")
    }
    fn is_external() -> bool {
        false
    }
}

#[cfg(feature = "rlua")]
impl<'lua> TypeName for rlua::Function<'lua> {
    fn get_type_name(_: Direction) -> Cow<'static, str> {
        Cow::from("function(...:any):any...")
    }
    fn is_external() -> bool {
        false
    }
}

#[cfg(feature = "mlua")]
impl<'lua> TypeName for mlua::Function<'lua> {
    fn get_type_name(_: Direction) -> Cow<'static, str> {
        Cow::from("function(...:any):any...")
    }
    fn is_external() -> bool {
        false
    }
}

impl<T: TypeName> TypeName for Vec<T> {
    fn get_type_name(d: Direction) -> Cow<'static, str> {
        Cow::from(format!("{{{}}}", T::get_type_name(d)))
    }
    fn is_external() -> bool {
        false
    }
}

impl<T: TypeName> TypeName for Option<T> {
    fn get_type_name(d: Direction) -> Cow<'static, str> {
        T::get_type_name(d)
    }
    fn is_external() -> bool {
        false
    }
}

impl<K: TypeName, V: TypeName> TypeName for HashMap<K, V> {
    fn get_type_name(d: Direction) -> Cow<'static, str> {
        Cow::from(format!(
            "{{{}:{}}}",
            K::get_type_name(d),
            V::get_type_name(d)
        ))
    }
    fn is_external() -> bool {
        false
    }
}
impl<K: TypeName, V: TypeName> TypeName for BTreeMap<K, V> {
    fn get_type_name(d: Direction) -> Cow<'static, str> {
        Cow::from(format!(
            "{{{}:{}}}",
            K::get_type_name(d),
            V::get_type_name(d)
        ))
    }
    fn is_external() -> bool {
        false
    }
}
///Creates the body of the type, so the functions and fields it exposes.
pub trait TypeBody {
    ///Fills in the TypeGenerator so a .d.tl file can be constructed.
    fn get_type_body(dir: Direction, gen: &mut TypeGenerator);
}
