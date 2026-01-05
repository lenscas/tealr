/// This macro creates a new type that acts as similar as possible to [mlua::Value](mlua::Value)
/// however, it acts as a generic type marker instead of `any`.
///
/// This makes it easy to expose a generic function, method and even type to teal.
///
///##Example
///```
/// let lua = tealr::mlu::mlua::Lua::new();
/// tealr::create_generic_mlua!(pub YourPublicType);
/// tealr::create_generic_mlua!(YourPrivateType);
/// //x will be exposed to lua as `function<YourPublicType>(YourPublicType): YourPublicType`
/// let x = tealr::mlu::TypedFunction::<YourPublicType, YourPublicType>::from_rust(|_, x| Ok(x), &lua);
///
///```
#[macro_export]
macro_rules! create_generic_mlua {
    ($visibility:vis $type_name:ident) => {
        #[derive(Clone,Debug)]
        #[allow(missing_docs)]
        $visibility struct $type_name ($crate::mlu::mlua::Value);
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
                $type_name(value)
            }
        }
        impl From<$type_name> for $crate::mlu::mlua::Value {
            fn from(value:$type_name) -> $crate::mlu::mlua::Value {
                value.0
            }
        }
        impl ::std::iter::FromIterator<$type_name> for $crate::mlu::mlua::MultiValue {
            fn from_iter<__MacroIterGeneric: IntoIterator<Item = $type_name>>(iter: __MacroIterGeneric) -> Self {
                iter.into_iter().map($crate::mlu::mlua::Value::from).collect()
            }
        }
        impl ::core::cmp::PartialEq<$crate::mlu::mlua::Value> for $type_name {
            fn eq(&self, other: &$crate::mlu::mlua::Value) -> bool {
                <$crate::mlu::mlua::Value as ::core::cmp::PartialEq<$crate::mlu::mlua::Value>>::eq(&self.0, &other)
            }
        }
        impl ::core::cmp::PartialEq for $type_name {
            fn eq(&self, other: &Self) -> bool {
                <$crate::mlu::mlua::Value as ::core::cmp::PartialEq<$crate::mlu::mlua::Value>>::eq(&self.0, &other.0)
            }
        }
        impl $crate::ToTypename for $type_name {
            fn to_typename() -> $crate::Type {
                $crate::Type::new_single(stringify!($type_name), $crate::KindOfType::Generic)
            }
        }
        impl $type_name {
            ///compares the 2 values, taking the __eq meta method into account if it is set.
            pub fn equals(&self, other: &$type_name) -> $crate::mlu::mlua::Result<bool> {
                $crate::mlu::mlua::Value::equals(
                    &self.0,
                    &other.0
                )
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
