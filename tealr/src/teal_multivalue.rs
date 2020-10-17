use std::borrow::Cow;

use crate::teal_data::TypeRepresentation;

///Represents a type
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TealType {
    pub(crate) name: Cow<'static, str>,
    pub(crate) is_external: bool,
}
impl TealType {
    fn new(name: Cow<'static, str>, is_external: bool) -> Self {
        Self { name, is_external }
    }
    ///generates a [TealType](crate::TealType) based on a type implementing [TealData](crate::TealData).
    ///```
    ///# use tealr::TealType;
    ///let nummeric_i8 = TealType::from::<i8>();
    ///let nummeric_float = TealType::from::<f32>();
    /// //both i8 and f32 become a "number" in lua/teal. As such, their TealTypes are equal.
    ///assert_eq!(nummeric_i8,nummeric_float)
    ///```
    pub fn from<A: TypeRepresentation>() -> Self {
        Self::new(A::get_type_name(), A::is_external())
    }
}

///A collection of TealValues.
///
///It is implemented by various tuples so they can be used
//as function/method parameters and their return types.
pub trait TealMultiValue {
    ///Gets the types contained in this collection.
    ///Order *IS* important.
    fn get_types() -> Vec<TealType>;
}

macro_rules! impl_teal_multi_value {
    () => (
        impl TealMultiValue for () {
            fn get_types() -> Vec<TealType> {
                vec![]
            }
        }
    );

    ($($names:ident) +) => (
        impl<$($names,)* > TealMultiValue for ($($names,)*)
            where $($names: TypeRepresentation,)*
        {
            #[allow(unused_mut)]
            #[allow(non_snake_case)]
            fn get_types() ->  Vec<TealType>{
                let mut types = Vec::new();
                $(types.push(TealType::from::<$names>());)*
                types
            }
        }
    );
}

impl<A> TealMultiValue for A
where
    A: TypeRepresentation,
{
    #[allow(unused_mut)]
    #[allow(non_snake_case)]
    fn get_types() -> Vec<TealType> {
        vec![TealType::from::<A>()]
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
