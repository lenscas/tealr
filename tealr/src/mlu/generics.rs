///This macro creates a new type that acts as similar as possible to [mlua::Value](mlua::Value)
///however, it acts as a generic type instead of being translated as `any`.
///
///This makes it easy to expose a generic function/method to teal.
///##Example
///```
///tealr::create_generic_mlua!(pub YourPublicType);
///tealr::create_generic_mlua!(YourPrivateType);
///```
#[macro_export]
macro_rules! create_generic_mlua {
    ($visibility:vis $type_name:ident) => {
        #[derive(Clone,Debug)]
        #[allow(missing_docs)]
        $visibility enum $type_name<'lua> {
            Nil,
            Boolean(bool),
            LightUserData($crate::mlu::mlua::LightUserData),
            Integer($crate::mlu::mlua::Integer),
            Number($crate::mlu::mlua::Number),
            String($crate::mlu::mlua::String<'lua>),
            Table($crate::mlu::mlua::Table<'lua>),
            Function($crate::mlu::mlua::Function<'lua>),
            Thread($crate::mlu::mlua::Thread<'lua>),
            UserData($crate::mlu::mlua::AnyUserData<'lua>),
            Error($crate::mlu::mlua::Error),
        }
        impl<'lua> $crate::mlu::mlua::FromLua<'lua> for $type_name<'lua> {
            fn from_lua(value: $crate::mlu::mlua::Value<'lua>, _: &'lua $crate::mlu::mlua::Lua) -> ::std::result::Result<Self, $crate::mlu::mlua::Error> {
                Ok(value.into())
            }
        }
        impl<'lua> $crate::mlu::mlua::ToLua<'lua> for $type_name<'lua> {
            fn to_lua(self, _: &'lua $crate::mlu::mlua::Lua) -> ::std::result::Result<$crate::mlu::mlua::Value<'lua>, $crate::mlu::mlua::Error> {
                Ok(self.into())
            }
        }
        impl<'lua> From<$crate::mlu::mlua::Value<'lua>> for $type_name<'lua> {
            fn from(value:$crate::mlu::mlua::Value<'lua>) -> $type_name {
                use $crate::mlu::mlua::Value::*;
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
        impl<'lua> From<$type_name<'lua>> for $crate::mlu::mlua::Value<'lua> {
            fn from(value:$type_name<'lua>) -> $crate::mlu::mlua::Value<'lua> {
                use $type_name::*;
                match value {
                    Nil => $crate::mlu::mlua::Value::Nil,
                    Boolean(x) => $crate::mlu::mlua::Value::Boolean(x),
                    LightUserData(x) => $crate::mlu::mlua::Value::LightUserData(x),
                    Integer(x) => $crate::mlu::mlua::Value::Integer(x),
                    Number(x) => $crate::mlu::mlua::Value::Number(x),
                    String(x) => $crate::mlu::mlua::Value::String(x),
                    Table(x) => $crate::mlu::mlua::Value::Table(x),
                    Function(x) => $crate::mlu::mlua::Value::Function(x),
                    Thread(x) => $crate::mlu::mlua::Value::Thread(x),
                    UserData(x) => $crate::mlu::mlua::Value::UserData(x),
                    Error(x) => $crate::mlu::mlua::Value::Error(x),
                }
            }
        }
        impl<'lua> ::std::iter::FromIterator<$type_name<'lua>> for $crate::mlu::mlua::MultiValue<'lua> {
            fn from_iter<I: IntoIterator<Item = $type_name<'lua>>>(iter: I) -> Self {
                iter.into_iter().map($crate::mlu::mlua::Value::from).collect()
            }
        }
        impl<'lua> ::core::cmp::PartialEq<$crate::mlu::mlua::Value<'lua>> for $type_name<'lua> {
            fn eq(&self, other: &$crate::mlu::mlua::Value<'lua>) -> bool {
                match (self, other) {
                    ($type_name::Nil, $crate::mlu::mlua::Value::Nil) => true,
                    ($type_name::Boolean(a), $crate::mlu::mlua::Value::Boolean(b)) => a == b,
                    ($type_name::LightUserData(a), $crate::mlu::mlua::Value::LightUserData(b)) => a == b,
                    ($type_name::Integer(a), $crate::mlu::mlua::Value::Integer(b)) => *a == *b,
                    ($type_name::Integer(a), $crate::mlu::mlua::Value::Number(b)) => *a as $crate::mlu::mlua::Number == *b,
                    ($type_name::Number(a), $crate::mlu::mlua::Value::Integer(b)) => *a == *b as $crate::mlu::mlua::Number,
                    ($type_name::Number(a), $crate::mlu::mlua::Value::Number(b)) => *a == *b,
                    ($type_name::String(a), $crate::mlu::mlua::Value::String(b)) => a == b,
                    ($type_name::Table(a), $crate::mlu::mlua::Value::Table(b)) => a == b,
                    ($type_name::Function(a), $crate::mlu::mlua::Value::Function(b)) => a == b,
                    ($type_name::Thread(a), $crate::mlu::mlua::Value::Thread(b)) => a == b,
                    ($type_name::UserData(a), $crate::mlu::mlua::Value::UserData(b)) => a == b,
                    _ => false,
                }
            }
        }
        impl<'lua> ::core::cmp::PartialEq for $type_name<'lua> {
            fn eq(&self, other: &Self) -> bool {
                match (self, other) {
                    ($type_name::Nil, $type_name::Nil) => true,
                    ($type_name::Boolean(a), $type_name::Boolean(b)) => a == b,
                    ($type_name::LightUserData(a), $type_name::LightUserData(b)) => a == b,
                    ($type_name::Integer(a), $type_name::Integer(b)) => *a == *b,
                    ($type_name::Integer(a), $type_name::Number(b)) => *a as $crate::mlu::mlua::Number == *b,
                    ($type_name::Number(a), $type_name::Integer(b)) => *a == *b as $crate::mlu::mlua::Number,
                    ($type_name::Number(a), $type_name::Number(b)) => *a == *b,
                    ($type_name::String(a), $type_name::String(b)) => a == b,
                    ($type_name::Table(a), $type_name::Table(b)) => a == b,
                    ($type_name::Function(a), $type_name::Function(b)) => a == b,
                    ($type_name::Thread(a), $type_name::Thread(b)) => a == b,
                    ($type_name::UserData(a), $type_name::UserData(b)) => a == b,
                    _ => false,
                }
            }
        }
        impl<'lua> $crate::TypeName for $type_name<'lua> {
            fn get_type_parts(_: $crate::Direction) -> std::borrow::Cow<'static, [$crate::NamePart]> {
                let x:&'static [$crate::NamePart] =&[
                    $crate::NamePart::Type($crate::TealType{
                        name: ::std::borrow::Cow::Borrowed(stringify!($type_name)),
                        type_kind: $crate::KindOfType::Generic,
                        generics: :: std::option::Option::None
                    })
                ];
                ::std::borrow::Cow::Borrowed(x)

            }
            fn get_type_kind() -> $crate::KindOfType {
                $crate::KindOfType::Generic
            }
        }
    };
}
create_generic_mlua!(pub A);
create_generic_mlua!(pub B);
create_generic_mlua!(pub C);
create_generic_mlua!(pub D);
create_generic_mlua!(pub E);
create_generic_mlua!(pub F);
create_generic_mlua!(pub G);
create_generic_mlua!(pub H);
create_generic_mlua!(pub J);
create_generic_mlua!(pub K);
create_generic_mlua!(pub L);
create_generic_mlua!(pub M);
create_generic_mlua!(pub N);
create_generic_mlua!(pub O);
create_generic_mlua!(pub P);
create_generic_mlua!(pub Q);
create_generic_mlua!(pub R);
create_generic_mlua!(pub S);
create_generic_mlua!(pub T);
create_generic_mlua!(pub U);
create_generic_mlua!(pub V);
create_generic_mlua!(pub W);
create_generic_mlua!(pub X);
create_generic_mlua!(pub Y);
create_generic_mlua!(pub Z);
