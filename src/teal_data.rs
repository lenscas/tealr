use crate::teal_data_methods::TealDataMethods;


///This is the teal version of Rlua::UserData
///It can only be implemented on types that have Userdata
pub trait TealData: Sized {
    ///same as RLua::Userdata::add_methods. Refer to its documentation on how to use it.
    ///only diffrence is that it takes a TealDataMethods, which is the teal version of RLua::UserDataMethods
    fn add_methods<'lua, T: TealDataMethods<'lua, Self>>(_methods: &mut T) {}
    
    ///returns the type name as how it should show up in the generated .d.tl file
    fn get_type_name() -> &'static str;
}
//this is probably the place to implement TealData for every type that Rlua already exports.
