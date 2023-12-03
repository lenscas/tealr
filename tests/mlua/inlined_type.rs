use tealr::{
    mlu::{TealData, TealDataMethods, UserData},
    ToTypename, TypeWalker,
};
//this example shows how the new traits allow you to generate the .d.tl file
//and shows how to use them to share data with lua
//It also shows how to generate the file
//NOTE: All it does it generate the contents of the file. Storing it is left to the user.

//First, create the struct you want to export to lua.
//instead of both deriving UserData and ToTypename you can also
//derive TealDerive, which does both. However you will still need to import
//UserData and ToTypename
//The clone is only needed because one of the example functions has it as a parameter
#[derive(Clone, UserData, ToTypename)]
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
        .process_type_inline::<Example>()
        .process_type::<Example>()
        .to_json_pretty()
        .expect("oh no :(");

    let new_value: serde_json::Value = serde_json::from_str(&file_contents).unwrap();
    let old_value: serde_json::Value =
        serde_json::from_str(include_str!("./inlined_type.json")).unwrap();
    assert_eq!(new_value, old_value);
}
