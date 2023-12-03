use crate::{FunctionParam, MapRepresentation, SingleType, ToTypename, Type};
macro_rules! impl_type_name_life_time {
    ($teal_type:literal $current_type:ty) => {
        impl<'lua> ToTypename for $current_type {
            fn to_typename() -> Type {
                Type::Single(SingleType {
                    name: $teal_type.into(),
                    kind: KindOfType::Builtin,
                })
            }
        }
    };
}

macro_rules! impl_type_name {
    ($teal_type:literal $current_type:ty) => {
        impl ToTypename for $current_type {
            fn to_typename() -> Type {
                Type::Single(SingleType {
                    name: $teal_type.into(),
                    kind: KindOfType::Builtin,
                })
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
#[cfg_attr(
    all(feature = "mlua", feature = "derive", not(feature = "rlua")),
    derive(crate::mlu::FromToLua, crate::ToTypename)
)]
#[cfg_attr(
    all(feature = "rlua", feature = "derive", not(feature = "mlua")),
    derive(crate::rlu::FromToLua, crate::ToTypename)
)]
#[cfg_attr(
    all(any(feature = "rlua", feature = "mlua"), feature = "derive", not(all(feature = "rlua", feature = "mlua")) ),
    tealr(tealr_name = crate)
)]
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
///An easy way to implement [TypeName::get_type_parts](crate::ToTypename#tymethod.get_type_parts) if it only needs to return a single type without generics.
/// ```rust
/// # use std::borrow::Cow;
/// # use tealr::TealType;
/// let name =  tealr::new_type!(Example, External);
/// assert_eq!(name,Cow::Borrowed(&[tealr::NamePart::Type(tealr::TealType{
///     name: Cow::Borrowed("Example"),
///     type_kind: tealr::KindOfType::External,
///     generics:None
/// })]))
///```
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
#[derive(Debug, Clone, PartialEq, Hash, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(
    all(feature = "mlua", feature = "derive", not(feature = "rlua")),
    derive(crate::mlu::FromToLua, crate::ToTypename)
)]
#[cfg_attr(
    all(feature = "rlua", feature = "derive", not(feature = "mlua")),
    derive(crate::rlu::FromToLua, crate::ToTypename)
)]
#[cfg_attr(
    all(any(feature = "rlua", feature = "mlua"), feature = "derive", not(all(feature = "rlua", feature = "mlua"))),
    tealr(tealr_name = crate)
)]
///The parts that a name consists of
pub enum NamePart {
    ///A piece of normal text that is part of the type.
    ///An example could be the `function(` part inside `function(integer):string`
    Symbol(
        #[cfg_attr(
        all(any(feature = "rlua", feature = "mlua"), feature = "derive",not(all(feature = "rlua", feature = "mlua"))),
        tealr(remote =  String))]
        Cow<'static, str>,
    ),
    ///A piece of the type that is actually a full type.
    ///An example could be the part `integer` part inside of `function(integer):string`
    Type(TealType),
    //Appended(Cow<'static, [NamePart]>),
}

impl NamePart {
    /// an easier way to create a [NamePart::Symbol], which does the Cow wrapping for you.
    pub fn symbol(symbol: impl Into<Cow<'static, str>>) -> Self {
        Self::Symbol(symbol.into())
    }
}

impl Display for NamePart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref_str())
    }
}

impl NamePart {
    ///Turn a NamePart into a `Cow<'static, str>`
    pub fn as_ref_str(&self) -> &Cow<'static, str> {
        match self {
            NamePart::Symbol(x) => x,
            NamePart::Type(x) => &x.name,
        }
    }
    ///checks if `&self` is of the `Symbol(_)` variant
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
                .cloned()
                .collect::<String>(),
        )
    }
}

///A trait to collect the required type information like the name of the type.
pub trait TypeName {
    ///returns the type name as how it should show up in the generated `.d.tl` file
    fn get_type_parts() -> Cow<'static, [NamePart]>;
    /// Generates the typename when used to describe a global value.
    ///
    /// Sometimes (for example in the case of lambda's with types marked as generic)
    /// you want an altered representation of the type if it is used as a global instance or is part of something else.
    ///
    /// You almost never want this though so you should probably think twice before altering the implementation.
    fn get_type_parts_as_global() -> Cow<'static, [NamePart]> {
        Self::get_type_parts()
    }
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
    fmt::Display,
};

use crate::{TealType, TypeGenerator};

impl_type_name!("boolean" bool);
impl_type_name!("string" String,std::ffi::CString,bstr::BString ,&str,&std::ffi::CStr,&bstr::BStr);
impl_type_name!("number" f32,f64);
impl_type_name!("integer" i8,u8,u16,i16,u32,i32,u64,i64,u128,i128,isize,usize);

#[cfg(feature = "rlua")]
impl_type_name_life_time!("thread" rlua::Thread<'lua>);

#[cfg(feature = "mlua")]
impl_type_name_life_time!("thread" mlua::Thread<'lua>);

#[cfg(feature = "mlua_async")]
impl<'lua, R> ToTypename for mlua::AsyncThread<'lua, R> {
    fn to_typename() -> Type {
        Type::new_single("thread", KindOfType::Builtin)
    }
}

#[cfg(feature = "rlua")]
impl_type_name_life_time!("any" rlua::Value<'lua>);

#[cfg(feature = "mlua")]
impl_type_name_life_time!("any" mlua::Value<'lua>);

#[cfg(feature = "rlua")]
use rlua::{Table, Value};

#[cfg(feature = "mlua")]
use mlua::{Table, Value};

impl<'lua> ToTypename for Table<'lua> {
    fn to_typename() -> Type {
        Type::Map(crate::MapRepresentation {
            key: Value::to_typename().into(),
            value: Value::to_typename().into(),
        })
    }
}

#[cfg(feature = "rlua")]
impl_type_name_life_time!("string" rlua::String<'lua>);

#[cfg(feature = "mlua")]
impl_type_name_life_time!("string" mlua::String<'lua>);

#[cfg(feature = "mlua")]
use mlua::Function;
#[cfg(feature = "rlua")]
use rlua::Function;

impl<'lua> ToTypename for Function<'lua> {
    fn to_typename() -> Type {
        Type::Function(crate::FunctionRepresentation {
            params: vec![FunctionParam {
                param_name: Some("...".into()),
                ty: Type::new_single("any", KindOfType::Builtin),
            }],
            returns: vec![Type::new_single("any...", KindOfType::Builtin)],
        })
    }
}

impl<T: ToTypename> ToTypename for Vec<T> {
    fn to_typename() -> Type {
        Type::Array(T::to_typename().into())
    }
}

impl<T: ToTypename, const N: usize> ToTypename for [T; N] {
    fn to_typename() -> Type {
        Vec::<T>::to_typename()
    }
}

impl<T: ToTypename> ToTypename for Option<T> {
    fn to_typename() -> Type {
        T::to_typename()
    }
}

impl<K: ToTypename, V: ToTypename> ToTypename for HashMap<K, V> {
    fn to_typename() -> Type {
        Type::Map(crate::MapRepresentation {
            key: K::to_typename().into(),
            value: V::to_typename().into(),
        })
    }
}

impl<K: ToTypename, V: ToTypename> ToTypename for BTreeMap<K, V> {
    fn to_typename() -> Type {
        Type::Map(MapRepresentation {
            key: K::to_typename().into(),
            value: V::to_typename().into(),
        })
    }
}
///Creates the body of the type, so the functions and fields it exposes.
pub trait TypeBody {
    ///Fills in the TypeGenerator so a .d.tl file can be constructed.
    fn get_type_body() -> TypeGenerator;
}
