use mlua::IntoLua;

use crate::ToTypename;

impl<T: IntoLua + ToTypename> ToTypename for mlua::Variadic<T> {
    fn to_typename() -> crate::Type {
        crate::Type::Variadic(Box::new(T::to_typename()))
    }
    fn to_function_param() -> Vec<crate::FunctionParam> {
        vec![crate::FunctionParam {
            param_name: Some(crate::Name::from("...")),
            ty: Self::to_typename(),
        }]
    }
}
