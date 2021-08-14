use crate::teal_multivalue::TealMultiValue;
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
            fn get_type_kind() -> KindOfType {
                KindOfType::Builtin
            }
        }
    };
    ($teal_type:literal $current_type:ty,$($types:ty),*) => {
        impl_type_name!($teal_type $current_type);
        impl_type_name!($teal_type $($types),+);
    };
}

///Keeps track of any special treatment a type needs to get
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum KindOfType {
    ///The type is build in to teal.
    ///
    ///Never do anything special in this case.
    Builtin,
    ///The type come from a library (including this one).
    ///
    ///In the future it might be possible that tealr generates the correct `require` statements in this case
    External,
    ///The type represent a generic type parameter.
    ///
    ///When used it turns the method/function into a generic method/function.
    Generic,
}
impl KindOfType {
    ///```
    ///# use tealr::KindOfType;
    ///assert!(KindOfType::Generic.is_generic());
    ///```
    pub fn is_generic(&self) -> bool {
        self == &Self::Generic
    }
    ///```
    ///# use tealr::KindOfType;
    ///assert!(KindOfType::Builtin.is_builtin());
    ///```
    pub fn is_builtin(&self) -> bool {
        self == &Self::Builtin
    }
    ///```
    ///# use tealr::KindOfType;
    ///assert!(KindOfType::External.is_external());
    ///```
    pub fn is_external(&self) -> bool {
        self == &Self::External
    }
}
impl Default for KindOfType {
    fn default() -> Self {
        Self::External
    }
}
///A trait to collect the required type information like the name of the type.
pub trait TypeName {
    ///returns the type name as how it should show up in the generated `.d.tl` file
    fn get_type_name(dir: Direction) -> Cow<'static, str>;
    ///This method tells the generator if this type is builtin to teal/lua, if it comes from somewhere else or if it stands in as a generic
    ///
    ///In almost all cases you want to return `KindOfType::External`
    ///
    ///KindOfType::Generic` is only needed if the type itself is meant as a generic type placeholder.
    ///
    //KindOfType::Builtin should almost NEVER be returned
    fn get_type_kind() -> KindOfType {
        KindOfType::External
    }
    ///Creates/updates a list of every child type this type has
    ///This is used to properly label methods/functions as being generic.
    fn collect_children(_: &mut Vec<TealType>) {}
}

use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
};

use crate::{TealType, TypeGenerator};

impl_type_name!("boolean" bool);
impl_type_name!("string" String,std::ffi::CString,bstr::BString ,&str,&std::ffi::CStr,&bstr::BStr);
impl_type_name!("number" f32,f64);
impl_type_name!("integer" i8,u8,u16,i16,u32,i32,u64,i64,u128,i128,isize,usize);

#[cfg(feature = "rlua")]
impl<'lua> TypeName for rlua::Thread<'lua> {
    fn get_type_name(_: Direction) -> Cow<'static, str> {
        Cow::from("thread")
    }
    fn get_type_kind() -> KindOfType {
        KindOfType::Builtin
    }
}

#[cfg(feature = "mlua")]
impl<'lua> TypeName for mlua::Thread<'lua> {
    fn get_type_name(_: Direction) -> Cow<'static, str> {
        Cow::from("thread")
    }
    fn get_type_kind() -> KindOfType {
        KindOfType::Builtin
    }
}

#[cfg(feature = "mlua_async")]
impl<'lua, R> TypeName for mlua::AsyncThread<'lua, R> {
    fn get_type_name(_: Direction) -> Cow<'static, str> {
        Cow::from("thread")
    }
    fn get_type_kind() -> KindOfType {
        KindOfType::Builtin
    }
}

#[cfg(feature = "rlua")]
impl<'lua> TypeName for rlua::Value<'lua> {
    fn get_type_name(_: Direction) -> Cow<'static, str> {
        Cow::from("any")
    }
    fn get_type_kind() -> KindOfType {
        KindOfType::Builtin
    }
}

#[cfg(feature = "mlua")]
impl<'lua> TypeName for mlua::Value<'lua> {
    fn get_type_name(_: Direction) -> Cow<'static, str> {
        Cow::from("any")
    }
    fn get_type_kind() -> KindOfType {
        KindOfType::Builtin
    }
}

