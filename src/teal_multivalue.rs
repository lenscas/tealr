use crate::teal_data::TealData;

///Represents a type
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TealType {
    pub(crate) name: &'static str,
    pub(crate) is_external: bool,
}
impl TealType {
    fn new(name: &'static str, is_external: bool) -> Self {
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
//TODO replace with macro like Rlua does
//the returned vec code should look something like : vec![A::get_type(),B::get_type()]
impl<T: TealData> TealMultiValue for T {
    fn get_types() -> Vec<TealType> {
        vec![TealType::from::<T>()]
    }
}

impl<A: TealData, B: TealData> TealMultiValue for (A, B) {
    fn get_types() -> Vec<TealType> {
        vec![TealType::from::<A>(), TealType::from::<B>()]
    }
}

impl TealData for i8 {
    fn get_type_name() -> &'static str {
        "number"
    }
    fn is_external() -> bool {
        false
    }
}
impl TealData for f32 {
    fn get_type_name() -> &'static str {
        "number"
    }
    fn is_external() -> bool {
        false
    }
}
impl TealData for String {
    fn get_type_name() -> &'static str {
        "string"
    }
    fn is_external() -> bool {
        false
    }
}
