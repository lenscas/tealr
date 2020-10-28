use rlua::{Lua, Result, UserData, UserDataMethods};
use tealr::{TealData, TealDataMethods, TypeRepresentation, TypeWalker, UserDataWrapper};
//This example shows how to manually implement UserData using TealData
//As you can see the amount of code is small and easy copy/pasteable.
//Because of this it may make sense to do the implementation yourself
//instead of paying the compile time cost of the macro

//First, create the struct you want to export to lua.
#[derive(Clone, Copy)]
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

impl TypeRepresentation for Example {
    //how the type should be called in lua.
    fn get_type_name() -> std::borrow::Cow<'static, str> {
        "Example".into()
    }
}

impl UserData for Example {
    fn add_methods<'lua, T: UserDataMethods<'lua, Self>>(methods: &mut T) {
        let mut wrapper = UserDataWrapper::from_user_data_methods(methods);
        <Self as TealData>::add_methods(&mut wrapper)
    }
}

fn main() -> Result<()> {
    let file_contents = TypeWalker::new() //creates the generator
        //tells it that we want to generate Example
        //add more calls to process_type to generate more types in the same file
        .proccess_type::<Example>()
        //generate the file
        .generate_global("test")
        //due to how the typings work, we technically can get an error.
        //this is however rather unlikely, so using a .expect is probly fine
        .expect("oh no :(");
    //normally you would now save the file somewhere.
    //however for this example we just print it.
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
