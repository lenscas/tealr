///A trait to turn a tuple of TealData into a list of type names.
pub trait TealMultiValue {
    fn get_types() -> Vec<&'static str>;
}
//TODO replace with macro like Rlua does
//the returned vec code should look something like : vec![A::get_type(),B::get_type()]
impl TealMultiValue for i8 {
    fn get_types() -> Vec<&'static str> {
        vec!["number"]
    }
}

impl TealMultiValue for (i8, i8) {
    fn get_types() -> Vec<&'static str> {
        vec!["number", "number"]
    }
}