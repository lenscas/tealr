# Changelog

All notable changes to this project are documented in this file.

## Overview
- [`0.3.0`](#030)
- [`0.2.0`](#020)
- [`0.1.1`](#011)
- [`0.1.0`](#010)
- [`0.0.1`](#001)

## upcomming
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