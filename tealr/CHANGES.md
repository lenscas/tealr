# Changelog

All notable changes to this project are documented in this file.

## Overview
- [`0.2.0`](#020)
- [`0.1.1`](#011)
- [`0.1.0`](#010)
- [`0.0.1`](#001)

## upcomming

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