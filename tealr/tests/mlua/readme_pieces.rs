use tealr::{
    compile_inline_teal, embed_compiler,
    mlu::{TealData, TealDataMethods},
    MluaUserData, TypeName, TypeWalker,
};

#[test]
fn test() {
    pieces().unwrap();
}

#[derive(Clone, MluaUserData, TypeName)]
struct Example {}

//now, implement TealData. This tells mlua what methods are available and tealr what the types are
impl TealData for Example {
    //implement your methods/functions
    fn add_methods<'lua, T: TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method("example_method", |_, _, x: i8| Ok(x));
        methods.add_method_mut("example_method_mut", |_, _, x: (i8, String)| Ok(x.1));
        methods.add_function("example_function", |_, x: Vec<String>| Ok((x, 8)));
        methods.add_function_mut("example_function_mut", |_, x: (bool, Option<Example>)| {
            Ok(x)
        })
    }
}

fn pieces() -> Result<(), mlua::Error> {
    //the functionality of these pieces of code are already being tested at other places
    //This is just to make sure the examples in the readme keep working
    if false {
        //create .d.tl file
        let _file_contents = TypeWalker::new()
            .process_type::<Example>(tealr::Direction::ToLua)
            .generate_global("test")
            .expect("oh no :(");

        //compile inline teal
        let _code = compile_inline_teal!("local x : number = 5 return x");
        //embed teal
        let compiler = embed_compiler!("v0.10.0");

        let lua = mlua::Lua::new();
        let code = compiler("example/mlua/basic_teal_file");
        let _: u8 = lua.load(&code).set_name("embedded_compiler")?.eval()?;
    }

    Ok(())
}
