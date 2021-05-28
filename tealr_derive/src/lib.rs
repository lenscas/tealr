#![warn(missing_docs)]
//!# Tealr_derive
//!The derive macro used by [tealr](https://github.com/lenscas/tealr/tree/master/tealr).
//!
//!Tealr is a crate that can generate `.d.tl` files for types that are exposed to `lua`/`teal` through [rlua](https://crates.io/crates/rlua)
//!
//!Read the [README.md](https://github.com/lenscas/tealr/tree/master/tealr/README.md) in [tealr](https://github.com/lenscas/tealr/tree/master/tealr) for more information.

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

#[cfg(any(
    feature = "embed_compiler_from_local",
    feature = "embed_compiler_from_download"
))]
mod embed_compiler;

use std::{
    ffi::OsStr,
    fs::{read_to_string, File},
    io::Write,
    path::PathBuf,
    process::Command,
};

#[cfg(any(
    feature = "embed_compiler_from_local",
    feature = "embed_compiler_from_download"
))]
use embed_compiler::EmbedOptions;
use proc_macro::TokenStream;
use syn::{parse::Parse, parse_macro_input, LitStr, Token};

///Implements [rlua::UserData](rlua::UserData) and `tealr::TypeBody`
///
///It wraps the [rlua::UserDataMethods](rlua::UserDataMethods) into `tealr::rlu::UserDataWrapper`
///and then passes it to `tealr::rlu::TealData::add_methods`.
///
///Type body is implemented in a similar way, where it uses the `tealr::TealData` implementation to get the types
#[cfg(feature = "derive")]
#[proc_macro_derive(RluaUserData)]
pub fn rlua_user_data_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_rlua_user_data_derive(&ast).into()
}

fn impl_rlua_user_data_derive(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl rlua::UserData for #name {
            fn add_methods<'lua, T: ::rlua::UserDataMethods<'lua, Self>>(methods: &mut T) {
                let mut x = ::tealr::rlu::UserDataWrapper::from_user_data_methods(methods);
                <Self as ::tealr::rlu::TealData>::add_methods(&mut x);
            }
        }
        impl ::tealr::TypeBody for #name {
            fn get_type_body(_: ::tealr::Direction, gen: &mut ::tealr::TypeGenerator) {
                gen.is_user_data = true;
                <Self as ::tealr::rlu::TealData>::add_methods(gen);
            }
        }
    };
    gen
}

///Implements [mlua::UserData](mlua::UserData) and `tealr::TypeBody`
///
///It wraps the [mlua::UserDataMethods](mlua::UserDataMethods) into `tealr::mlu::UserDataWrapper`
///and then passes it to `tealr::TealData::add_methods`.
///
///Type body is implemented in a similar way, where it uses the `tealr::mlu::TealData` implementation to get the types
#[cfg(feature = "derive")]
#[proc_macro_derive(MluaUserData)]
pub fn mlua_user_data_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_mlua_user_data_derive(&ast).into()
}

fn impl_mlua_user_data_derive(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl mlua::UserData for #name {
            fn add_methods<'lua, T: ::mlua::UserDataMethods<'lua, Self>>(methods: &mut T) {
                let mut x = ::tealr::mlu::UserDataWrapper::from_user_data_methods(methods);
                <Self as ::tealr::mlu::TealData>::add_methods(&mut x);
            }
        }
        impl ::tealr::TypeBody for #name {
            fn get_type_body(_: ::tealr::Direction, gen: &mut ::tealr::TypeGenerator) {
                gen.is_user_data = true;
                <Self as ::tealr::mlu::TealData>::add_methods(gen);
            }
        }
    };
    gen
}

///Implements `tealr::TypeName`.
///
///`TypeName::get_type_name` will return the name of the rust type.
#[cfg(feature = "derive")]
#[proc_macro_derive(TypeName)]
pub fn type_representation_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_type_representation_derive(&ast).into()
}
fn impl_type_representation_derive(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl ::tealr::TypeName for #name {
            fn get_type_name(_: ::tealr::Direction) -> ::std::borrow::Cow<'static, str> {
                ::std::borrow::Cow::from(stringify!(#name))
            }
        }
    };
    gen
}

///Implement both [rlua::UserData](rlua::UserData) and `tealr::TypeName`.
///
///Look at [tealr_derive::RluaUserData](tealr_derive::RluaUserData) and [tealr_derive::TypeName](tealr_derive::TypeName)
///for more information on how the implemented traits will behave.
#[cfg(feature = "derive")]
#[proc_macro_derive(RluaTealDerive)]
pub fn rlua_teal_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let mut stream = impl_type_representation_derive(&ast);
    stream.extend(impl_rlua_user_data_derive(&ast));
    stream.into()
}

///Implement both [mlua::UserData](mlua::UserData) and `tealr::TypeName`.
///
///Look at [tealr_derive::MluaUserData](tealr_derive::MluaUserData) and [tealr_derive::TypeName](tealr_derive::TypeName)
///for more information on how the implemented traits will behave.
#[cfg(feature = "derive")]
#[proc_macro_derive(MluaTealDerive)]
pub fn mlua_teal_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let mut stream = impl_type_representation_derive(&ast);
    stream.extend(impl_mlua_user_data_derive(&ast));
    stream.into()
}

