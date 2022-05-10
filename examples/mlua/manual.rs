use tealr::{
    mlu::{
        mlua::{Lua, Result, UserData, UserDataMethods},
        TealData, TealDataMethods, UserDataWrapper,
    },
    new_type, NamePart, TypeBody, TypeName, TypeWalker,
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

impl TypeName for Example {
    //how the type should be called in lua.
    fn get_type_parts() -> std::borrow::Cow<'static, [NamePart]> {
        //this
        new_type!(Example)
        //is the same as this:
        // std::borrow::Cow::Borrowed(&[NamePart::Type(TealType {
        // name: std::borrow::Cow::Borrowed("Example"),
        // type_kind: tealr::KindOfType::External,
        // generics: None,
        // })])
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
    fn get_type_body(gen: &mut tealr::TypeGenerator) {
        gen.is_user_data = true;
        <Self as TealData>::add_fields(gen);
        <Self as TealData>::add_methods(gen);
    }
}
fn main() -> Result<()> {
    let file_contents = TypeWalker::new() //creates the generator
        //tells it that we want to generate Example
        //add more calls to process_type to generate more types in the same file
        .process_type::<Example>()
        //generate the file
        .generate_global("test")
        //due to how the typings work, we technically can get an error.
        //this is however rather unlikely, so using a .expect is probably fine
        .expect("oh no :(");
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
    let x = lua.load(code).set_name("test?")?.eval();
    x
}
