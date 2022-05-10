use tealr::{
    compile_inline_teal, create_generic_mlua, create_union_mlua, embed_compiler,
    mlu::{TealData, TealDataMethods, TypedFunction},
    MluaUserData, TypeName, TypeWalker,
};

#[test]
fn test() {
    pieces().unwrap();
}
#[derive(Clone, tealr::MluaUserData, TypeName)]
struct ExampleMlua {}
impl tealr::mlu::TealData for ExampleMlua {
    //implement your methods/functions
    fn add_methods<'lua, T: tealr::mlu::TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.document_type("This is documentation added to the type itself.");
        methods.document("This documentation gets added to the exposed function bellow.");
        methods.add_method("example_method", |_, _, x: i8| Ok(x));
        methods.add_method_mut("example_method_mut", |_, _, x: (i8, String)| Ok(x.1));
        methods.add_function("example_function", |_, x: Vec<String>| Ok((x, 8)));
        methods.document("***You*** can also embed markdown to the documentation, which gets picked up by [tealr_doc_gen](https://github.com/lenscas/type_generator)`");
        methods.document("It is also possible to use this function multiple times. These are added as paragraphs.");
        methods.add_function_mut(
            "example_function_mut",
            |_, x: (bool, Option<ExampleMlua>)| Ok(x),
        );
        //This creates the instance.help() function, which returns the documentation as a string.
        methods.generate_help()
    }
}

create_union_mlua!(enum YourTypeName = i32 | String);

create_generic_mlua!(X);
#[derive(Clone, MluaUserData, TypeName)]
struct Example {
    example: u32,
}

impl TealData for Example {
    fn add_methods<'lua, T: TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method(
            "generic_function_callback",
            |_, _, fun: TypedFunction<String, X>| fun.call("A nice string!".to_string()),
        );
    }
    fn add_fields<'lua, F: tealr::mlu::TealDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("example", |_, this| Ok(this.example));
        fields.add_field_method_set("example", |_, this, value| {
            this.example = value;
            Ok(())
        })
    }
}

fn pieces() -> Result<(), mlua::Error> {
    //the functionality of these pieces of code are already being tested at other places
    //This is just to make sure the examples in the readme keep working
    if false {
        //create .d.tl file
        let _file_contents = TypeWalker::new()
            .process_type::<ExampleMlua>()
            .generate_global("test")
            .expect("oh no :(");

        //compile inline teal
        let _code = compile_inline_teal!("local x : number = 5 return x");
        //embed teal
        let compiler = embed_compiler!("v0.13.1");

        let code = compiler("example/basic_teal_file");
        let lua = tealr::mlu::mlua::Lua::new();
        let _res: u8 = lua.load(&code).set_name("embedded_compiler")?.eval()?;
    }

    Ok(())
}
