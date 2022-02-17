use std::borrow::Cow;

use crate::{type_representation::KindOfType, Direction, NamePart, TypeName};

///Represents a type
#[derive(Debug, PartialEq, Eq, Clone, Hash, serde::Serialize, serde::Deserialize)]
pub struct TealType {
    ///Name of the type
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
    fn get_types(dir: Direction) -> Vec<NamePart>;
}

macro_rules! impl_teal_multi_value {
    () => (
        impl TealMultiValue for () {
            fn get_types(_:Direction) -> Vec<NamePart> {
                Vec::new()
            }
        }
    );

    ($($names:ident) +) => (
        impl<$($names,)* > TealMultiValue for ($($names,)*)
            where $($names: TypeName,)*
        {
            #[allow(unused_mut)]
            #[allow(non_snake_case)]
            fn get_types(dir:Direction) ->  Vec<NamePart>{
                let x:Vec<Cow<'static,[$crate::NamePart]>> = vec![
                    $($names::get_type_parts(dir),)*
                ];
                let x = itertools::Itertools::intersperse(
                    x.into_iter(),
                    Cow::Borrowed(
                        &[
                            $crate::NamePart::Symbol(
                                Cow::Borrowed("),(")
                            )
                        ]
                    )
                ).flat_map(
                    |v|
                        v.into_iter()
                        .map(|v|v.to_owned())
                        .collect::<Vec<_>>()
                        .into_iter()
                );

                std::iter::once(
                    $crate::NamePart::Symbol(
                        Cow::Borrowed("(")
                    )
                )
                .chain(x)
                .chain(
                    std::iter::once(
                        $crate::NamePart::Symbol(
                            Cow::Borrowed(")")
                        )
                    )
                ).collect()
            }
        }
    );
}

impl<A> TealMultiValue for A
where
    A: TypeName,
{
    #[allow(unused_mut)]
    #[allow(non_snake_case)]
    fn get_types(dir: Direction) -> Vec<NamePart> {
        A::get_type_parts(dir).to_vec()
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
