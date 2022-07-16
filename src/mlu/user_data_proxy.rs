use std::marker::PhantomData;

use mlua::{AnyUserData, Error, Lua, ToLua, UserData};

use crate::{TypeBody, TypeName, RecordGenerator, EnumGenerator};

/// A userdata which can be used as a static proxy
pub trait StaticUserdata: UserData + 'static {}
impl<T: UserData + 'static> StaticUserdata for T {}

/// A newtype storing userdata created via [`mlua::Lua::create_proxy`].
///
/// if `T` implements TypeName or TypeBody the implementations are forwarded to this type.
pub struct UserDataProxy<'lua, T: StaticUserdata> {
    user_data: AnyUserData<'lua>,
    ph_: PhantomData<T>,
}

impl<'lua, T: StaticUserdata> UserDataProxy<'lua, T> {
    /// Creates a new UserDataProxy
    pub fn new(lua: &'lua Lua) -> Result<Self, Error> {
        Ok(Self {
            user_data: lua.create_proxy::<T>()?,
            ph_: Default::default(),
        })
    }
}

impl<T: StaticUserdata + TypeName> TypeName for UserDataProxy<'_, T> {
    fn get_type_parts() -> std::borrow::Cow<'static, [crate::NamePart]> {
        let mut base = T::get_type_parts().to_vec();
        let suffix = crate::NamePart::Symbol("Class".into());
        base.push(suffix);
        std::borrow::Cow::Owned(base)
    }
}

impl<T: StaticUserdata + TypeBody + TypeName> TypeBody for UserDataProxy<'_, T> {
    fn get_type_body() -> crate::TypeGenerator {
        let generator = T::get_type_body();
        // extract only "functions"
        match generator {
            crate::TypeGenerator::Record(record_generator) => {
                crate::TypeGenerator::Record(
                    Box::new(
                        RecordGenerator{
                            // TODO: deal with this to reflect this is the "Class" version 
                            type_name: Self::get_type_parts(),
                            // documentation: todo!(),
                            // type_doc: todo!(),

                            // we overwrite anything which is not static
                            fields: Default::default(),
                            methods: Default::default(),
                            mut_methods: Default::default(),
                            meta_method: Default::default(),
                            meta_method_mut: Default::default(),
                            ..record_generator.as_ref().clone()
                        }
                    )
                )
            },
            crate::TypeGenerator::Enum(enum_generator) => {
                crate::TypeGenerator::Enum(
                    EnumGenerator{
                        name: Self::get_type_parts(),
                        ..enum_generator.clone()
                    }
                )
            },
        }
    }
}

impl<'lua, T: StaticUserdata> ToLua<'lua> for UserDataProxy<'lua, T> {
    fn to_lua(self, lua: &'lua Lua) -> mlua::Result<mlua::Value<'lua>> {
        self.user_data.to_lua(lua)
    }
}
