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
        $visibility enum $type_name {
            Nil,
            Boolean(bool),
            LightUserData($crate::mlu::mlua::LightUserData),
            Integer($crate::mlu::mlua::Integer),
            Number($crate::mlu::mlua::Number),
            String($crate::mlu::mlua::String),
            Table($crate::mlu::mlua::Table),
            Function($crate::mlu::mlua::Function),
            Thread($crate::mlu::mlua::Thread),
            UserData($crate::mlu::mlua::AnyUserData),
            Error(Box<$crate::mlu::mlua::Error>),
            #[cfg(feature = "mlua_luau")]
            Vector(f32,f32,f32)
        }
        impl $crate::mlu::mlua::FromLua for $type_name {
            fn from_lua(value: $crate::mlu::mlua::Value, _: &$crate::mlu::mlua::Lua) -> ::std::result::Result<Self, $crate::mlu::mlua::Error> {
                Ok(value.into())
            }
        }
        impl $crate::mlu::mlua::IntoLua for $type_name {
            fn into_lua(self, _: &$crate::mlu::mlua::Lua) -> ::std::result::Result<$crate::mlu::mlua::Value, $crate::mlu::mlua::Error> {
                Ok(self.into())
            }
        }
        impl From<$crate::mlu::mlua::Value> for $type_name {
            fn from(value:$crate::mlu::mlua::Value) -> $type_name {
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
                    #[cfg(feature = "mlua_luau")]
                    Vector(vec) => $type_name::Vector(vec.x(),vec.y(),vec.z()),
                    _ => unimplemented!("Unsupported variant"),
                }
            }
        }
        impl From<$type_name> for $crate::mlu::mlua::Value {
            fn from(value:$type_name) -> $crate::mlu::mlua::Value {
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
                    #[cfg(feature = "mlua_luau")]
                    Vector(x,y,z) => $crate::mlu::mlua::Value::Vector($crate::mlu::mlua::Vector::new(x,y,z))
                }
            }
        }
        impl ::std::iter::FromIterator<$type_name> for $crate::mlu::mlua::MultiValue {
            fn from_iter<__MacroIterGeneric: IntoIterator<Item = $type_name>>(iter: __MacroIterGeneric) -> Self {
                iter.into_iter().map($crate::mlu::mlua::Value::from).collect()
            }
        }
        impl ::core::cmp::PartialEq<$crate::mlu::mlua::Value> for $type_name {
            fn eq(&self, other: &$crate::mlu::mlua::Value) -> bool {
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
                    #[cfg(feature = "mlua_luau")]
                    ($type_name::Vector(x,y,z), $crate::mlu::mlua::Value::Vector(vec)) => *x == vec.x() && *y == vec.y() && *z == vec.z(),
                    _ => false,
                }
            }
        }
        impl ::core::cmp::PartialEq for $type_name {
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
                    #[cfg(feature = "mlua_luau")]
                    ($type_name::Vector(x,y,z), $type_name::Vector(x2,y2,z2)) => x == x2 && y == y2 && z == z2,
                    _ => false,
                }
            }
        }
        impl $crate::ToTypename for $type_name {
            fn to_typename() -> $crate::Type {
                $crate::Type::new_single(stringify!($type_name), $crate::KindOfType::Generic)
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
create_generic_mlua!(pub I);
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
