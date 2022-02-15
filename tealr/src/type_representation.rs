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

macro_rules! impl_type_name_life_time {
    ($teal_type:literal $current_type:ty) => {
        impl<'lua> TypeName for $current_type {
            fn get_type_parts(_: Direction) -> Cow<'static, [NamePart]> {
                Cow::Borrowed(&[NamePart::Type(TealType {
                    name: Cow::Borrowed($teal_type),
                    type_kind: KindOfType::Builtin,
                    generics: None,
                })])
            }
            fn get_type_kind() -> KindOfType {
                KindOfType::Builtin
            }
        }
    };
}

macro_rules! impl_type_name {
    ($teal_type:literal $current_type:ty) => {
        impl TypeName for $current_type {
            fn get_type_parts(_ :Direction) -> Cow<'static, [NamePart]> {
                Cow::Borrowed(&[NamePart::Type(TealType {
                        name: Cow::Borrowed($teal_type),
                        type_kind: KindOfType::Builtin,
                        generics: None,
                    }
                )])
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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
#[macro_export]
///An easy way to implement TypeName::get_type_parts if it only needs to return a single, external type without generics.
macro_rules! new_type {
    ($type_name:ident,BuiltIn) => {
        ::std::borrow::Cow::Borrowed(&[$crate::NamePart::Type($crate::TealType {
            name: ::std::borrow::Cow::Borrowed(stringify!($type_name)),
            type_kind: $crate::KindOfType::Builtin,
            generics: None,
        })])
    };
    ($type_name:ident,External) => {
        ::std::borrow::Cow::Borrowed(&[$crate::NamePart::Type($crate::TealType {
            name: ::std::borrow::Cow::Borrowed(stringify!($type_name)),
            type_kind: $crate::KindOfType::External,
            generics: None,
        })])
    };

    ($type_name:ident) => {
        new_type!($type_name, External)
    };
    ($type_name:ident,Generic) => {
        ::std::borrow::Cow::Borrowed(&[$crate::NamePart::Type($crate::TealType {
            name: ::std::borrow::Cow::Borrowed(stringify!($type_name)),
            type_kind: $crate::KindOfType::Generic,
            generics: None,
        })])
    };
}
#[derive(Clone, serde::Serialize, serde::Deserialize)]
///The parts that a name consists of
pub enum NamePart {
    ///A piece of normal text that is part of the type.
    ///An example could be the `function(` part inside `function(integer):string`
    Symbol(Cow<'static, str>),
    ///A piece of the type that is actually a full type.
    ///An example could be the part `integer` part inside of `function(integer):string`
    Type(TealType),
    //Appended(Cow<'static, [NamePart]>),
}

impl NamePart {
    ///Turn a NamePart into a `Cow<'static, str>`
    pub fn as_ref_str(&self) -> &Cow<'static, str> {
        match self {
            NamePart::Symbol(x) => x,
            NamePart::Type(x) => &x.name,
        }
    }
    ///checks if &self is of the `Symbol(_)` variant
    pub fn is_symbol(&self) -> bool {
        matches!(&self, NamePart::Symbol(_))
    }
}

impl From<String> for NamePart {
    fn from(x: String) -> Self {
        NamePart::Symbol(Cow::Owned(x))
    }
}

impl From<&'static str> for NamePart {
    fn from(x: &'static str) -> Self {
        NamePart::Symbol(Cow::Borrowed(x))
    }
}

impl From<NamePart> for Cow<'static, str> {
    fn from(x: NamePart) -> Self {
        match x {
            NamePart::Symbol(x) => x,
            NamePart::Type(x) => x.name,
        }
    }
}
///Used to turn an entire type (`Cow<'static, [NamePart]>`) into a string representing this type
pub fn type_parts_to_str(x: Cow<'static, [NamePart]>) -> Cow<'static, str> {
    if x.len() == 1 {
        let el = match x {
            Cow::Borrowed(x) => x.to_vec(),
            Cow::Owned(x) => x,
        }
        .pop()
        .unwrap();
        match el {
            NamePart::Symbol(x) => x,
            NamePart::Type(x) => x.name,
        }
    } else if x.is_empty() {
        Cow::Borrowed("")
    } else {
        Cow::Owned(
            x.iter()
                .map(|v| v.as_ref_str())
                .map(|v| v.to_owned())
                .collect::<String>(),
        )
    }
}

///A trait to collect the required type information like the name of the type.
pub trait TypeName {
    ///returns the type name as how it should show up in the generated `.d.tl` file
    fn get_type_parts(dir: Direction) -> Cow<'static, [NamePart]>;
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
    fn get_type_parts(_: Direction) -> Cow<'static, [NamePart]> {
        crate::new_type!(thread, BuiltIn)
    }
    fn get_type_kind() -> KindOfType {
        KindOfType::Builtin
    }
}

#[cfg(feature = "mlua")]
impl_type_name_life_time!("thread" mlua::Thread<'lua>);
// impl<'lua> TypeName for mlua::Thread<'lua> {
//     fn get_type_name(_: Direction) -> Cow<'static, str> {}
//     fn get_type_kind() -> KindOfType {
//         KindOfType::Builtin
//     }
// }

#[cfg(feature = "mlua_async")]
impl<'lua, R> TypeName for mlua::AsyncThread<'lua, R> {
    fn get_type_parts(_: Direction) -> Cow<'static, [NamePart]> {
        new_type!(thread, BuiltIn)
    }
    fn get_type_kind() -> KindOfType {
        KindOfType::Builtin
    }
}

#[cfg(feature = "rlua")]
impl_type_name_life_time!("any" rlua::Value<'lua>);

#[cfg(feature = "mlua")]
impl_type_name_life_time!("any" mlua::Value<'lua>);

fn get_type_parts_table() -> Cow<'static, [NamePart]> {
    Cow::Borrowed(&[
        NamePart::Symbol(Cow::Borrowed("{")),
        NamePart::Type(TealType {
            name: Cow::Borrowed("any"),
            type_kind: KindOfType::Builtin,
            generics: None,
        }),
        NamePart::Symbol(Cow::Borrowed(" : ")),
        NamePart::Type(TealType {
            name: Cow::Borrowed("any"),
            type_kind: KindOfType::Builtin,
            generics: None,
        }),
        NamePart::Symbol(Cow::Borrowed(" : ")),
    ])
}

#[cfg(feature = "rlua")]
impl<'lua> TypeName for rlua::Table<'lua> {
    fn get_type_parts(_: Direction) -> Cow<'static, [NamePart]> {
        get_type_parts_table()
    }
    fn get_type_kind() -> KindOfType {
        KindOfType::Builtin
    }
}

#[cfg(feature = "mlua")]
impl<'lua> TypeName for mlua::Table<'lua> {
    fn get_type_kind() -> KindOfType {
        KindOfType::Builtin
    }

    fn get_type_parts(_: Direction) -> Cow<'static, [NamePart]> {
        get_type_parts_table()
    }
}

