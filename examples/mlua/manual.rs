use tealr::{
    mlu::{
        mlua::{Lua, Result, UserData, UserDataMethods},
        TealData, TealDataMethods, UserDataWrapper,
    },
    ToTypename, TypeBody, TypeWalker,
};
//This example shows how to manually implement UserData using TealData
//As you can see the amount of code is small and easy copy/paste able.
//Because of this it may make sense to do the implementation yourself
//instead of paying the compile time cost of the macro

//First, create the struct you want to export to lua.
#[derive(Clone, Copy)]
struct Example(u32);

//now, implement TealData. This tells rlua what methods are available and tealr what the types are
impl TealData for Example {
    //implement your methods/functions
    fn add_methods<'lua, T: TealDataMethods<'lua, Self>>(methods: &mut T) {
        //methods.add_method("example_method", |_, _, x: i8| Ok(x));
        methods.add_method(
            "example_method",
            |_, _, x: tealr::mlu::TypedFunction<i32, i64>| Ok(x),
        );
        methods.add_method_mut("example_method_mut", |_, _, x: (i8, String)| Ok(x.1));
        methods.add_function("example_function", |_, x: Vec<String>| Ok((x, 8)));
        methods.add_function_mut("example_function_mut", |_, x: (bool, Option<Example>)| {
            Ok(x)
        })
    }
    fn add_fields<'lua, F: tealr::mlu::TealDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("example_field", |_, this| Ok(this.0));
        fields.add_field_method_set("example_field", |_, this, value| {
            this.0 = value;
            Ok(())
        });
    }
}

impl ToTypename for Example {
    //how the type should be called in lua.
    fn to_typename() -> tealr::Type {
        tealr::Type::new_single("Example", tealr::KindOfType::External)
    }
}

impl UserData for Example {
    fn add_methods<'lua, T: UserDataMethods<'lua, Self>>(methods: &mut T) {
        let mut wrapper = UserDataWrapper::from_user_data_methods(methods);
        <Self as TealData>::add_methods(&mut wrapper)
    }
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        let mut wrapper = UserDataWrapper::from_user_data_fields(fields);
        <Self as TealData>::add_fields(&mut wrapper)
    }
}

impl TypeBody for Example {
    fn get_type_body() -> tealr::TypeGenerator {
        let mut gen = tealr::RecordGenerator::new::<Self>(false);
        gen.is_user_data = true;
        <Self as TealData>::add_fields(&mut gen);
        <Self as TealData>::add_methods(&mut gen);
        gen.into()
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
    //however for this example we just print it.
    println!("{}\n ", file_contents);
    //how you pass this type to lua hasn't changed:
    let lua = Lua::new();

    let globals = lua.globals();
    globals.set("test", Example(1))?;
    let code = "
print(test:example_method(function()return 1 end))
print(test:example_method_mut(2,\"test\"))
print(test.example_function({}))
print(test.example_function_mut(true))
print(\"Example field\", test.example_field)
test.example_field = 2
print(\"After modifying\",test.example_field)
        ";
    lua.load(code).set_name("test?")?.eval()?;
    Ok(())
}
