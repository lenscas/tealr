use std::borrow::Cow;

use crate::{type_representation::KindOfType, ToTypename, Type};

///Represents a type
#[derive(Debug, PartialEq, Eq, Clone, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(
    feature = "derive",
    derive(crate::mlu::FromToLua, crate::ToTypename)
)]
#[cfg_attr(
    all(feature = "derive"),
    tealr(tealr_name = crate)
)]
pub struct TealType {
    ///Name of the type
    #[cfg_attr(
        all(feature = "derive"),
        tealr(remote =  String))]
    pub name: Cow<'static, str>,
    ///If the type is build in, a generic or from a library
    pub type_kind: KindOfType,
    ///any generics that this type has
    pub generics: Option<Vec<TealType>>,
}

///A collection of TealValues.
///
///It is implemented by various tuples so they can be used
//as function/method parameters and their return types.
pub trait TealMultiValue {
    ///Gets the types contained in this collection.
    ///Order *IS* important.
    fn get_types() -> Vec<Type> {
        Self::get_types_as_params()
            .into_iter()
            .map(|v| v.ty)
            .collect()
    }
    ///Gets the type representations as used for function parameters
    fn get_types_as_params() -> Vec<crate::FunctionParam>;
}

macro_rules! impl_teal_multi_value {
    () => (
        impl TealMultiValue for () {
            fn get_types_as_params() -> Vec<crate::FunctionParam> {
                Vec::new()
            }
        }
    );

    ($($names:ident) +) => (
        impl<$($names,)* > TealMultiValue for ($($names,)*)
            where $($names: ToTypename,)*
        {
            #[allow(unused_mut)]
            #[allow(non_snake_case)]
            fn get_types_as_params() -> Vec<crate::FunctionParam> {
                let mut params = Vec::new();
                $(params.extend($names::to_function_param(),);)*
                params

            }
        }
    );
}

impl<A> TealMultiValue for A
where
    A: ToTypename,
{
    #[allow(unused_mut)]
    #[allow(non_snake_case)]
    fn get_types_as_params() -> Vec<crate::FunctionParam> {
        A::to_function_param()
    }
}

impl_teal_multi_value!();
impl_teal_multi_value!(A);
impl_teal_multi_value!(A B);
impl_teal_multi_value!(A B C);
impl_teal_multi_value!(A B C D);
impl_teal_multi_value!(A B C D E);
impl_teal_multi_value!(A B C D E F);
impl_teal_multi_value!(A B C D E F G);
impl_teal_multi_value!(A B C D E F G H);
impl_teal_multi_value!(A B C D E F G H I);
impl_teal_multi_value!(A B C D E F G H I J);
impl_teal_multi_value!(A B C D E F G H I J K);
impl_teal_multi_value!(A B C D E F G H I J K L);
impl_teal_multi_value!(A B C D E F G H I J K L M);
impl_teal_multi_value!(A B C D E F G H I J K L M N);
impl_teal_multi_value!(A B C D E F G H I J K L M N O);
impl_teal_multi_value!(A B C D E F G H I J K L M N O P);
