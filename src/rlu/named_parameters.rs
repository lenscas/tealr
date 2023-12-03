/// Creates a type that allows you to give names to the positional parameters.
/// The names only show up in the documentation and definition files. Making them great to add just a bit more of documentation in the function signature itself
///
/// Syntax is `create_named_parameters!(YourTypeName with first_field_name : TypeFirstField, second_field_name : TypeSecondField,);`
/// ## Example
/// ```
/// tealr::rlua_create_named_parameters!(
///     Example with
///     field_1 : String,
///     field_2 : i64,
/// );
/// tealr::rlu::rlua::Lua::new()
///     .context(|ctx| {
///         let example_func = tealr::rlu::TypedFunction::from_rust(|_, example: Example| {
///             Ok((example.field_1,example.field_2))
///         },ctx)?;
///         ctx.globals().set("example_func", example_func)?;
///         //Lua still calls the method as normal
///         let (param1,param2) : (String,i64) = ctx.load("return example_func(\"hello, named parameters\", 2)").eval()?;
///         assert_eq!(param1,"hello, named parameters".to_string());
///         assert_eq!(param2, 2);
///         Ok(())
///     })?;
/// # Result::<_, tealr::rlu::rlua::Error>::Ok(())
/// ```
#[macro_export]
macro_rules! rlua_create_named_parameters {
    ($type_name:ident with $($field_name:ident : $field_type_name:ty, )*) => {
        pub struct $type_name {
            $(pub $field_name : $field_type_name,)*
        }
        impl $crate::ToTypename for $type_name {
            #[allow(clippy::vec_init_then_push)]
            fn to_typename() -> $crate::Type {
                let mut x = Vec::new();
                $(
                    x.push(<$field_type_name as $crate::ToTypename>::to_typename());
                )*
                std::convert::From::from($crate::Type::Tuple(x))
            }
            #[allow(clippy::vec_init_then_push)]
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
        impl<'lua> $crate::rlu::rlua::FromLuaMulti<'lua> for $type_name {
            fn from_lua_multi(
                values: $crate::rlu::rlua::MultiValue<'lua>,
                lua: $crate::rlu::rlua::Context<'lua>,
            ) -> rlua::Result<Self> {
                let mut as_vec = values.into_vec().into_iter();
                Ok(Self {
                    $($field_name: <_ as $crate::rlu::rlua::FromLua>::from_lua(
                        as_vec
                            .next()
                            .unwrap_or_else(|| $crate::rlu::rlua::Value::Nil),
                        lua,
                    )?,)*
                })
            }
        }
    };
}

rlua_create_named_parameters!(
    TestStruct with
    field_1 : String,
    field_2 : i64,
);
