use tealr::{
    mlu::{mlua::Lua, FromToLua},
    ToTypename, TypeWalker,
};

#[derive(FromToLua, ToTypename, PartialEq, Debug, Clone)]
enum ExampleCStyleEnum {
    This,
    Is,
    A,
    Basic,
    Example,
}

#[derive(FromToLua, Clone, ToTypename)]
struct V(String);
impl From<String> for V {
    fn from(x: String) -> Self {
        Self(x)
    }
}
impl From<V> for String {
    fn from(x: V) -> Self {
        x.0
    }
}

#[derive(FromToLua, Clone, ToTypename, serde::Deserialize)]
#[tealr(creator_name = TestCreatorOfDOOM)]
#[serde(deny_unknown_fields)]
pub(crate) enum Test2 {
    Amazing(#[tealr(remote = V)] String),
    LessSo,
    OWowADouble(String, i8),
}
impl From<String> for Test2 {
    fn from(field: String) -> Self {
        Self::Amazing(field)
    }
}
impl From<Test2> for String {
    fn from(s: Test2) -> Self {
        match s {
            Test2::Amazing(x) => x,
            Test2::LessSo => String::new(),
            Test2::OWowADouble(x, _) => x,
        }
    }
}

#[derive(Clone, Debug, FromToLua, ToTypename, PartialEq)]
struct Example {
    #[tealr(remote = Test2)]
    field1: String,
    nice: i32,
    v: ExampleCStyleEnum,
}

#[test]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    //lets first generate the definition file
    let file_contents = TypeWalker::new()
        .process_type::<V>()
        .process_type::<ExampleCStyleEnum>()
        .process_type::<TestCreatorOfDOOM>()
        .process_type::<Test2>()
        .process_type::<Example>()
        .to_json()
        .expect("oh no :(");

    let new_value: serde_json::Value = serde_json::from_str(&file_contents).unwrap();
    let mut old_value: serde_json::Value =
        serde_json::from_str(include_str!("./to_from_derived.json")).unwrap();

    let mut x = old_value
        .get_mut("tealr_version_used")
        .expect("missing tealr_version_used in original");
    if let serde_json::Value::String(x) = &mut x {
        *x = tealr::get_tealr_version().to_string();
    }

    assert_eq!(new_value, old_value);

    let mut to_pass = Example {
        field1: String::from("nice"),
        nice: 2,
        v: ExampleCStyleEnum::Basic,
    };

    let lua = Lua::new();
    let globals = lua.globals();

    globals.set("test", to_pass.clone())?;
    globals.set("creator", TestCreatorOfDOOM::new())?;
    let code = "
    local v = assert(test.field1:GetAmazingOrNil())
    assert(v[0] == \"nice\")
    assert(test.nice == 2)
    test.field1 = creator.NewLessSo()
    return test
";
    let res: Example = lua.load(code).set_name("test?").eval()?;
    to_pass.field1 = String::from(Test2::LessSo);
    assert_eq!(res, to_pass);

    Ok(())
}
