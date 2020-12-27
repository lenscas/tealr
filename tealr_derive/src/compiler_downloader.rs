use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
    process::Command,
};

use syn::{parse::Parse, LitStr};
use ureq::get;
use zip::{read::ZipFile, ZipArchive};

pub(crate) struct EmbedOptions {
    pub(crate) version: String,
}
impl Parse for EmbedOptions {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let version: String = input.parse::<LitStr>()?.value();
        enum Checker {
            Start,
            V,
            Number,
            Dot,
        };
        let mut last = Checker::Start;
        let is_valid_version = version.char_indices().all(|(loc, chara)| {
            if loc == 0 {
                if chara == 'v' {
                    last = Checker::V;
                    return true;
                } else {
                    return false;
                }
            }
            match last {
                Checker::V => {
                    if chara.is_digit(10) {
                        last = Checker::Number;
                        return true;
                    }
                    false
                }
                Checker::Number => {
                    if chara == '.' {
                        last = Checker::Dot;
                        return true;
                    }
                    chara.is_digit(10)
                }
                Checker::Dot => {
                    if chara.is_digit(10) {
                        last = Checker::Number;
                        return true;
                    }
                    false
                }
                Checker::Start => {
                    unreachable!()
                }
            }
        });
        if !is_valid_version {
            panic!("Given version is not valid. Versions should look like v{{number}}.{{number}}.{{number}}.")
        }
        Ok(Self { version })
    }
}

pub(crate) fn download_teal(options: EmbedOptions) -> String {
    let url = format!(
        "https://github.com/teal-language/tl/archive/{}.zip",
        options.version
    );
    let res = get(&url).call();
    let mut reader = res.into_reader();
    let mut buffer = Vec::with_capacity(100000);
    reader
        .read_to_end(&mut buffer)
        .expect("Could not read response");

    let mut file = tempfile::tempfile().expect("Could not create file to unpack teal compiler");
    file.write_all(&buffer).expect("Could not write zip file");
    file.flush().expect("Could not flush zip file");

    let mut archive = zip::ZipArchive::new(file).expect("Could not read downloaded zip file");

    let main_folder = format!("tl-{}/", &options.version[1..]);

    let build_dir = tempfile::tempdir().expect("Could not get temp dir to build teal");

    let tl = get_file_from_zip(&mut archive, format!("{}tl", main_folder));
    write_read_to_file(tl, &build_dir.path().join("tl"), &mut buffer);
    let tl_lua = get_file_from_zip(&mut archive, format!("{}tl.lua", main_folder));
    write_read_to_file(tl_lua, &build_dir.path().join("tl.lua"), &mut buffer);
    let tl_tl = get_file_from_zip(&mut archive, format!("{}tl.tl", main_folder));

    let teal_compiler_path = build_dir.path().join("tl.tl");
    write_read_to_file(tl_tl, &teal_compiler_path, &mut buffer);

    let mut compiler = Command::new("lua")
        .current_dir(build_dir.path())
        .args(&["tl", "gen", "-o", "output.lua", "--skip-compat53"])
        .arg(teal_compiler_path)
        .spawn()
        .expect("could not run lua to compile teal without compat");
    if !compiler
        .wait()
        .expect("Could not wait for compiler")
        .success()
    {
        panic!("Could not compile teal without compatibility library")
    }
    let mut buf = String::with_capacity(100);
    File::open(build_dir.path().join("output.lua"))
        .expect("Could not open compiled compiler")
        .read_to_string(&mut buf)
        .expect("Coult not read compiled compiler");
    buf
}

fn get_file_from_zip(zip: &mut ZipArchive<File>, name: String) -> ZipFile {
    zip.by_name(&name)
        .unwrap_or_else(|v| panic!("Could not get {} out of zip file. Error:\n{}", name, v))
}
fn write_read_to_file<T: Read>(mut reader: T, file_path: &PathBuf, buf: &mut Vec<u8>) {
    buf.clear();
    reader
        .read_to_end(buf)
        .unwrap_or_else(|_| panic!("Could not read {:?}", file_path));
    File::create(file_path)
        .unwrap_or_else(|_| panic!("Could not create {:?}", file_path))
        .write_all(buf)
        .unwrap_or_else(|_| panic!("Could not write to {:?}", file_path))
}
