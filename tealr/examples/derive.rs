use rlua::{Lua, Result};
use tealr::{
    rlu::{TealData, TealDataMethods},
    TypeName, TypeWalker, UserData,
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
#[derive(Clone, UserData, TypeName)]
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

fn main() -> Result<()> {
    //lets first generate the definition file
    let file_contents = TypeWalker::new() //creates the generator
        //tells it that we want to generate Example
        //add more calls to process_type to generate more types in the same file
        .process_type::<Example>(tealr::Direction::ToLua)
        //generate the file
        .generate_global("test")
        //the name parameter for TealDataMethods::{add_method,add_method_mut,add_function,add_function_mut}
        //takes anything that can be used as a &[u8]
        //this is to match the types from UserDataMethods
        //however, as we turn it back into a string it is technically possible to get an error
        //in this case, as &str's where used it can't happen though, so the .expect is fine
        .expect("oh no :(");

    //normally you would now save the file somewhere.
    println!("{}\n ", file_contents);

    //how you pass this type to lua hasn't changed:
    let lua = Lua::new();
    lua.context(|lua_ctx| {
        let globals = lua_ctx.globals();
        globals.set("test", Example {})?;
        let code = "
print(test:example_method(1))
print(test:example_method_mut(2,\"test\"))
print(test.example_function({}))
print(test.example_function_mut(true))
        ";
        lua_ctx.load(code).set_name("test?")?.eval()?;
        Ok(())
    })?;
    Ok(())
}
