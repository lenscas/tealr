use crate::teal_data_methods::TealDataMethods;

///This is the teal version of [UserData](rlua::UserData).
pub trait TealData: Sized {
    ///same as [UserData::add_methods](rlua::UserData::add_methods).
    ///Refer to its documentation on how to use it.
    ///
    ///only diffrence is that it takes a [TealDataMethods](crate::TealDataMethods),
    ///which is the teal version of [UserDataMethods](rlua::UserDataMethods)
    fn add_methods<'lua, T: TealDataMethods<'lua, Self>>(_methods: &mut T) {}
}

macro_rules! impl_teal_data {
    ($teal_type:literal $current_type:ty) => {
        impl TypeRepresentation for $current_type {
            fn get_type_name() -> Cow<'static, str> {
                Cow::from($teal_type)
            }
            fn is_external() -> bool {
                false
            }
        }
    };
    ($teal_type:literal $current_type:ty,$($types:ty),*) => {
        impl_teal_data!($teal_type $current_type);
        impl_teal_data!($teal_type $($types),+);
    };
}

///A trait to collect the required type information like the name of the type.
pub trait TypeRepresentation {
    ///returns the type name as how it should show up in the generated `.d.tl` file
    fn get_type_name() -> Cow<'static, str>;
    ///This method tells the generator that the type will always be available
    ///This is pretty much only the case for native lua/teal types like `number`
    ///
    ///***DO NOT*** overwrite it unless you are ***ABSOLUTLY*** sure you need to.
    ///The default is correct for 99.999% of the cases.
    fn is_external() -> bool {
        true
    }
}

use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
};

impl_teal_data!("boolean" bool);
impl_teal_data!("string" String,std::ffi::CString,bstr::BString,&str,&std::ffi::CStr,&bstr::BStr);
impl_teal_data!("number" i8,u8,u16,i16,u32,i32,u64,i64,u128,i128,isize,usize,f32,f64);

impl<T: TypeRepresentation> TypeRepresentation for Vec<T> {
    fn get_type_name() -> Cow<'static, str> {
        Cow::from(format!("{{{}}}", T::get_type_name()))
    }
    fn is_external() -> bool {
        false
    }
}

impl<T: TypeRepresentation> TypeRepresentation for Option<T> {
    fn get_type_name() -> Cow<'static, str> {
        T::get_type_name()
    }
    fn is_external() -> bool {
        false
    }
}

impl<K: TypeRepresentation, V: TypeRepresentation> TypeRepresentation for HashMap<K, V> {
    fn get_type_name() -> Cow<'static, str> {
        Cow::from(format!("{{{}:{}}}", K::get_type_name(), V::get_type_name()))
    }
    fn is_external() -> bool {
        false
    }
}
impl<K: TypeRepresentation, V: TypeRepresentation> TypeRepresentation for BTreeMap<K, V> {
    fn get_type_name() -> Cow<'static, str> {
        Cow::from(format!("{{{}:{}}}", K::get_type_name(), V::get_type_name()))
    }
    fn is_external() -> bool {
        false
    }
}