#[cfg(feature = "rlua")]
impl<'lua> TypeName for rlua::Table<'lua> {
    fn get_type_name(_: Direction) -> Cow<'static, str> {
        Cow::from("{any:any}")
    }
    fn get_type_kind() -> KindOfType {
        KindOfType::Builtin
    }
}

#[cfg(feature = "mlua")]
impl<'lua> TypeName for mlua::Table<'lua> {
    fn get_type_name(_: Direction) -> Cow<'static, str> {
        Cow::from("{any:any}")
    }
    fn get_type_kind() -> KindOfType {
        KindOfType::Builtin
    }
}

#[cfg(feature = "rlua")]
impl<'lua> TypeName for rlua::String<'lua> {
    fn get_type_name(_: Direction) -> Cow<'static, str> {
        Cow::from("string")
    }
    fn get_type_kind() -> KindOfType {
        KindOfType::Builtin
    }
}

#[cfg(feature = "mlua")]
impl<'lua> TypeName for mlua::String<'lua> {
    fn get_type_name(_: Direction) -> Cow<'static, str> {
        Cow::from("string")
    }
    fn get_type_kind() -> KindOfType {
        KindOfType::Builtin
    }
}

#[cfg(feature = "rlua")]
impl<'lua> TypeName for rlua::Function<'lua> {
    fn get_type_name(_: Direction) -> Cow<'static, str> {
        Cow::from("function(...:any):any...")
    }
    fn get_type_kind() -> KindOfType {
        KindOfType::Builtin
    }
}

#[cfg(feature = "mlua")]
impl<'lua> TypeName for mlua::Function<'lua> {
    fn get_type_name(_: Direction) -> Cow<'static, str> {
        Cow::from("function(...:any):any...")
    }
    fn get_type_kind() -> KindOfType {
        KindOfType::Builtin
    }
}

impl<T: TypeName> TypeName for Vec<T> {
    fn get_type_name(d: Direction) -> Cow<'static, str> {
        Cow::from(format!("{{{}}}", T::get_type_name(d)))
    }
    fn get_type_kind() -> KindOfType {
        KindOfType::Builtin
    }
    fn collect_children(child: &mut Vec<TealType>) {
        child.extend(T::get_types(crate::Direction::FromLua));
        child.extend(T::get_types(crate::Direction::ToLua));
    }
}

impl<T: TypeName> TypeName for Option<T> {
    fn get_type_name(d: Direction) -> Cow<'static, str> {
        T::get_type_name(d)
    }
    fn get_type_kind() -> KindOfType {
        KindOfType::Builtin
    }
    fn collect_children(child: &mut Vec<TealType>) {
        child.extend(T::get_types(crate::Direction::FromLua));
        child.extend(T::get_types(crate::Direction::ToLua));
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
    fn get_type_kind() -> KindOfType {
        KindOfType::Builtin
    }
    fn collect_children(child: &mut Vec<TealType>) {
        child.extend(
            K::get_types(crate::Direction::FromLua)
                .into_iter()
                .chain(K::get_types(crate::Direction::ToLua)),
        );
        child.extend(
            V::get_types(crate::Direction::FromLua)
                .into_iter()
                .chain(V::get_types(crate::Direction::ToLua)),
        );
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
    fn get_type_kind() -> KindOfType {
        KindOfType::Builtin
    }
    fn collect_children(child: &mut Vec<TealType>) {
        child.extend(
            K::get_types(crate::Direction::FromLua)
                .into_iter()
                .chain(K::get_types(crate::Direction::ToLua)),
        );
        child.extend(
            V::get_types(crate::Direction::FromLua)
                .into_iter()
                .chain(V::get_types(crate::Direction::ToLua)),
        );
    }
}
///Creates the body of the type, so the functions and fields it exposes.
pub trait TypeBody {
    ///Fills in the TypeGenerator so a .d.tl file can be constructed.
    fn get_type_body(dir: Direction, gen: &mut TypeGenerator);
}
