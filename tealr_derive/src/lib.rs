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

mod compiler_downloader;

use std::{
    ffi::OsStr,
    fs::{read_to_string, File},
    io::Write,
    path::PathBuf,
    process::Command,
};

use compiler_downloader::EmbedOptions;
use proc_macro::TokenStream;
use syn::{parse::Parse, parse_macro_input, LitStr, Token};

///Implements UserData.
///
///It wraps the UserDataMethods into tealr::UserDataWrapper
///and then passes it to tealr::TealData::add_methods.
#[cfg(feature = "derive")]
#[proc_macro_derive(UserData)]
pub fn user_data_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_user_data_derive(&ast).into()
}

fn impl_user_data_derive(ast: &syn::DeriveInput) -> syn::export::TokenStream2 {
    let name = &ast.ident;
    let gen = quote! {
        impl UserData for #name {
            fn add_methods<'lua, T: UserDataMethods<'lua, Self>>(methods: &mut T) {
                let mut x = tealr::UserDataWrapper::from_user_data_methods(methods);
                <Self as TealData>::add_methods(&mut x);
            }
        }
    };
    gen
}

///Implements TypeRepresentation.
///
///TypeRepresentation::get_type_name will return the name of the type.
#[cfg(feature = "derive")]
#[proc_macro_derive(TypeRepresentation)]
pub fn type_representation_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_type_representation_derive(&ast).into()
}
fn impl_type_representation_derive(ast: &syn::DeriveInput) -> syn::export::TokenStream2 {
    let name = &ast.ident;
    let gen = quote! {
        impl TypeRepresentation for #name {
            fn get_type_name() -> ::std::borrow::Cow<'static, str> {
                ::std::borrow::Cow::from(stringify!(#name))
            }
        }
    };
    gen
}

///Implement both UserData and TypeRepresentation.
///
///Look at tealr_derive::UserData and tealr_derive::TypeRepresentation
///for more information on how the implemented traits will behave.
#[cfg(feature = "derive")]
#[proc_macro_derive(TealDerive)]
pub fn teal_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let mut stream = impl_type_representation_derive(&ast);
    stream.extend(impl_user_data_derive(&ast));
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
///NOTE: At this point in time this requires you to have the teal compiler installed and accessible as `tl`.
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
///It does so by downloading the given version of the teal compiler from github
///Compiling it without the lua5.3 compatibility library and embedding it into your application.
///
///It returns a closure that takes the file that needs to run
///and returns valid lua code that both prepares the lua vm so it can run teal files and
///loads the given file using `require`, returning the result of the file that got loaded.

///NOTE: Due to how the teal files are being loaded, they won't be typed checked.
///More info on: https://github.com/teal-language/tl/blob/master/docs/tutorial.md (Search for "loader")

#[cfg(feature = "embed_compiler")]
#[proc_macro]
pub fn embed_compiler(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as EmbedOptions);
    let compiler = compiler_downloader::download_teal(input);
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
