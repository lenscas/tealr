use tealr::{
    mlu::{
        mlua::{FromLua, Lua},
        TealData, TealDataMethods, UserData,
    },
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
impl FromLua for Example {
    fn from_lua(value: mlua::prelude::LuaValue, _: &Lua) -> tealr::mlu::mlua::Result<Self> {
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
    let mut old_value: serde_json::Value =
        serde_json::from_str(include_str!("inlined_type.json")).unwrap();
    let mut x = old_value
        .get_mut("tealr_version_used")
        .expect("missing tealr_version_used in original");
    if let serde_json::Value::String(x) = &mut x {
        *x = tealr::get_tealr_version().to_string();
    }
    assert_eq!(new_value, old_value);
}
