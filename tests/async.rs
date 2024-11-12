use tealr::{
    mlu::{
        mlua::{FromLua, Lua, Result},
        TealData, TealDataMethods, UserData,
    },
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

impl FromLua for Example {
    fn from_lua(value: mlua::prelude::LuaValue, _: &Lua) -> Result<Self> {
        value
            .as_userdata()
            .map(|x| x.take())
            .unwrap_or(Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "Example".to_string(),
                message: None,
            }))
    }
}

//now, implement TealData. This tells mlua what methods are available and tealr what the types are
impl TealData for Example {
    //implement your methods/functions
    fn add_methods<T: TealDataMethods<Self>>(methods: &mut T) {
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
        .to_json()
        .expect("oh no :(");

    let generated: serde_json::Value = serde_json::from_str(&file_contents).unwrap();

    let mut original: serde_json::Value = serde_json::from_str(include_str!("async.json")).unwrap();
    let x = original
        .get_mut("tealr_version_used")
        .expect("missing tealr_version_used in original");
    if let serde_json::Value::String(x) = x {
        *x = tealr::get_tealr_version().to_string();
    }

    assert_eq!(generated, original);

    //how you pass this type to lua hasn't changed:
    let lua = Lua::new();
    let globals = lua.globals();
    globals.set("test", Example {})?;
    let code = "
    test:example_method(2)
    ";
    let x: i8 = lua.load(code).set_name("test?").eval()?;

    assert_eq!(x, 2);
    Ok(())
}