#[cfg(feature = "rlua")]
impl<'lua> TypeName for rlua::String<'lua> {
    fn get_type_parts(_: Direction) -> Cow<'static, [NamePart]> {
        crate::new_type!(string, BuiltIn)
    }
    fn get_type_kind() -> KindOfType {
        KindOfType::Builtin
    }
}

fn get_pars_any_func() -> Cow<'static, [NamePart]> {
    Cow::Borrowed(&[
        NamePart::Symbol(Cow::Borrowed("function(...")),
        NamePart::Type(TealType {
            name: Cow::Borrowed("any"),
            type_kind: KindOfType::Builtin,
            generics: None,
        }),
        NamePart::Symbol(Cow::Borrowed("):")),
        NamePart::Type(TealType {
            name: Cow::Borrowed("any"),
            type_kind: KindOfType::Builtin,
            generics: None,
        }),
        NamePart::Symbol(Cow::Borrowed("...")),
    ])
}

#[cfg(feature = "mlua")]
impl_type_name_life_time!("string" mlua::String<'lua>);

#[cfg(feature = "rlua")]
impl<'lua> TypeName for rlua::Function<'lua> {
    fn get_type_parts(_: Direction) -> Cow<'static, [NamePart]> {
        get_pars_any_func()
    }
    fn get_type_kind() -> KindOfType {
        KindOfType::Builtin
    }
}

#[cfg(feature = "mlua")]
impl<'lua> TypeName for mlua::Function<'lua> {
    fn get_type_kind() -> KindOfType {
        KindOfType::Builtin
    }

    fn get_type_parts(_: Direction) -> Cow<'static, [NamePart]> {
        get_pars_any_func()
    }
}

impl<T: TypeName> TypeName for Vec<T> {
    fn get_type_kind() -> KindOfType {
        KindOfType::Builtin
    }
    fn collect_children(child: &mut Vec<TealType>) {
        child.extend(T::get_types(crate::Direction::FromLua));
        child.extend(T::get_types(crate::Direction::ToLua));
    }
    fn get_type_parts(dir: Direction) -> Cow<'static, [NamePart]> {
        let mut v = vec!["{".into()];
        v.append(&mut T::get_type_parts(dir).to_vec());
        v.push("}".into());
        Cow::Owned(v)
    }
}

impl<T: TypeName> TypeName for Option<T> {
    fn get_type_kind() -> KindOfType {
        KindOfType::Builtin
    }
    fn collect_children(child: &mut Vec<TealType>) {
        child.extend(T::get_types(crate::Direction::FromLua));
        child.extend(T::get_types(crate::Direction::ToLua));
    }

    fn get_type_parts(dir: Direction) -> Cow<'static, [NamePart]> {
        T::get_type_parts(dir)
    }
}

impl<K: TypeName, V: TypeName> TypeName for HashMap<K, V> {
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

    fn get_type_parts(dir: Direction) -> Cow<'static, [NamePart]> {
        let mut key_parts = K::get_type_parts(dir).to_vec();
        let mut value_parts = V::get_type_parts(dir).to_vec();
        let mut type_def = vec!["{".into()];
        type_def.append(&mut key_parts);
        type_def.push(":".into());
        type_def.append(&mut value_parts);
        type_def.push("}".into());
        Cow::from(type_def)
    }
}
impl<K: TypeName, V: TypeName> TypeName for BTreeMap<K, V> {
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
    fn get_type_parts(dir: Direction) -> Cow<'static, [NamePart]> {
        let mut key_parts = K::get_type_parts(dir).to_vec();
        let mut value_parts = V::get_type_parts(dir).to_vec();
        let mut type_def = vec!["{".into()];
        type_def.append(&mut key_parts);
        type_def.push(":".into());
        type_def.append(&mut value_parts);
        type_def.push("}".into());
        Cow::from(type_def)
    }
}
///Creates the body of the type, so the functions and fields it exposes.
pub trait TypeBody {
    ///Fills in the TypeGenerator so a .d.tl file can be constructed.
    fn get_type_body(dir: Direction, gen: &mut TypeGenerator);
}
