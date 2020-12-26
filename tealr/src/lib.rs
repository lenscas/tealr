#![warn(missing_docs)]
//!# tealr
//!A wrapper around [rlua](https://crates.io/crates/rlua) to help with embedding teal
//!
//!tealr adds some traits that replace/extend those from [rlua](https://crates.io/crates/rlua), allowing it to generate the `.d.tl` files needed for teal.
//!
//!## Small example
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
//!    println!("{}\n ", file_contents);
//!    Ok(())
//!}
//!```
//!You can find longer ones [here](https://github.com/lenscas/tealr/tree/master/tealr/examples)
//!
//!## Future plans
//!Its possible for lua to load .tl files directly after it loaded the compiler. I would like to make use of this and expose methods that already perpare the lua vm in this way.
//!
//!This should make it pretty much as easy to work with teal as with lua. However, I am not sure if doing this breaks any rules from rlua. As such, some research is required.

pub use rlua::UserData;
pub use teal_data::{TealData, TypeRepresentation};
pub use teal_data_methods::TealDataMethods;
pub use teal_multivalue::{TealMultiValue, TealType};
pub use type_walker::TypeWalker;
pub use user_data_wrapper::UserDataWrapper;

#[cfg(feature = "derive")]
pub use tealr_derive::{TealDerive, TypeRepresentation, UserData};

#[cfg(feature = "compile")]
pub use tealr_derive::compile_inline_teal;

mod teal_data;
mod teal_data_methods;
mod teal_multivalue;
mod type_walker;
mod user_data_wrapper;
