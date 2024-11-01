use std::{
    ffi::OsStr,
    fs::{read_to_string, File},
    io::Write,
    path::PathBuf,
    process::Command,
};

use proc_macro2::{TokenStream, TokenTree};

struct CompileConfig {
    code: String,
    path: PathBuf,
}

impl CompileConfig {
    fn parse(input: TokenStream) -> Result<CompileConfig, venial::Error> {
        let mut input = input.into_iter();
        let code = match input.next() {
            Some(TokenTree::Literal(x)) => {
                let stringified = x.to_string().trim().to_string();
                stringified[1..(stringified.len() - 1)].to_string()
            }
            Some(_) => return Err(venial::Error::new("Expected string literal")),
            None => return Err(venial::Error::new("Missing code to run")),
        };
        match input.next() {
            Some(TokenTree::Punct(x)) => {
                if x.as_char() != ',' {
                    return Err(venial::Error::new(format!(
                        "Expected `,` got `{}`.",
                        x.as_char()
                    )));
                }
            }
            Some(x) => return Err(venial::Error::new(format!("Expected `,` got `{}`.", x))),
            None => (),
        }
        let path_extra = match input.next() {
            None => None,
            Some(TokenTree::Literal(x)) => Some(x.to_string()),
            Some(_) => return Err(venial::Error::new("Expected nothing or string literal.")),
        };
        let mut path: PathBuf = std::env::var("CARGO_MANIFEST_DIR")
            .expect("Could not get the crate directory")
            .into();
        if let Some(x) = path_extra {
            path = path.join(x);
        }
        Ok(CompileConfig { code, path })
    }
}

pub(crate) fn compile_inline_teal(input: TokenStream) -> TokenStream {
    let input = match CompileConfig::parse(input) {
        Ok(x) => x,
        Err(x) => return x.to_compile_error(),
    };

    let code = input.code.trim();
    let path = input.path;

    let dir = tempfile::tempdir().expect("Could not create a temporary directory");
    let temp_path = dir.path();
    let mut input_file =
        File::create(temp_path.join("input.tl")).expect("Could not create teal source file");
    input_file
        .write_all(code.as_bytes())
        .expect("Could not write teal source file");

    let output = Command::new("tl")
        .args([
            OsStr::new("check"),
            OsStr::new("-I"),
            path.as_os_str(),
            OsStr::new("input.tl"),
        ])
        .current_dir(temp_path)
        .output()
        .expect("Could not run `tl check`. Make sure it is available in the path");

    if !output.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr).trim());
        panic!("There was an error while typechecking your teal code.")
    }

    let mut command = Command::new("tl")
        .args([
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

    quote! {#contents}
}
