# Changelog

All notable changes to this project are documented in this file.

## Overview
- [`0.8.0`](#080)
- [`0.7.0`](#070)
- [`0.6.0`](#060)
- [`0.6.0-preview1`](#060-preview1)
- [`0.5.1`](#051)
- [`0.5.0`](#050)
- [`0.4.0`](#040)
- [`0.3.0`](#030)
- [`0.2.0`](#020)
- [`0.1.1`](#011)
- [`0.1.0`](#010)
- [`0.0.1`](#001)

## upcoming
## 0.8.0
- Improved readme
- Add ability to document the api
- Make types serializable with serde
- **BREAKING** removed `Direction`
- **BREAKING** used different way to store type info

## 0.7.0
- Add generic methods/functions
- **BREAKING** `TypeName::is_external` got replaced by `TypeName::get_type_kind`
- `TypeName::collect_children` got added with a default implementation. It is HIGHLY encouraged to implement this if your type is generic.
- `create_generic_rlua!` and `create_generic_mlua!` got added. These macros help to create placeholder types that act as generics
- `create_union_rlua!` and `create_union_mlua!` got added. These macros help to create types that act as a union to teal.
- `TypeWalker::process_type_inline` got added. This acts similar to `Typewalker::process_type` but doesn't put the type in its own child record. Useful when making a module with `mlua`

## 0.6.0
- reexport mlua/rlua
- add reexport of the features that rlua/mlua expose
## 0.6.0-preview1
- Add basic support for Mlua
- Make derive macros bit more hygienic (last time I missed some)
- **BREAKING** rename `UserData` derive macro to `RluaUserData`. This is to have good support for Mlua
- **BREAKING** rename `TealDerive` derive macro to `RluaTealDerive`. This is to have good support for Mlua
- **BREAKING** `rlua` is now an optional dependency, disabled by default.
- Add `MluaUserData` and `MluaTealDerive` derive macro's.
- Add `mlu` module which contains the structs and traits needed for mlua support.
- Add `mlua` and `rlua` feature flags
## 0.5.1
- Fix embed_compiler(Local()) not finding teal if it was installed using --local
## 0.5.0
- Add support for `metaMethods`
- **BREAKING** Update minimum supported tl version to `0.13.1`
- Add support for `integer` type

## 0.4.0
- **BREAKING** mark the generated record types as UserData, this is automatically done when using the derives, limiting the generated types to teal 0.10.0 and higher

- **BREAKING** add an abstraction layer between generating types and TealData. This is to support rlua::ToLua and rlua::FromLua

- **BREAKING** When generating types make a distinction if it is a lua value to rust, or rust to lua. This is to support rlua::ToLua and rlua::FromLua

- **BREAKING** Rename `TypeRepresentation` to `TypeName` to better reflect what it about.
## 0.3.0
- Macro to compile inline teal code at the same time as your rust code
- Macro to embed the teal compiler into your application. Allowing you to execute external teal files as normal lua files.
- wrapper for rlua::Function, so you can better define the types of functions you return/need as a parameter.
## 0.2.0
- **BREAKING** Allow the scope of the container type to be either `local` or `global`
- Add some more types that tealr recognizes.
- **BREAKING** split `TealData` up into `TealData` and `TypeRepresentation`
- **BREAKING** change return type of TypeRepresentation::get_type_name to `Cow<'static, str>` (was `String`)

## 0.1.1
- Fix problems with documentation
- Add derive macro to implement `rlua::UserData`

## 0.1.0
- Implement generation of `.d.tl` files
- Improve amount of types that implement `TealrData`

## 0.0.1
- Fist release!