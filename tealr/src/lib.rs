#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

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
#[cfg(feature = "derive")]
pub use tealr_derive::{MluaTealDerive, MluaUserData, RluaTealDerive, RluaUserData, TypeName};
pub use type_generator::{NameContainer, TypeGenerator};
pub use type_representation::{type_parts_to_str, KindOfType, NamePart, TypeBody, TypeName};
pub use type_walker::TypeWalker;

#[cfg(feature = "compile")]
pub use tealr_derive::compile_inline_teal;

#[cfg(any(
    feature = "embed_compiler_from_local",
    feature = "embed_compiler_from_download"
))]
pub use tealr_derive::embed_compiler;

/// Implements the needed traits to make this trait convertible to and from lua values.
/// It does this differently for structs and enums but will implement the [TypeBody](crate::TypeBody) trait in both cases.
/// # Structs
/// Structs implement the [FromLua](mlua::FromLua) and [ToLua](mlua::ToLua) directly.
/// These trait implementations convert the struct directly to and from a table. This table contains every filed INCLUDING private fields.
/// ## Attributes
/// ### Field attributes
/// - `remote`: used as `#[tealr(remote = OtherType)]`
///     
///     Allows you to specify that a given field should be converted to and from `OtherType` before passing and receiving it to and from lua.
///     This is done using the [From<T>](std::convert::From) trait.
/// # Warning:
/// Using this macro on structs WILL make any private fields freely accessible to lua.
///
/// ## Example
///
///```rust
///  use tealr::{MluaFromToLua,TypeName,mlu::mlua::Lua};
///  #[derive(MluaFromToLua,Clone,TypeName)]
///  struct Example {
///     test_field: String
///  }
///  impl From<String> for Example {
///     fn from(t : String) -> Self {
///         Example{test_field:t}
///     }
///  }
///  impl From<Example> for String {
///     fn from(t: Example) -> Self {
///         t.test_field
///     }
///  }
///  #[derive(MluaFromToLua,Clone,TypeName)]
/// struct Example2 {
///     #[tealr(remote = Example)]
///     field1: String
/// }
/// let instance = Example2 {field1:"amazing".into()};
/// let lua = Lua::new();
/// let mut globals = lua.globals();
/// globals.set("instance",instance).unwrap();
/// let code = "
///     assert(instance.field1.test_field == \"amazing\")
///     instance.field1.test_field = \"new_value\"
///     return instance
/// ";
/// let res: Example2 = lua.load(code).set_name("MluaToFromLuaStruct").unwrap().eval().unwrap();
/// assert_eq!(res.field1,"new_value");
///
///```
/// # Enums
/// Right now only tuple enums or enums without inner values are supported.
/// In both cases it works by implementing [TealData](crate::mlu::TealData) and [UserData](mlua::UserData).
///
/// For every variant with inner values 3 methods get added to the [TealData](crate::mlu::TealData). These are:
///
///  - `Is{VariantName}`
///     
///     Returns true if the underlying enum is of that variant
///  - `Get{VariantName}`,
///     
///     Returns `true` and the `inner value` if the enum is of the given variant. Else it returns `false` and `nil`
///  - `Get{VariantName}OrNil`
///     
///     Returns the `inner value` if the enum is of the given variant. Else it returns `nil`
///
/// For variants that don't have inner values only the `Is{VariantName}` method gets generated.
///
/// Custom methods can be added using the `extend_methods` attribute.
/// Similarly, the `extend_fields` attribute can be used to extend the fields that this TealData exposes.
///
/// In addition to the above mentioned traits, it also creates a new zero sized struct. This can be exposed to lua so new instances of this enum can be made while inside lua.
/// By default this struct has the name `{EnumName}Creator`, but this can be changed using the `creator_name` attribute.
/// This struct implements [Clone], [TealData](crate::mlu::TealData), [UserData](mlua::UserData), [TypeName](crate::TypeName) and [TypeBody](crate::TypeBody)
/// and has the same visibility as the original enum.
///
/// The [TealData](crate::mlu::TealData) of this struct exposes the functions:
///  - `New{VariantName}From`
///     
///     This function only gets generated if the variant contains inner values.
///     
///     It takes the needed values to construct this variant in the same order as it was defined and returns a new instance of the variant.
///
///  - `New{VariantName}`
///     
///     This function only gets generated if the variant has no inner values.
///     
///     It returns a new instance of this enum of the given variant.
/// ## Attributes
/// ### type level attributes
///  - `extend_fields` : Used as `#[tealr(extend_fields = function_name)]`
///
///     calls the given function when adding fields to the [TealData](crate::mlu::TealData) of the enum
///
///  - `extend_methods` : Used as `#[tealr(extend_methods = function_name)]`
///
///     calls the given function when adding methods to the [TealData](crate::mlu::TealData) of the enum
///
/// - `creator_name` : Used as ``#[tealr(creator_name = NewTypeForCreatorType)]`
///
///     Uses the given name for the enum creator struct
///
///  ### Field level attributes
///
///  - `remote`: used as `#[tealr(remote = OtherType)]`
///    
///     Allows you to specify that a given field should be converted to and from `OtherType` before passing and receiving it to and from lua.
///     This is done using the [From<T>](std::convert::From) trait.
/// ## Example
///
///```rust
/// use tealr::{MluaFromToLua,TypeName,mlu::{TealData,mlua::Lua}};
/// #[derive(MluaFromToLua,Clone,TypeName)]
/// struct ExampleStruct {
///     test_field: String
///  }
///  impl From<String> for ExampleStruct {
///     fn from(t : String) -> Self {
///         ExampleStruct{test_field:t}
///     }
///  }
///  impl From<ExampleStruct> for String {
///     fn from(t: ExampleStruct) -> Self {
///         t.test_field
///     }
///  }
/// #[derive(MluaFromToLua,Clone,TypeName)]
/// #[tealr(creator_name = ExampleMaker)]
/// #[tealr(extend_fields = field_extension)]
/// #[tealr(extend_methods = method_extension)]
/// enum Example {
///     NoInnerValue,
///     SingularInnerValue(
///         #[tealr(remote = ExampleStruct)]
///         String
///     ),
///     DoubleInnerValue(String,u8)
/// }
/// fn field_extension<'lua,B:TypeName + TealData,A: tealr::mlu::TealDataFields<'lua,B>>(fields: &mut A) {
///     //set fields as usual
/// }
/// fn method_extension<'lua,B:TypeName,A: tealr::mlu::TealDataMethods<'lua,B>>(fields: &mut A) {
///     //set methods as usual
/// }
/// let instance = Example::SingularInnerValue("SomeValue".into());
/// let lua = Lua::new();
/// let mut globals = lua.globals();
/// globals.set("instance",instance).unwrap();
/// globals.set("ExampleCreator",ExampleMaker::new()).unwrap();
/// let code = "
///
///     assert(instance:IsSingularInnerValue())
///     assert(not instance:IsNoInnerValue())
///     assert(instance:GetSingularInnerValueOrNil().test_field == \"SomeValue\")
///     return ExampleCreator.NewDoubleInnerValueFrom(\"some_new_value\",2)
///     
/// ";
/// let res: Example = lua.load(code).set_name("MluaToFromLuaEnum").unwrap().eval().unwrap();
/// assert!(matches!{Example::DoubleInnerValue("some_new_value".to_string(),5),res});
///```
///
#[cfg(all(feature = "derive", feature = "mlua"))]
pub use tealr_derive::MluaFromToLua;

