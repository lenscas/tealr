use crate::teal_data_methods::TealDataMethods;


///This is the teal version of [UserData](rlua::UserData).
pub trait TealData: Sized {
    ///same as [UserData::add_methods](rlua::UserData::add_methods).
    ///Refer to its documentation on how to use it.
    ///
    ///only diffrence is that it takes a [TealDataMethods](crate::TealDataMethods),
    ///which is the teal version of [UserDataMethods](rlua::UserDataMethods)
    fn add_methods<'lua, T: TealDataMethods<'lua, Self>>(_methods: &mut T) {}
    
    ///returns the type name as how it should show up in the generated `.d.tl` file
    fn get_type_name() -> &'static str;
    ///this method tells the generator whether a type is from `lua`/`teal`
    ///or is always available in some other way
    ///
    ///***DO NOT*** overwrite it unless you are ***ABSOLUTLY*** sure you need to.
    ///The default is correct for 99.999% of the cases.
    fn is_external() -> bool {
        true
    }
}