# tealr
A wrapper around [rlua](https://crates.io/crates/rlua) and/or [mlua](https://crates.io/crates/mlua) to help with embedding teal

tealr adds some traits that replace/extend those from [rlua](https://crates.io/crates/rlua) and [mlua](https://crates.io/crates/mlua),
allowing it to generate the `.d.tl` files needed for teal.
It also contains some macro's to make it easier to load/execute teal scripts.

### Note:
Both `rlua` and `mlua` are behind feature flags with the same name.

It also reexports these crates and allows you to set flags through it (the forwarded flags are the same with either the prefix `rlua_` or `mlua_`. For example if you want to enable `mlua/async` then you need to enable `tealr/mlua_async`)

## Expose a value to teal
Exposing types to teal as userdata is almost the same using tealr as it is using rlua and mlua
```rust ignore
 use tealr::TypeName;
#[derive(Clone, tealr::RluaUserData, TypeName)]
struct ExampleRlua {}

//now, implement rlu::TealData.
//This tells rlua what methods are available and tealr what the types are
impl tealr::rlu::TealData for ExampleRlua {
    //implement your methods/functions
    fn add_methods<'lua, T: tealr::rlu::TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method("example_method", |_, _, x: i8| Ok(x));
        methods.add_method_mut("example_method_mut", |_, _, x: (i8, String)| Ok(x.1));
        methods.add_function("example_function", |_, x: Vec<String>| Ok((x, 8)));
        methods.add_function_mut("example_function_mut", |_, x: (bool, Option<ExampleRlua>)| {
            Ok(x)
        })
    }
}
```
Working with mlua is almost the same
```rust ignore
 use tealr::TypeName;
#[derive(Clone, tealr::MluaUserData, TypeName)]
struct ExampleMlua {}
impl tealr::mlu::TealData for ExampleMlua {
    //implement your methods/functions
    fn add_methods<'lua, T: tealr::mlu::TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method("example_method", |_, _, x: i8| Ok(x));
        methods.add_method_mut("example_method_mut", |_, _, x: (i8, String)| Ok(x.1));
        methods.add_function("example_function", |_, x: Vec<String>| Ok((x, 8)));
        methods.add_function_mut("example_function_mut", |_, x: (bool, Option<ExampleMlua>)| {
            Ok(x)
        })
    }
}
```
## Create a .d.tl file
Creating of the `.d.tl` files works the same for rlua or mlua
```rust
//set your type up with either rlua or mlua
use tealr::{TypeName};
#[cfg(feature = "mlua")]
use tealr::{MluaUserData,mlu::TealData};
#[cfg(feature = "rlua")]
use tealr::{RluaUserData,rlu::TealData};
#[cfg_attr(feature = "mlua", derive(MluaUserData, TypeName))]
#[cfg_attr(feature = "rlua", derive(RluaUserData, TypeName))]
struct Example {}
impl TealData for Example {};

//time to create the type definitions
let file_contents = tealr::TypeWalker::new()
    .process_type::<Example>(tealr::Direction::ToLua)
    .generate_global("test")
    .expect("oh no :(");

//write the output to a file
println!("{}",file_contents)
```
## Compile inline teal code into lua
 As you get a string containing the lua code back this feature works the same for both rlua and mlua
```rust
use tealr::compile_inline_teal;
let code = compile_inline_teal!("local x : number = 5 return x");
```

## Embed the teal compiler

It is possible to embed the teal compiler into your rust application.
This makes it possible to easily load it into the lua vm thus allowing it to run teal files as normal lua files.

```rust no_run
#[cfg(feature = "rlua")]
{
    use tealr::embed_compiler;
    let compiler = embed_compiler!("v0.13.1");
    let res = rlua::Lua::new().context(|ctx| {
        let code = compiler("example/basic_teal_file");
        let res: u8 = ctx.load(&code).set_name("embedded_compiler")?.eval()?;
        Ok(res)
    })?;
    return Ok::<(), Box<dyn std::error::Error>>(());
};

#[cfg(feature = "mlua")]
{
    use tealr::embed_compiler;
    let compiler = embed_compiler!("v0.13.1");
    let lua = mlua::Lua::new();
    let code = compiler("example/basic_teal_file");
    let res: u8 = lua.load(&code).set_name("embedded_compiler")?.eval()?;

    return Ok::<(), Box<dyn std::error::Error>>(());
};
```
There are a few sources tealr can use to get the compiler. If no source is specified it defaults to github releases.
Other sources can be specified as follows:
```rust ignore
//get the teal compiler using the given path
embed_compiler!(Local(path = "some/path/to/tl.tl"));
//this uses luarocks to try and discover the location of the compiler
embed_compiler!(Local());
//download the compiler at compile time from github (default) 
embed_compiler!(GitHub(version = "v0.13.1"));
//download the compiler at compile time from luarocks
embed_compiler!(Luarocks(version = "v0.13.1"));
```

You can find longer ones with comments on what each call does [here](https://github.com/lenscas/tealr/tree/master/tealr/examples)

## Future plans
Tealr can already help with 2 ways to run teal scripts.

It can compile inline teal code at the same time as your rust code

It can also embed the teal compiler for you, allowing you to execute external teal scripts like normal lua scripts.

There is a third method I want tealr to help with. In this mode, it will compile a teal project, pack it into 1 file and embed it into the project.