/// Implements the needed traits to make this trait convertible to and from lua values.
/// It does this differently for structs and enums but will implement the [TypeBody](crate::TypeBody) trait in both cases.
/// # Structs
/// Structs implement the [FromLua](rlua::FromLua) and [ToLua](rlua::ToLua) directly.
/// These trait implementations convert the struct directly to and from a table. This table contains every filed INCLUDING private fields.
/// ## Attributes
/// ### Field attributes
/// - `remote`: used as `#[tealr(remote = OtherType)]`
///     
///     Allows you to specify that a given field should be converted to and from `OtherType` before passing and receiving it to and from lua.
///     This is done using the [From<T>](std::convert::From) trait.
/// # Warning:
/// Using this macro on structs WILL make any private fields freely accessible to lua.
///
/// ## Example
///
///```rust
///use tealr::{RluaFromToLua,TypeName,rlu::rlua::Lua};
///#[derive(RluaFromToLua,Clone,TypeName)]
///struct Example {
///   test_field: String
///}
///impl From<String> for Example {
///   fn from(t : String) -> Self {
///       Example{test_field:t}
///   }
///}
///impl From<Example> for String {
///   fn from(t: Example) -> Self {
///       t.test_field
///   }
///}
///#[derive(RluaFromToLua,Clone,TypeName)]
///struct Example2 {
///    #[tealr(remote = Example)]
///    field1: String
///}
///let lua = Lua::new();
///lua.context(|lua| {
///    let instance = Example2 {field1:"amazing".into()};
///    let globals = lua.globals();
///     globals.set("instance",instance).unwrap();
///    let code = "
///        assert(instance.field1.test_field == \"amazing\")
///        instance.field1.test_field = \"new_value\"
///        return instance
///    ";
///    let res: Example2 = lua.load(code).set_name("RluaToFromLuaStruct").unwrap().eval().unwrap();
///    assert_eq!(res.field1,"new_value");
///});
///```
/// # Enums
/// Right now only tuple enums or enums without inner values are supported.
/// In both cases it works by implementing [TealData](crate::rlu::TealData) and [UserData](rlua::UserData).
///
/// For every variant with inner values 3 methods get added to the [TealData](crate::rlu::TealData). These are:
///
///  - `Is{VariantName}`
///     
///     Returns true if the underlying enum is of that variant
///  - `Get{VariantName}`,
///     
///     Returns `true` and the `inner value` if the enum is of the given variant. Else it returns `false` and `nil`
///  - `Get{VariantName}OrNil`
///     
///     Returns the `inner value` if the enum is of the given variant. Else it returns `nil`
///
/// For variants that don't have inner values only the `Is{VariantName}` method gets generated.
///
/// Custom methods can be added using the `extend_methods` attribute.
/// Similarly, the `extend_fields` attribute can be used to extend the fields that this TealData exposes.
///
/// In addition to the above mentioned traits, it also creates a new zero sized struct. This can be exposed to lua so new instances of this enum can be made while inside lua.
/// By default this struct has the name `{EnumName}Creator`, but this can be changed using the `creator_name` attribute.
/// This struct implements [Clone], [TealData](crate::rlu::TealData), [UserData](rlua::UserData), [TypeName](crate::TypeName) and [TypeBody](crate::TypeBody)
/// and has the same visibility as the original enum.
///
/// The [TealData](crate::rlu::TealData) of this struct exposes the functions:
///  - `New{VariantName}From`
///     
///     This function only gets generated if the variant contains inner values.
///     
///     It takes the needed values to construct this variant in the same order as it was defined and returns a new instance of the variant.
///
///  - `New{VariantName}`
///     
///     This function only gets generated if the variant has no inner values.
///     
///     It returns a new instance of this enum of the given variant.
/// ## Attributes
/// ### type level attributes
///  - `extend_fields` : Used as `#[tealr(extend_fields = function_name)]`
///
///     calls the given function when adding fields to the [TealData](crate::rlu::TealData) of the enum
///
///  - `extend_methods` : Used as `#[tealr(extend_methods = function_name)]`
///
///     calls the given function when adding methods to the [TealData](crate::rlu::TealData) of the enum
///
/// - `creator_name` : Used as ``#[tealr(creator_name = NewTypeForCreatorType)]`
///
///     Uses the given name for the enum creator struct
///
///  ### Field level attributes
///
///  - `remote`: used as `#[tealr(remote = OtherType)]`
///    
///     Allows you to specify that a given field should be converted to and from `OtherType` before passing and receiving it to and from lua.
///     This is done using the [From<T>](std::convert::From) trait.
/// ## Example
///
///```rust
/// use tealr::{RluaFromToLua,TypeName,rlu::{TealData,rlua::Lua}};
/// #[derive(RluaFromToLua,Clone,TypeName)]
/// struct ExampleStruct {
///     test_field: String
///  }
///  impl From<String> for ExampleStruct {
///     fn from(t : String) -> Self {
///         ExampleStruct{test_field:t}
///     }
///  }
///  impl From<ExampleStruct> for String {
///     fn from(t: ExampleStruct) -> Self {
///         t.test_field
///     }
///  }
/// #[derive(RluaFromToLua,Clone,TypeName)]
/// #[tealr(creator_name = ExampleMaker)]
/// #[tealr(extend_methods = method_extension)]
/// enum Example {
///     NoInnerValue,
///     SingularInnerValue(
///         #[tealr(remote = ExampleStruct)]
///         String
///     ),
///     DoubleInnerValue(String,u8)
/// }
/// fn method_extension<'lua,B:TypeName,A: tealr::rlu::TealDataMethods<'lua,B>>(fields: &mut A) {
///     //set methods as usual
/// }
/// let instance = Example::SingularInnerValue("SomeValue".into());
/// let lua = Lua::new();
/// lua.context(|lua|{
///     let mut globals = lua.globals();
///     globals.set("instance",instance).unwrap();
///     globals.set("ExampleCreator",ExampleMaker::new()).unwrap();
///     let code = "
///
///         assert(instance:IsSingularInnerValue())
///         assert(not instance:IsNoInnerValue())
///         assert(instance:GetSingularInnerValueOrNil().test_field == \"SomeValue\")
///         return ExampleCreator.NewDoubleInnerValueFrom(\"some_new_value\",2)
///         
///     ";
///     let res: Example = lua.load(code).set_name("RluaToFromLuaEnum").unwrap().eval().unwrap();
///     assert!(matches!{Example::DoubleInnerValue("some_new_value".to_string(),5),res});
///});
///```
///
#[cfg(all(feature = "derive", feature = "rlua"))]
pub use tealr_derive::RluaFromToLua;
