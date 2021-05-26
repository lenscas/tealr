#![warn(missing_docs)]
//!# tealr
//!A wrapper around [rlua](https://crates.io/crates/rlua) and/or [mlua](https://crates.io/crates/mlua) to help with embedding teal
//!
//!tealr adds some traits that replace/extend those from [rlua](https://crates.io/crates/rlua) and [mlua](https://crates.io/crates/mlua),
//!allowing it to generate the `.d.tl` files needed for teal.
//!It also contains some macro's to make it easier to load/execute teal scripts.
//!
//!### Note:
//!Both rlua and mlua are behind feature flags and both feature flags can be enabled at the same time.
//!
//!## Expose a value to teal
//!Exposing types to teal as userdata is almost the same using tealr as it is using rlua and mlua
//!```rust
//!# use tealr::{RluaUserData,MluaUserData,TypeName};
//!#[derive(Clone, RluaUserData, TypeName)]
//!struct ExampleRlua {}
//!
//!//now, implement rlu::TealData.
//!//This tells rlua what methods are available and tealr what the types are
//!impl tealr::rlu::TealData for ExampleRlua {
//!    //implement your methods/functions
//!    fn add_methods<'lua, T: tealr::rlu::TealDataMethods<'lua, Self>>(methods: &mut T) {
//!        methods.add_method("example_method", |_, _, x: i8| Ok(x));
//!        methods.add_method_mut("example_method_mut", |_, _, x: (i8, String)| Ok(x.1));
//!        methods.add_function("example_function", |_, x: Vec<String>| Ok((x, 8)));
//!        methods.add_function_mut("example_function_mut", |_, x: (bool, Option<ExampleRlua>)| {
//!            Ok(x)
//!        })
//!    }
//!}
//!//Working with Mlua is pretty much the same
//!#[derive(Clone, MluaUserData, TypeName)]
//!struct ExampleMlua {}
//!impl tealr::mlu::TealData for ExampleMlua {
//!    //implement your methods/functions
//!    fn add_methods<'lua, T: tealr::mlu::TealDataMethods<'lua, Self>>(methods: &mut T) {
//!        methods.add_method("example_method", |_, _, x: i8| Ok(x));
//!        methods.add_method_mut("example_method_mut", |_, _, x: (i8, String)| Ok(x.1));
//!        methods.add_function("example_function", |_, x: Vec<String>| Ok((x, 8)));
//!        methods.add_function_mut("example_function_mut", |_, x: (bool, Option<ExampleMlua>)| {
//!            Ok(x)
//!        })
//!    }
//!}
//!```
//!## Create a .d.tl file
//! Creating of the `.d.tl` files works the same for rlua or mlua
//!```rust
//!# use tealr::{TypeWalker,TypeName,RluaUserData,rlu::TealData};
//!# #[derive(RluaUserData,TypeName)]
//!# struct Example {}
//!# impl TealData for Example {};
//!let file_contents = TypeWalker::new()
//!    .process_type::<Example>(tealr::Direction::ToLua)
//!    .generate_global("test")
//!    .expect("oh no :(");
//!
//!println!("{}",file_contents)
//!```
//!## Compile inline teal code into lua
//! As you get a string containing the lua code back this feature works the same for both rlua and mlua
//!```rust
//! # use tealr::compile_inline_teal;
//!let code = compile_inline_teal!("local x : number = 5 return x");
//!```
//!
//!## Embed the teal compiler, run teal files as if they where lua
//!### Rlua:
//!```no_run
//!# use tealr::embed_compiler;
//!let compiler = embed_compiler!("v0.10.0");
//!let res = rlua::Lua::new().context(|ctx| {
//!    let code = compiler("example/basic_teal_file");
//!    let res: u8 = ctx.load(&code).set_name("embedded_compiler")?.eval()?;
//!    Ok(res)
//!})?;
//!# Ok::<(), rlua::Error>(())
//!```
//!### Mlua:
//!```no_run
//!# use tealr::embed_compiler;
//!let compiler = embed_compiler!("v0.10.0");
//!let lua = mlua::Lua::new();
//!let code = compiler("example/basic_teal_file");
//!let res: u8 = lua.load(&code).set_name("embedded_compiler")?.eval()?;
//!
//!# Ok::<(), mlua::Error>(())
//!```
//!You can find longer ones with comments on what each call does [here](https://github.com/lenscas/tealr/tree/master/tealr/examples)
//!
//!## Future plans
//!Tealr can already help with 2 ways to run teal scripts.
//!It can compile inline teal code at the same time as your rust code
//!It can also embed the teal compiler for you, allowing you to execute external teal scripts like normal lua scripts.
//!There is a third method I want tealr to help with. In this mode, it will compile a teal project, pack it into 1 file and embed it into the project.

///traits and types specific to rlua
#[cfg(feature = "rlua")]
pub mod rlu;

///traits and types specific to mlua
#[cfg(feature = "mlua")]
pub mod mlu;

mod exported_function;
mod teal_multivalue;
mod type_generator;
mod type_representation;
mod type_walker;

pub use exported_function::ExportedFunction;
pub use teal_multivalue::{TealMultiValue, TealType};
pub use type_generator::TypeGenerator;
pub use type_representation::{Direction, TypeBody, TypeName};
pub use type_walker::TypeWalker;

#[cfg(feature = "derive")]
pub use tealr_derive::{MluaTealDerive, MluaUserData, RluaTealDerive, RluaUserData, TypeName};

#[cfg(feature = "compile")]
pub use tealr_derive::compile_inline_teal;

#[cfg(any(
    feature = "embed_compiler_from_local",
    feature = "embed_compiler_from_download"
))]
pub use tealr_derive::embed_compiler;
