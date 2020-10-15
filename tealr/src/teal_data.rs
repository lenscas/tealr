use crate::teal_data_methods::TealDataMethods;

///This is the teal version of [UserData](rlua::UserData).
pub trait TealData: Sized {
    ///same as [UserData::add_methods](rlua::UserData::add_methods).
    ///Refer to its documentation on how to use it.
    ///
    ///only diffrence is that it takes a [TealDataMethods](crate::TealDataMethods),
    ///which is the teal version of [UserDataMethods](rlua::UserDataMethods)
    fn add_methods<'lua, T: TealDataMethods<'lua, Self>>(_methods: &mut T) {}

    ///returns the type name as how it should show up in the generated `.d.tl` file
    fn get_type_name() -> String;
    ///this method tells the generator whether a type is from `lua`/`teal`
    ///or is always available in some other way
    ///
    ///***DO NOT*** overwrite it unless you are ***ABSOLUTLY*** sure you need to.
    ///The default is correct for 99.999% of the cases.
    fn is_external() -> bool {
        true
    }
}

macro_rules! impl_teal_data {
    ($teal_type:literal $current_type:ty) => {
        impl TealData for $current_type {
            fn get_type_name() ->String {
                String::from($teal_type)
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

use std::collections::{BTreeMap, HashMap};

impl_teal_data!("boolean" bool);
impl_teal_data!("string" String,std::ffi::CString,bstr::BString,&str,&std::ffi::CStr,&bstr::BStr);
impl_teal_data!("number" i8,u8,u16,i16,u32,i32,u64,i64,u128,i128,isize,usize,f32,f64);

impl<T: TealData> TealData for Vec<T> {
    fn get_type_name() -> String {
        format!("{{{}}}", T::get_type_name())
    }
    fn is_external() -> bool {
        false
    }
}

impl<T: TealData> TealData for Option<T> {
    fn get_type_name() -> String {
        T::get_type_name()
    }
    fn is_external() -> bool {
        false
    }
}

impl<K: TealData, V: TealData> TealData for HashMap<K, V> {
    fn get_type_name() -> String {
        format!("{{{}:{}}}", K::get_type_name(), V::get_type_name())
    }
    fn is_external() -> bool {
        false
    }
}
impl<K: TealData, V: TealData> TealData for BTreeMap<K, V> {
    fn get_type_name() -> String {
        format!("{{{}:{}}}", K::get_type_name(), V::get_type_name())
    }
    fn is_external() -> bool {
        false
    }
}