struct CompileInput {
    code: String,
    path: PathBuf,
}
impl Parse for CompileInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let code: String = input.parse::<LitStr>()?.value();
        let has_comma = input.parse::<Option<Token![,]>>()?.is_some();
        let mut path: PathBuf = std::env::var("CARGO_MANIFEST_DIR")
            .expect("Could not get the crate directory")
            .into();

        if has_comma {
            let extra: LitStr = input.parse()?;

            path = path.join(extra.value());
        }

        Ok(Self { code, path })
    }
}

///Compiles the given teal code at compile time to lua.
///
///The macro tries it best to pass the correct `--include-dir` to tl using `CARGO_MANIFEST_DIR`.
///However, this isn't always where you want it to be. In that case you can add an extra argument that will be joined with `CARGO_MANIFEST_DIR` using [std::path::PathBuf::join](std::path::PathBuf#method.join)
///
///## Compile time requirement!
///At this point in time this requires you to have the teal compiler installed and accessible as `tl`.
///
///## Example
///```
///# use tealr_derive::compile_inline_teal;
///assert_eq!(compile_inline_teal!("local a : number = 1\n"),"local a = 1\n")
///```

#[cfg(feature = "compile")]
#[proc_macro]
pub fn compile_inline_teal(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as CompileInput);

    let code = input.code;
    let path = input.path;

    let dir = tempfile::tempdir().expect("Could not create a temporary directory");
    let temp_path = dir.path();
    let mut input_file =
        File::create(temp_path.join("input.tl")).expect("Could not create teal source file");
    input_file
        .write_all(code.as_bytes())
        .expect("Could not write teal source file");

    println!("{:?}", path);

    let mut command = Command::new("tl")
        .args(&[
            OsStr::new("check"),
            OsStr::new("-I"),
            path.as_os_str(),
            OsStr::new("input.tl"),
        ])
        .current_dir(temp_path)
        .spawn()
        .expect("Could not run `tl check`. Make sure it is available in the path");

    if !command
        .wait()
        .expect("Something has gone wrong while running `tl check`")
        .success()
    {
        panic!("There was an error while typechecking your teal code.")
    }

    let mut command = Command::new("tl")
        .args(&[
            OsStr::new("gen"),
            OsStr::new("-o"),
            OsStr::new("output.lua"),
            OsStr::new("-I"),
            path.as_os_str(),
            OsStr::new("input.tl"),
        ])
        .current_dir(temp_path)
        .spawn()
        .expect("Could not run `tl gen`. Make sure it is available in the path");

    if !command
        .wait()
        .expect("Something has gone wrong while running `tl gen`")
        .success()
    {
        panic!("Could not compile teal code.");
    }
    let contents =
        read_to_string(temp_path.join("output.lua")).expect("Could not read generated lua");

    let stream = quote! {#contents};
    stream.into()
}
///Embeds the teal compiler, making it easy to load teal files directly.
///
///It can either download the given version from Github (default), luarocks or uses the compiler already installed on your system
///Compiling it without the lua5.3 compatibility library and embedding it into your application.
///
///It returns a closure that takes the file that needs to run
///and returns valid lua code that both prepares the lua vm so it can run teal files and
///loads the given file using `require`, returning the result of the file that got loaded.
///## NOTE!
///Due to how the teal files are being loaded, they won't be typed checked.
///More info on: [https://github.com/teal-language/tl/blob/master/docs/tutorial.md](https://github.com/teal-language/tl/blob/master/docs/tutorial.md) (Search for "loader")
///
///## Compile time requirement!
///This needs to be able to run `lua` at compile time to compile the teal compiler.
///
///If a local teal compiler is used, then `tl` needs to run at compile time instead.
///
///## Example
///Downloads:
///```rust
///# use tealr_derive::embed_compiler;
/// //This downloads from github
/// let compiler = embed_compiler!("v0.9.0");
///
/// let compiler = embed_compiler!(Github(version = "v0.9.0"));
/// let compiler = embed_compiler!(Luarocks(version = "v0.9.0"));
/// let lua_code = compiler("your_teal_file.tl");
///```
///From filesystem
//Not tested so it can have a nice path and also to not depend on having the teal compiler at a static place.
///```ignore
/// let compiler = embed_compiler!(Local(path = "some/path/to/tl.tl"));
/// //This tries to find the teal compiler on its own
/// let compiler = embed_compiler!(Local());
///```
#[cfg(any(
    feature = "embed_compiler_from_local",
    feature = "embed_compiler_from_download"
))]
#[proc_macro]
pub fn embed_compiler(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as EmbedOptions);
    let compiler = embed_compiler::get_teal(input);
    let primed_vm_string = format!(
        "local tl = (function()\n{}\nend)()\ntl.loader()\n",
        compiler
    );
    let stream = quote! {
        |require:&str| {
            format!("{}\n return require('{}')",#primed_vm_string,require)
        }
    };
    stream.into()
}
