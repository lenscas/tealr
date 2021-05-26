use std::{borrow::Cow, string::FromUtf8Error};

use crate::TealType;

#[cfg(any(feature = "rlua", feature = "mlua"))]
use crate::{Direction, TealMultiValue};

#[cfg(feature = "rlua")]
use rlua::{FromLuaMulti as FromLuaMultiR, ToLuaMulti as ToLuaMultiR};

#[cfg(feature = "mlua")]
use mlua::{FromLuaMulti as FromLuaMultiM, ToLuaMulti as ToLuaMultiM};

///Contains the data needed to write down the type of a function
pub struct ExportedFunction {
    pub(crate) name: Vec<u8>,
    pub(crate) params: Vec<TealType>,
    pub(crate) returns: Vec<TealType>,
    pub(crate) is_meta_method: bool,
}
impl ExportedFunction {
    ///Creates an ExportedFunction with the given name, Parameters and return value
    ///```no_run
    ///# use tealr::ExportedFunction;
    ///# use std::borrow::Cow;
    ///ExportedFunction::new_rlua::<(String,String),String>(Cow::from("concat"),false);
    ///```
    #[cfg(feature = "rlua")]
    pub fn new_rlua<
        'lua,
        Params: ToLuaMultiR<'lua> + TealMultiValue,
        Response: FromLuaMultiR<'lua> + TealMultiValue,
    >(
        name: Cow<'static, str>,
        is_meta_method: bool,
    ) -> Self {
        Self {
            name: name.as_bytes().to_owned(),
            params: Params::get_types(Direction::FromLua),
            returns: Response::get_types(Direction::ToLua),
            is_meta_method,
        }
    }
    ///Creates an ExportedFunction with the given name, Parameters and return value
    ///```no_run
    ///# use tealr::ExportedFunction;
    ///# use std::borrow::Cow;
    ///ExportedFunction::new_mlua::<(String,String),String>(Cow::from("concat"),false);
    ///```
    #[cfg(feature = "mlua")]
    pub fn new_mlua<
        'lua,
        Params: ToLuaMultiM<'lua> + TealMultiValue,
        Response: FromLuaMultiM<'lua> + TealMultiValue,
    >(
        name: Cow<'static, str>,
        is_meta_method: bool,
    ) -> Self {
        Self {
            name: name.as_bytes().to_owned(),
            params: Params::get_types(Direction::FromLua),
            returns: Response::get_types(Direction::ToLua),
            is_meta_method,
        }
    }
    pub(crate) fn generate(
        self,
        self_type: Option<Cow<'static, str>>,
    ) -> std::result::Result<String, FromUtf8Error> {
        let params = self_type
            .iter()
            .map(|v| v.to_owned())
            .chain(self.params.iter().map(|v| v.name.to_owned()))
            .collect::<Vec<_>>()
            .join(", ");

        let returns = self
            .returns
            .iter()
            .map(|v| v.name.to_owned())
            .collect::<Vec<_>>()
            .join(", ");

        Ok(format!(
            "{}{}: function({}):({})",
            if self.is_meta_method {
                "metamethod "
            } else {
                ""
            },
            String::from_utf8(self.name)?,
            params,
            returns
        ))
    }
}
