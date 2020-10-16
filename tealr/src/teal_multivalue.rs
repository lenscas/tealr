use crate::teal_data::TealData;

///Represents a type
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TealType {
    pub(crate) name: String,
    pub(crate) is_external: bool,
}
impl TealType {
    fn new(name: String, is_external: bool) -> Self {
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
    pub fn from<A: TealData>() -> Self {
        Self::new(A::get_type_name(), A::is_external())
    }
}

///Used to get the types of the parameters and return types of methods/functions
pub trait TealMultiValue {
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
            where $($names: TealData,)*
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
    A: TealData,
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
