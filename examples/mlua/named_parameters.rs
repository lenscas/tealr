use tealr::{
    mlu::{
        mlua::{Lua, Result},
        TealData, TealDataMethods, UserData,
    },
    TypeName, TypeWalker,
};
//this example shows how to use the create_named_parameters! macro to create methods which has names for their parameters in the documentation
#[derive(Clone, UserData, TypeName)]
struct Example {}

impl TealData for Example {
    fn add_methods<'lua, T: TealDataMethods<'lua, Self>>(methods: &mut T) {
        //this creates a new struct that will contain our parameters
        //it has a field with the name `field_1` of type `String`
        //and a field with the name `field_2` of type `i64`
        tealr::mlua_create_named_parameters!(
            TestName with
                field_1 : String,
                field_2 : i64,
        );
        methods.add_method("example_method", |_, _, a: TestName| {
            println!("field_1 = {}; field_2 = {}", a.field_1, a.field_2);
            Ok(())
        });
    }
}

fn main() -> Result<()> {
    let file_contents = TypeWalker::new()
        //tells it that you want to include the Example type
        //chain extra calls to include more types
        .process_type::<Example>()
        //generate the file
        .to_json()
        .expect("serde_json failed to serialize our data");

    //normally you would now save the file somewhere.
    println!("{}\n ", file_contents);

    //lua is still using position parameters as normal.
    let lua = Lua::new();
    let globals = lua.globals();
    globals.set("test", Example {})?;
    let code = "test:example_method(\"field_1 is a string\", 3)";
    lua.load(code).set_name("test?")?.eval()?;
    Ok(())
}
