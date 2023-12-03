/// Creates a type that allows you to give names to the positional parameters.
/// The names only show up in the documentation and definition files. Making them great to add just a bit more of documentation in the function signature itself
///
/// Syntax is `create_named_parameters!(YourTypeName with first_field_name : TypeFirstField, second_field_name : TypeSecondField,);`
/// ## Example
/// ```
/// tealr::mlua_create_named_parameters!(
///     Example with
///     field_1 : String,
///     field_2 : i64,
/// );
/// let lua = tealr::mlu::mlua::Lua::new();
/// let example_func = tealr::mlu::TypedFunction::from_rust(|_, example: Example| {
///     Ok((example.field_1,example.field_2))
/// },&lua)?;
/// lua.globals().set("example_func", example_func)?;
/// //Lua still calls the method as normal
/// let (param1,param2) : (String,i64) = lua.load("return example_func(\"hello, named parameters\", 2)").eval()?;
///
/// assert_eq!(param1,"hello, named parameters".to_string());
/// assert_eq!(param2, 2);
///
/// # Result::<_, tealr::mlu::mlua::Error>::Ok(())
/// ```
#[macro_export]
macro_rules! mlua_create_named_parameters {
    ($type_name:ident with $($field_name:ident : $field_type_name:ty, )*) => {
        pub struct $type_name {
            $(pub $field_name : $field_type_name,)*
        }
        impl $crate::ToTypename for $type_name {
            fn to_typename() -> $crate::Type {
                let mut x = Vec::new();
                $(
                    x.push(<$field_type_name as $crate::ToTypename>::to_typename());
                )*
                std::convert::From::from($crate::Type::Tuple(x))
            }
            fn to_function_param() -> Vec<$crate::FunctionParam> {
                let mut x = Vec::new();
                $(
                    x.push($crate::FunctionParam {
                        param_name: Some(stringify!($field_name).into()),
                        ty: <$field_type_name as $crate::ToTypename>::to_typename()
                    });
                )*
                x
            }
        }
        impl<'lua> $crate::mlu::mlua::FromLuaMulti<'lua> for $type_name {
            fn from_lua_multi(
                mut values: $crate::mlu::mlua::MultiValue<'lua>,
                lua: &'lua $crate::mlu::mlua::Lua,
            ) -> $crate::mlu::mlua::Result<Self> {
                Ok(Self {
                    $($field_name: <_ as $crate::mlu::mlua::FromLua>::from_lua(
                        values
                            .pop_front()
                            .unwrap_or_else(|| $crate::mlu::mlua::Value::Nil),
                        lua,
                    )?,)*
                })
            }
        }
    };
}
