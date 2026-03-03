use tealr::{
    mlu::{
        mlua::{FromLua, Lua, Result},
        TealData, TealDataMethods, UserData,
    },
    ToTypename, TypeWalker,
};
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

impl TealData for Example {
    fn add_methods<T: TealDataMethods<Self>>(methods: &mut T) {
        tealr::mlua_create_named_parameters!(
            TestName with
                field_1 : String,
                field_2 : i64,
        );
        methods.add_method("example_method", |_, _, a: TestName| {
            Ok((a.field_1, a.field_2))
        });
    }
}

#[test]
fn main() -> Result<()> {
    let file_contents = TypeWalker::new()
        .process_type::<Example>()
        .to_json()
        .expect("oh no :(");

    let generated: serde_json::Value = serde_json::from_str(&file_contents).unwrap();
    let mut old_value: serde_json::Value =
        serde_json::from_str(include_str!("named_parameters.json")).unwrap();

    let mut x = old_value
        .get_mut("tealr_version_used")
        .expect("missing tealr_version_used in original");
    if let serde_json::Value::String(x) = &mut x {
        *x = tealr::get_tealr_version().to_string();
    }

    assert_eq!(generated, old_value);

    let lua = Lua::new();
    let globals = lua.globals();
    globals.set("test", Example {})?;
    let code = "return test:example_method(\"field_1 is a string\", 3)";
    let (field1, field2): (String, i64) = lua.load(code).set_name("test?").eval()?;
    assert_eq!(field1, "field_1 is a string");
    assert_eq!(field2, 3);
    Ok(())
}
