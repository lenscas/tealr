use tealr::{
    mlu::{
        mlua::{FromLua, Lua, Result, UserData, UserDataMethods},
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
        methods.document_type("This is just an example type");
        methods.document_type("This part of the documentation is for the type itself");
        methods.document_type(
            "That means it gets placed before the `record {name}` part in the .d.tl file",
        );
        methods.document_type(
            "And is also visible when calling instance.help() without any parameters",
        );

        methods.document("This documentation is for the next registered method.");
        methods.document("In this case that will be example_method");
        methods.document("This means that it gets placed before this method in the .d.tl file");
        methods.document("You can access it by calling instance.help(\"example_method\")");
        methods.add_method("example_method", |_, _, x: i8| Ok(x));
        methods.add_method_mut("example_method_mut", |_, _, x: (i8, String)| Ok(x.1));
        methods.add_function("example_function", |_, x: Vec<String>| Ok((x, 8)));
        methods.add_function_mut("example_function_mut", |_, x: (bool, Option<Example>)| {
            Ok(x)
        });
        methods.generate_help();
    }
    fn add_fields<F: tealr::mlu::TealDataFields<Self>>(fields: &mut F) {
        fields.document("This is an example field");
        fields.add_field_method_get("example", |_, this| Ok(this.0));
        fields.document("Documentation for fields with the same name get merged");
        fields.add_field_method_set("example", |_, this, value| {
            this.0 = value;
            Ok(())
        })
    }
}

impl ToTypename for Example {
    //how the type should be called in lua.
    fn to_typename() -> tealr::Type {
        tealr::Type::new_single("Example", tealr::KindOfType::External)
    }
}

impl UserData for Example {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        let mut wrapper = UserDataWrapper::from_user_data_fields(fields);
        <Self as TealData>::add_fields(&mut wrapper);
    }
    fn add_methods<T: UserDataMethods<Self>>(methods: &mut T) {
        let mut wrapper = UserDataWrapper::from_user_data_methods(methods);
        <Self as TealData>::add_methods(&mut wrapper);
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
fn main() {
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
    globals.set("test", Example(1)).unwrap();
    let code = "
print(test.help())
print(\"----\")
print(test.help(\"example_method\"))
print(test:example_method(1))
print(test:example_method_mut(2,\"test\"))
print(test.example_function({}))
print(test.example_function_mut(true))
        ";

    lua.load(code).set_name("test?").eval::<()>().unwrap();
}
