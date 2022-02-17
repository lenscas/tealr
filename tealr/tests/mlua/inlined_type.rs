use tealr::{
    mlu::{TealData, TealDataMethods},
    MluaUserData, TypeName, TypeWalker,
};
//this example shows how the new traits allow you to generate the .d.tl file
//and shows how to use them to share data with lua
//It also shows how to generate the file
//NOTE: All it does it generate the contents of the file. Storing it is left to the user.

//First, create the struct you want to export to lua.
//instead of both deriving UserData and TypeName you can also
//derive TealDerive, which does both. However you will still need to import
//UserData and TypeName
//The clone is only needed because one of the example functions has it as a parameter
#[derive(Clone, MluaUserData, TypeName)]
struct Example {}

//now, implement TealData. This tells rlua what methods are available and tealr what the types are
impl TealData for Example {
    //implement your methods/functions
    fn add_methods<'lua, T: TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method("example_method", |_, _, x: i8| Ok(x));
        methods.add_method_mut("example_method_mut", |_, _, x: (i8, String)| Ok(x.1));
        methods.add_function("example_function", |_, x: Vec<String>| Ok((x, 8)));
        methods.add_function_mut("example_function_mut", |_, x: (bool, Option<Example>)| {
            Ok(x)
        })
    }
}

#[test]
fn make_inline_type() {
    let file_contents = TypeWalker::new()
        .process_type_inline::<Example>(tealr::Direction::ToLua)
        .process_type::<Example>(tealr::Direction::ToLua)
        .generate_global("test")
        .expect("oh no :(");

    assert_eq!(file_contents,"global record test\n\t-- Example\n\n\t\t-- Pure methods\n\t\texample_method: function(Example,integer):(integer)\n\n\t\t-- Mutating methods\n\t\texample_method_mut: function(Example,(integer),(string)):(string)\n\n\t\t-- Pure functions\n\t\texample_function: function({string}):(({string}),(integer))\n\n\t\t-- Mutating functions\n\t\texample_function_mut: function((boolean),(Example)):((boolean),(Example))\n\n\n\n\trecord Example\n\t\tuserdata\n\n\t\t-- Pure methods\n\t\texample_method: function(Example,integer):(integer)\n\n\t\t-- Mutating methods\n\t\texample_method_mut: function(Example,(integer),(string)):(string)\n\n\t\t-- Pure functions\n\t\texample_function: function({string}):(({string}),(integer))\n\n\t\t-- Mutating functions\n\t\texample_function_mut: function((boolean),(Example)):((boolean),(Example))\n\n\n\tend\nend\nreturn test");
}
