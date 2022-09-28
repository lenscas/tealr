#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

///traits and types specific to rlua
#[cfg(feature = "rlua")]
pub mod rlu;

///traits and types specific to mlua
#[cfg(feature = "mlua")]
pub mod mlu;

mod export_instance;
mod exported_function;
mod teal_multivalue;
mod type_generator;
mod type_representation;
mod type_walker;

pub use exported_function::ExportedFunction;
pub use teal_multivalue::{TealMultiValue, TealType};

///Implements [TypeName](crate::TypeName).
///
///`TypeName::get_type_name` will return the name of the rust type.
#[cfg(feature = "derive")]
pub use tealr_derive::TypeName;

pub use type_generator::{EnumGenerator, Field, NameContainer, RecordGenerator, TypeGenerator};
pub use type_representation::{type_parts_to_str, KindOfType, NamePart, TypeBody, TypeName};
pub use type_walker::{GlobalInstance, TypeWalker};

#[cfg(feature = "compile")]
pub use tealr_derive::compile_inline_teal;

#[cfg(any(
    feature = "embed_compiler_from_local",
    feature = "embed_compiler_from_download"
))]
pub use tealr_derive::embed_compiler;

/// Gets the current version of tealr.
///
/// Useful when consuming the .json files to check if it is a supported version
pub fn get_tealr_versions() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
