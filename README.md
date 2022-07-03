# tealr
A crate to enhance the APIs provided by the [rlua](https://crates.io/crates/rlua) and [mlua](https://crates.io/crates/mlua) crates


It aims to do this by improving the following:
- Allow the api to have easily accessible documentation embedded into it
- Allow the documentation to be built to web pages (using [tealr_doc_gen](https://github.com/lenscas/tealr_doc_gen) )
- To go along with the documentation, `tealr` also allow you to be more precise in the types your api works with. Think generic methods and typed lambdas. No more `Lua::Value`
- Allow for the generation of `.d.tl` files. They allow the API to be used in teal, exposing type errors at compile time.

It does this by adding new traits and replacing/extending the existing ones from [rlua](https://crates.io/crates/rlua) and [mlua](https://crates.io/crates/mlua). As a result, the api that tealr exposes is as similar as the api from those 2 crates as possible.

It also contains some macro's to easily generate new types to better express the API type wise and also some macro's to make it easier to embed teal.

## Example of `instance.help()`

The library shown is <https://github.com/lenscas/tealsql>

![<https://github.com/lenscas/tealr/tree/master/tealr/images/help_example.gif>](https://raw.githubusercontent.com/lenscas/tealr/master/images/help_example.gif)

## html rendered documentation
Rendered html is also available at <https://lenscas.github.io/tealsql/>
## Note:
Both `rlua` and `mlua` are behind the feature flags `rlua` and `mlua`.

It also reexports these crates and allows you to set flags through it (the forwarded flags are the same with either the prefix `rlua_` or `mlua_`. For example if you want to enable `mlua/async` then you need to enable `tealr/mlua_async`)

## Expose a value to lua/teal
Exposing types to lua as userdata is almost the same using tealr as it is using rlua and mlua

#### Rlua:
```rust ignore
use tealr::TypeName;
#[derive(Clone, tealr::rlu::UserData, TypeName)]
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
        methods.add_function_mut("example_function_mut", |_, x: (bool, Option<ExampleRlua>)| {
            Ok(x)
        });
        ///This creates the instance.help() function, which returns the documentation as a string.
        methods.generate_help();
    }
}
```
Mlua:
```rust ignore
use tealr::TypeName;
#[derive(Clone, tealr::mlu::UserData, TypeName)]
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
        methods.add_function_mut("example_function_mut", |_, x: (bool, Option<ExampleMlua>)| {
            Ok(x)
        });
        ///This creates the instance.help() function, which returns the documentation as a string.
        methods.generate_help();
    }
}
```

## Replacing lua::Value
Though it is perfectly possible to use the `lua::Value` from `rlua` and `mlua`, `tealr` does offer some alternatives to better document your code.

### Simple unions:

These allow you to easily create a type that is only one of the types you give.

```rust ignore
use tealr::{
    create_union_mlua,
};
create_union_mlua!(enum YourTypeName = i32 | String);
```

### Generics
```rust ignore
use mlua::ToLua;
use tealr::{
    create_generic_mlua,
    mlu::{mlua::FromLua, TealData, TealDataMethods, TypedFunction,UserData},
    TypeName, TypeWalker,
};

create_generic_mlua!(X);
#[derive(Clone, UserData, TypeName)]
struct Example {}
impl TealData for Example {
    fn add_methods<'lua, T: TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method(
            "generic_function_callback",
            |lua, _, fun: TypedFunction<String, X>| {
                fun.call("A nice string!".to_string())
            },
        );
    }
}
```
In this example, the generated signature of `generic_function_callback` will be `function<X>(function(string):X):X`. Without generics and using `lua::Value` instead, the generated signature would have instead become `function(function(string):any):any` which is a lot less descriptive.

For rlua, all you have to do is replace `mlua` for `rlua`

## Teal integration

The [teal](https://github.com/teal-language/tl) language is basically just a statically typed variant of lua and can even be made to run in the lua vm without compiling to lua first.

As a result of this and `tealr`'s focus on enabling a richer typed api causes the 2 projects to work well together. However, to further help bind the 2 projects, `tealr` contains some extra helpers for those that want to use teal.

### Create a .d.tl file
`.d.tl` files are type definition files used by the teal compiler so it knows what types a library has that isn't written in teal. Similar to how `.d.ts` files allow typescript to work with libraries written in `JS`

`tealr` allows you to easily create these `.d.tl` files based on your types.
```rust
//set your type up with either rlua or mlua
use tealr::{TypeName};
#[cfg(feature = "mlua")]
use tealr::mlu::{TealData,UserData};
#[cfg(feature = "rlua")]
use tealr::rlu::{TealData,UserData};
#[derive(UserData, TypeName)]
#[derive(UserData, TypeName)]
struct Example {}
impl TealData for Example {};

//time to create the type definitions
let file_contents = tealr::TypeWalker::new()
    .process_type::<Example>()
    .generate_global("test")
    .expect("oh no :(");

//write the output to a file
println!("{}",file_contents)
```
### Compile inline teal code into lua
Both rlua and mlua allow you to run lua code embedded in your application.

Similarly, tealr allows you to compile embedded teal code to lua while compiling your application. This can then be executed by rlua and mlua.

This means that you can make use of teal's static type system even for small scripts inside your rust codebase.

```rust
use tealr::compile_inline_teal;
let code = compile_inline_teal!("local x : number = 5 return x");
```

## Embed the teal compiler

Teal makes it possible for the lua vm to load teal files as if they are normal lua files.

Tealr makes doing this from withing rust a bit easier, by exposing a macro that can embed the teal compiler in your application and create a function that creates the needed lua code to set the VM up. This function takes a string, which is the file that needs to get required.


```rust no_run
use tealr::embed_compiler;
let compiler = embed_compiler!("v0.13.1");
#[cfg(feature = "rlua")]
{
    let res : u8 = tealr::rlu::rlua::Lua::new().context(|ctx| {
        let code = compiler("example/basic_teal_file");
        ctx.load(&code).set_name("embedded_compiler")?.eval()
    })?;
};

#[cfg(feature = "mlua")]
{
    let code = compiler("example/basic_teal_file");
    let lua = tealr::mlu::mlua::Lua::new();
    let res: u8 = lua.load(&code).set_name("embedded_compiler")?.eval()?;
};
Ok::<(), Box<dyn std::error::Error>>(())
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
