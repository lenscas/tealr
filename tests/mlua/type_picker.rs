use mlua::{IntoLua, Lua};
use tealr::{
    create_union_mlua,
    mlu::{mlua::FromLua, TealData, TealDataMethods, TypedFunction, UserData},
    ToTypename, TypeWalker,
};

create_union_mlua!(enum X = String | f32 | bool);

#[derive(Clone, UserData, ToTypename)]
struct Example {}

//now, implement TealData. This tells mlua what methods are available and tealr what the types are
impl TealData for Example {
    //implement your methods/functions
    fn add_methods<'lua, T: TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method("limited_callback", |lua, _, fun: TypedFunction<X, X>| {
            let param = X::from_lua("nice!".into_lua(lua)?, lua)?;
            let res = fun.call(param)?;
            Ok(res)
        });
        methods.add_method("limited_array", |_, _, x: Vec<X>| Ok(x));
        methods.add_method("limited_simple", |_, _, x: X| Ok(x));
    }
}

#[test]
fn test_limited() {
    let file_contents = TypeWalker::new()
        .process_type::<Example>()
        .to_json()
        .expect("oh no :(");

    let new_value: serde_json::Value = serde_json::from_str(&file_contents).unwrap();
    let mut old_value: serde_json::Value =
        serde_json::from_str(include_str!("./type_picker.json")).unwrap();

    let mut x = old_value
        .get_mut("tealr_version_used")
        .expect("missing tealr_version_used in original");
    if let serde_json::Value::String(x) = &mut x {
        *x = tealr::get_tealr_version().to_string();
    }

    assert_eq!(new_value, old_value);
    let lua = Lua::new();
    let globals = lua.globals();
    globals.set("test", Example {}).unwrap();
    let code = "
    return test:limited_simple(true)
    ";
    let x: bool = lua.load(code).set_name("test_limited_lua").eval().unwrap();
    assert!(x);
}
