# tealr
A wrapper around [rlua](https://crates.io/crates/rlua) to help with embedding teal

tealr adds some traits that replace/extend those from [rlua](https://crates.io/crates/rlua), allowing it to generate the `.d.tl` files needed for teal.
It also contains some macro's to make it easier to load/execute teal scripts. Without having to compile them yourself first.

## Expose a value to teal
Exposing types to teal as userdata is almost the same using tealr as it is using rlua
```rust
#[derive(Clone, UserData, TypeName)]
struct Example {}

//now, implement TealData. This tells rlua what methods are available and tealr what the types are
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
```
## Create a .d.tl file
```rust
let file_contents = TypeWalker::new() 
    .process_type::<Example>(tealr::Direction::ToLua)
    .generate_global("test")
    .expect("oh no :(");
    
println!("{}",file_contents)
```
## Compile inline teal code into lua
```rust
let code = compile_inline_teal!("local x : number = 5 return x");
```

## Embed the teal compiler, run teal files as if they where lua
```rust
let compiler = embed_compiler!("v0.9.0");
let res = rlua::Lua::new().context(|ctx| {
    let code = compiler("example/basic_teal_file");
    let res: u8 = ctx.load(&code).set_name("embedded_compiler")?.eval()?;
    Ok(res)
})?;
```

You can find longer ones with comments on what each call does [here](https://github.com/lenscas/tealr/tree/master/tealr/examples)

## Future plans
Tealr can already help with 2 ways to run teal scripts.
It can compile inline teal code at the same time as your rust code
It can also embed the teal compiler for you, allowing you to execute external teal scripts like normal lua scripts.
There is a third method I want tealr to help with. In this mode, it will compile a teal project, pack it into 1 file and embed it into the project.
