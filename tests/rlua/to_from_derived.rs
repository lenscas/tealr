use tealr::{rlu::rlua::Lua, RluaFromToLua, TypeName, TypeWalker};

#[derive(RluaFromToLua, TypeName, PartialEq, Debug, Clone)]
enum ExampleCStyleEnum {
    This,
    Is,
    A,
    Basic,
    Example,
}

#[derive(RluaFromToLua, Clone, TypeName)]
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

#[derive(RluaFromToLua, Clone, TypeName)]
#[tealr(creator_name = TestCreatorOfDOOM)]
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

#[derive(Clone, Debug, RluaFromToLua, TypeName, PartialEq)]
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
        .generate_global("test")
        .expect("oh no :(");

    assert_eq!(
        "global record test\n\trecord V\n\n\t\t-- Fields\n\t\t0 : string\n\n\n\tend\n\tenum ExampleCStyleEnum\n\t\t\"This\"\n\t\t\"Is\"\n\t\t\"A\"\n\t\t\"Basic\"\n\t\t\"Example\"\n\tend\n\trecord TestCreatorOfDOOM\n\t\tuserdata\n\n\t\t-- Pure functions\n\t\tNewAmazingFrom: function(V):(Test2)\n\n\t\tNewLessSo: function():(Test2)\n\n\t\tNewOWowADoubleFrom: function((string),(integer)):(Test2)\n\n\n\tend\n\trecord Test2\n\t\tuserdata\n\n\t\t-- Pure methods\n\t\tIsAmazing: function(Test2):(boolean)\n\n\t\tGetAmazing: function(Test2):((boolean),(V))\n\n\t\tGetAmazingOrNil: function(Test2):(V)\n\n\t\tIsLessSo: function(Test2):(boolean)\n\n\t\tIsOWowADouble: function(Test2):(boolean)\n\n\t\tGetOWowADouble: function(Test2):((boolean),(string),(integer))\n\n\t\tGetOWowADoubleOrNil: function(Test2):((string),(integer))\n\n\n\tend\n\trecord Example\n\n\t\t-- Fields\n\t\tfield1 : Test2\n\n\t\tnice : integer\n\n\t\tv : ExampleCStyleEnum\n\n\n\tend\nend\nreturn test",
        file_contents
    );

    let mut to_pass = Example {
        field1: String::from("nice"),
        nice: 2,
        v: ExampleCStyleEnum::Example,
    };

    let lua = Lua::new();
    let res: Example = lua.context(|lua| {
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
        lua.load(code).set_name("test?")?.eval()
    })?;
    to_pass.field1 = String::from(Test2::LessSo);
    assert_eq!(res, to_pass);

    Ok(())
}
