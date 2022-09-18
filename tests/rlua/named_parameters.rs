use tealr::{
    rlu::{rlua::Result, TealData, TealDataMethods, UserData},
    TypeName, TypeWalker,
};
#[derive(Clone, UserData, TypeName)]
struct Example {}

impl TealData for Example {
    fn add_methods<'lua, T: TealDataMethods<'lua, Self>>(methods: &mut T) {
        tealr::rlu::create_named_parameters!(
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
        .generate_global("test")
        .expect("oh no :(");

    assert_eq!(file_contents, "global record test\n\trecord Example\n\t\tuserdata\n\n\t\t-- Pure methods\n\t\texample_method: function(Example,field_1 : string , field_2 : integer):((string),(integer))\n\n\n\tend\nend\nreturn test");

    tealr::rlu::rlua::Lua::new().context(|ctx| {
        let globals = ctx.globals();
        globals.set("test", Example {})?;
        let code = "return test:example_method(\"field_1 is a string\", 3)";
        let (field1, field2): (String, i64) = ctx.load(code).set_name("test?")?.eval()?;
        assert_eq!(field1, "field_1 is a string");
        assert_eq!(field2, 3);
        Ok(())
    })?;
    Ok(())
}
