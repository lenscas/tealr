use mlua::{Lua, Result};
use tealr::{
    mlu::{TealData, TealDataMethods, UserData},
    ToTypename, TypeWalker,
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
#[derive(Clone, UserData, ToTypename)]
struct Example {}

//now, implement TealData. This tells rlua what methods are available and tealr what the types are
impl TealData for Example {
    //implement your methods/functions
    fn add_methods<'lua, T: TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_async_method("example_method", |_, _, x: i8| std::future::ready(Ok(x)));
        methods.add_async_function("example_method_mut", |_, x: (i8, String)| {
            std::future::ready(Ok(x.1))
        });
        methods.add_function("example_function", |_, x: Vec<String>| Ok((x, 8)));
        methods.add_function_mut("example_function_mut", |_, x: (bool, Option<Example>)| {
            Ok(x)
        })
    }
}

#[test]
fn async_fn() -> Result<()> {
    //lets first generate the definition file
    let file_contents = TypeWalker::new() //creates the generator
        //tells it that we want to generate Example
        //add more calls to process_type to generate more types in the same file
        .process_type::<Example>()
        //generate the file
        .generate_global("test")
        //the name parameter for TealDataMethods::{add_method,add_method_mut,add_function,add_function_mut}
        //takes anything that can be used as a &[u8]
        //this is to match the types from UserDataMethods
        //however, as we turn it back into a string it is technically possible to get an error
        //in this case, as &str's where used it can't happen though, so the .expect is fine
        .expect("oh no :(");
    assert_eq!(file_contents, "global record test\n\trecord Example\n\t\tuserdata\n\n\t\t-- Pure methods\n\t\texample_method: function(self:Example , integer):integer\n\n\t\t-- Pure functions\n\t\texample_method_mut: function(integer , string):string\n\n\t\texample_function: function({string}):{string} , integer\n\n\t\t-- Mutating functions\n\t\texample_function_mut: function(boolean , Example):boolean , Example\n\n\n\tend\nend\nreturn test");
    //normally you would now save the file somewhere.
    println!("{}\n ", file_contents);

    //how you pass this type to lua hasn't changed:
    let lua = Lua::new();
    let globals = lua.globals();
    globals.set("test", Example {})?;
    let code = "
    test:example_method(2)
    ";
    let x: i8 = lua.load(code).set_name("test?")?.eval()?;

    assert_eq!(x, 2);
    Ok(())
}
