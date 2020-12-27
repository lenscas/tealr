#![warn(missing_docs)]
//!# tealr
//!A wrapper around [rlua](https://crates.io/crates/rlua) to help with embedding teal
//!
//!tealr adds some traits that replace/extend those from [rlua](https://crates.io/crates/rlua), allowing it to generate the `.d.tl` files needed for teal.
//!
//!It also contains some macro's to make it easier to load/execute teal scripts. Without having to compile them yourself first.
//!## Small example type generation
//!```rust
//!# use rlua::{Lua, Result, UserDataMethods};
//!# use tealr::{TealData, TealDataMethods, TypeWalker, TealDerive,UserData,TypeRepresentation};
//!#[derive(Clone,Copy,TealDerive)]
//!struct Example {}
//!impl TealData for Example {
//!}
//!fn main() -> Result<()> {
//!    let file_contents = TypeWalker::new()
//!        .proccess_type::<Example>()
//!        .generate_global("test")
//!        .expect("oh no :(");
//!    //save the file
//!    println!("{}\n ", file_contents);
//!    Ok(())
//!}
//!```
//!## Compile inline teal code to lua at the same time as your rust code
//!```rust
//!# use tealr::compile_inline_teal;
//!let lua_code = compile_inline_teal!("-- your teal code");
//!```
//!## Embed the teal compiler, allowing you to run external teal files the same way as external lua files.
//!```rust
//!# use tealr::embed_compiler;
//!let compiler = embed_compiler!("v0.9.0");
//!let lua_code_to_run_external_file = compiler("your_teal_file.tl");
//!```
//!You can find longer ones [here](https://github.com/lenscas/tealr/tree/master/tealr/examples)
//!which also go over on how to use the generated lua code.
//!
//!## Future plans
//!Tealr can already help with 2 ways to run teal scripts
//!
//!It can compile inline teal code at the same time as your rust code
//!
//!It can also embed the teal compiler for you, allowing you to execute external teal scripts like normal lua scripts.
//!
//!There is a third method I want tealr to help with. In this mode, it will compile a teal project, pack it into 1 file and embed it into the project.

pub use rlua::UserData;
pub use teal_data::{TealData, TypeRepresentation, TypedFunction};
pub use teal_data_methods::TealDataMethods;
pub use teal_multivalue::{TealMultiValue, TealType};
pub use type_walker::TypeWalker;
pub use user_data_wrapper::UserDataWrapper;

#[cfg(feature = "derive")]
pub use tealr_derive::{TealDerive, TypeRepresentation, UserData};

#[cfg(feature = "compile")]
pub use tealr_derive::compile_inline_teal;

#[cfg(feature = "embed_compiler")]
pub use tealr_derive::embed_compiler;

mod teal_data;
mod teal_data_methods;
mod teal_multivalue;
mod type_walker;
mod user_data_wrapper;
