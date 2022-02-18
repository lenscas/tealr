use tealr::{
    compile_inline_teal, create_union_rlua, embed_compiler,
    rlu::{TealData, TealDataMethods},
    RluaUserData, TypeName, TypeWalker,
};

#[test]
fn test() {
    pieces().unwrap();
}

#[derive(Clone, tealr::RluaUserData, TypeName)]
struct ExampleRlua {}

//now, implement rlu::TealData.
//This tells rlua what methods are available and tealr what the types are
impl tealr::rlu::TealData for ExampleRlua {
    //implement your methods/functions
    fn add_methods<'lua, T: tealr::rlu::TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.document_type("This is documentation added to the type itself.");

        methods.document("This documentation gets added to the exposed function bellow.");
        methods.add_method("example_method", |_, _, x: i8| Ok(x));
        methods.add_method_mut("example_method_mut", |_, _, x: (i8, String)| Ok(x.1));
        methods.add_function("example_function", |_, x: Vec<String>| Ok((x, 8)));
        methods.document("***You*** can also embed markdown to the documentation, which gets picked up by [tealr_doc_gen](https://github.com/lenscas/type_generator)`");
        methods.document("It is also possible to use this function multiple times. These are added as paragraphs.");
        methods.add_function_mut(
            "example_function_mut",
            |_, x: (bool, Option<ExampleRlua>)| Ok(x),
        );
        ///This creates the instance.help() function, which returns the documentation as a string.
        methods.generate_help()
    }
}
create_union_rlua!(enum YourTypeName = i32 | String);

fn pieces() -> Result<(), rlua::Error> {
    //the functionality of these pieces of code are already being tested at other places
    //This is just to make sure the examples in the readme keep working
    if false {
        //create .d.tl file
        let _file_contents = TypeWalker::new()
            .process_type::<ExampleRlua>(tealr::Direction::ToLua)
            .generate_global("test")
            .expect("oh no :(");

        //compile inline teal
        let _code = compile_inline_teal!("local x : number = 5 return x");
        //embed teal
        let compiler = embed_compiler!("v0.13.1");
        let res: u8 = tealr::rlu::rlua::Lua::new().context(|ctx| {
            let code = compiler("example/basic_teal_file");
            ctx.load(&code).set_name("embedded_compiler")?.eval()
        })?;
    }

    Ok(())
}
