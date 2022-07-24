# Changelog

All notable changes to this project are documented in this file.

## Overview
- [`0.8.0`](#080)
- [`0.6.0-preview1`](#060-preview1)
- [`0.5.1`](#051)
- [`0.5.0`](#050)
- [`0.4.0`](#040)
- [`0.2.0`](#020)
- [`0.1.1`](#011)

## upcoming

## 0.9.0
- Move away from syn to use venial instead for most macro's.
- Macros now accept the attribute `[tealr(tealr_name = "some_name")]` to tell macro's what name to use instead of tealr
- Add RluaFromTo and MluaFromTo macro. 
## 0.8.0
- Update macros to work with tealr changes
## 0.6.0-preview1
- Make derive macros bit more hygienic (last time I missed some)
- **BREAKING** rename `UserData` derive macro to `RluaUserData`. This is to have good support for Mlua
- **BREAKING** rename `TealDerive` derive macro to `RluaTealDerive`. This is to have good support for Mlua
- Add `MluaUserData` and `MluaTealDerive` derive macro's.
## 0.5.1
- Fix embed_compiler(Local()) not finding teal if it was installed using --local
## 0.5.0

## 0.4.0
- Make the UserData derive macro a bit more hygienic
- Automatically implement `TypeBody` when implementing `UserData`
## 0.2.0
- Make derive macros bit more hygienic
- Add derive macro for `TypeRepresentation`
- Add derive macro that implements both `rlua::UserData` and `TypeRepresentation`

## 0.1.1
- Add derive macro to implement `rlua::UserData`
