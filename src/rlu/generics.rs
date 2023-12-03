///This macro creates a new type that acts as similar as possible to [rlua::Value](rlua::Value)
///however, it acts as a generic type instead of being translated as `any`.
///
///This makes it easy to expose a generic function/method to teal.
///##Example
///```
///tealr::create_generic_rlua!(pub YourPublicType);
///tealr::create_generic_rlua!(YourPrivateType);
///```
#[macro_export]
macro_rules! create_generic_rlua {
    ($visibility:vis $type_name:ident) => {
        #[derive(Clone,Debug)]
        #[allow(missing_docs)]
        $visibility enum $type_name<'lua> {
            Nil,
            Boolean(bool),
            LightUserData($crate::rlu::rlua::LightUserData),
            Integer($crate::rlu::rlua::Integer),
            Number($crate::rlu::rlua::Number),
            String($crate::rlu::rlua::String<'lua>),
            Table($crate::rlu::rlua::Table<'lua>),
            Function($crate::rlu::rlua::Function<'lua>),
            Thread($crate::rlu::rlua::Thread<'lua>),
            UserData($crate::rlu::rlua::AnyUserData<'lua>),
            Error($crate::rlu::rlua::Error),
        }
        impl<'lua> $crate::rlu::rlua::FromLua<'lua> for $type_name<'lua> {
            fn from_lua(value: $crate::rlu::rlua::Value<'lua>, _:  $crate::rlu::rlua::Context<'lua>) -> ::std::result::Result<Self, $crate::rlu::rlua::Error> {
                Ok(value.into())
            }
        }
        impl<'lua> $crate::rlu::rlua::ToLua<'lua> for $type_name<'lua> {
            fn to_lua(self, _: $crate::rlu::rlua::Context<'lua>) -> ::std::result::Result<$crate::rlu::rlua::Value<'lua>, $crate::rlu::rlua::Error> {
                Ok(self.into())
            }
        }
        impl<'lua> From<$crate::rlu::rlua::Value<'lua>> for $type_name<'lua> {
            fn from(value:$crate::rlu::rlua::Value<'lua>) -> $type_name {
                use $crate::rlu::rlua::Value::*;
                match value {
                    Nil => $type_name::Nil,
                    Boolean(x) => $type_name::Boolean(x),
                    LightUserData(x) => $type_name::LightUserData(x),
                    Integer(x) => $type_name::Integer(x),
                    Number(x) => $type_name::Number(x),
                    String(x) => $type_name::String(x),
                    Table(x) => $type_name::Table(x),
                    Function(x) => $type_name::Function(x),
                    Thread(x) => $type_name::Thread(x),
                    UserData(x) => $type_name::UserData(x),
                    Error(x) => $type_name::Error(x),
                }
            }
        }
        impl<'lua> From<$type_name<'lua>> for $crate::rlu::rlua::Value<'lua> {
            fn from(value:$type_name<'lua>) -> $crate::rlu::rlua::Value<'lua> {
                use $type_name::*;
                match value {
                    Nil => $crate::rlu::rlua::Value::Nil,
                    Boolean(x) => $crate::rlu::rlua::Value::Boolean(x),
                    LightUserData(x) => $crate::rlu::rlua::Value::LightUserData(x),
                    Integer(x) => $crate::rlu::rlua::Value::Integer(x),
                    Number(x) => $crate::rlu::rlua::Value::Number(x),
                    String(x) => $crate::rlu::rlua::Value::String(x),
                    Table(x) => $crate::rlu::rlua::Value::Table(x),
                    Function(x) => $crate::rlu::rlua::Value::Function(x),
                    Thread(x) => $crate::rlu::rlua::Value::Thread(x),
                    UserData(x) => $crate::rlu::rlua::Value::UserData(x),
                    Error(x) => $crate::rlu::rlua::Value::Error(x),
                }
            }
        }
        impl<'lua> ::std::iter::FromIterator<$type_name<'lua>> for $crate::rlu::rlua::MultiValue<'lua> {
            fn from_iter<__MacroIterGeneric: IntoIterator<Item = $type_name<'lua>>>(iter: __MacroIterGeneric) -> Self {
                iter.into_iter().map($crate::rlu::rlua::Value::from).collect()
            }
        }
        impl<'lua> $crate::ToTypename for $type_name<'lua> {
            fn to_typename() -> $crate::Type {
                $crate::Type::new_single(stringify!($type_name), $crate::KindOfType::Generic)
            }
        }
    };
}
create_generic_rlua!(pub A);
create_generic_rlua!(pub B);
create_generic_rlua!(pub C);
create_generic_rlua!(pub D);
create_generic_rlua!(pub E);
create_generic_rlua!(pub F);
create_generic_rlua!(pub G);
create_generic_rlua!(pub H);
create_generic_rlua!(pub I);
create_generic_rlua!(pub J);
create_generic_rlua!(pub K);
create_generic_rlua!(pub L);
create_generic_rlua!(pub M);
create_generic_rlua!(pub N);
create_generic_rlua!(pub O);
create_generic_rlua!(pub P);
create_generic_rlua!(pub Q);
create_generic_rlua!(pub R);
create_generic_rlua!(pub S);
create_generic_rlua!(pub T);
create_generic_rlua!(pub U);
create_generic_rlua!(pub V);
create_generic_rlua!(pub W);
create_generic_rlua!(pub X);
create_generic_rlua!(pub Y);
create_generic_rlua!(pub Z);
