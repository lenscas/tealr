use std::borrow::Cow;

use crate::{type_representation::KindOfType, Direction, TypeName};

///Represents a type
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct TealType {
    pub(crate) name: Cow<'static, str>,
    pub(crate) type_kind: KindOfType,
    pub(crate) generics: Vec<TealType>,
}
impl TealType {
    fn new(name: Cow<'static, str>, type_kind: KindOfType, generics: Vec<TealType>) -> Self {
        Self {
            name,
            type_kind,
            generics,
        }
    }
    ///generates a [TealType](crate::TealType) based on a type implementing [TealData](crate::rlu::TealData).
    ///```
    ///# use tealr::{Direction,TealType};
    ///let numeric_i8 = TealType::from::<i8>(Direction::ToLua);
    ///let numeric_float = TealType::from::<i16>(Direction::ToLua);
    /// //both i8 and f32 become a "number" in lua/teal. As such, their TealTypes are equal.
    ///assert_eq!(numeric_i8,numeric_float)
    ///```
    pub fn from<A: TypeName>(dir: Direction) -> Self {
        let mut generics = Vec::new();
        A::collect_children(&mut generics);
        Self::new(A::get_type_name(dir), A::get_type_kind(), generics)
    }
}

///A collection of TealValues.
///
///It is implemented by various tuples so they can be used
//as function/method parameters and their return types.
pub trait TealMultiValue {
    ///Gets the types contained in this collection.
    ///Order *IS* important.
    fn get_types(dir: Direction) -> Vec<TealType>;
}

macro_rules! impl_teal_multi_value {
    () => (
        impl TealMultiValue for () {
            fn get_types(_:Direction) -> Vec<TealType> {
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
            fn get_types(dir:Direction) ->  Vec<TealType>{
                let types = vec![
                    $(TealType::from::<$names>(dir),)*
                ];
                // let mut types = Vec::new();
                // $(types.push(TealType::from::<$names>(dir));)*
                types
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
    fn get_types(dir: Direction) -> Vec<TealType> {
        vec![TealType::from::<A>(dir)]
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
