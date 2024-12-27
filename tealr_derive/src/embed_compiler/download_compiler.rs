use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

use ureq::get;
use zip::{read::ZipFile, ZipArchive};

use super::load_from_disk::get_local_teal;

pub(crate) fn download_teal(url: String, main_folder: String) -> String {
    let res = match get(&url).call() {
        Ok(x) => x,
        Err(ureq::Error::Status(state, res)) => {
            eprintln!("Did not get a success status. Got: {}", state);
            eprintln!("Message: {:?}", res);
            res
        },
        Err(x) => panic!("Failed downloading teal compiler. Error:{}", x),
    };
    let mut reader = res.into_reader();
    let mut buffer = Vec::with_capacity(100000);
    reader
        .read_to_end(&mut buffer)
        .expect("Could not read response");

    let mut file = tempfile::tempfile().expect("Could not create file to unpack teal compiler");
    file.write_all(&buffer).expect("Could not write zip file");
    file.flush().expect("Could not flush zip file");

    let mut archive = zip::ZipArchive::new(file).expect("Could not read downloaded zip file");

    let build_dir = tempfile::tempdir().expect("Could not get temp dir to build teal");

    let tl = get_file_from_zip(&mut archive, format!("{}tl", main_folder));
    write_read_to_file(tl, &build_dir.path().join("tl"), &mut buffer);
    let tl_tl = get_file_from_zip(&mut archive, format!("{}tl.tl", main_folder));

    let teal_compiler_path = build_dir.path().join("tl.tl");
    write_read_to_file(tl_tl, &teal_compiler_path, &mut buffer);

    get_local_teal(teal_compiler_path.to_string_lossy().to_string())
}

pub(crate) fn download_teal_from_luarocks(version: String) -> String {
    let version = version[1..].to_string();
    let url = format!(
        "https://luarocks.org/manifests/hisham/tl-{}-1.src.rock",
        version
    );
    let main_folder = "tl/".to_string();
    download_teal(url, main_folder)
}

pub(crate) fn download_teal_from_github(version: String) -> String {
    let url = format!(
        "https://github.com/teal-language/tl/archive/{}.zip",
        version
    );
    let main_folder = format!("tl-{}/", &version[1..]);
    download_teal(url, main_folder)
}

fn get_file_from_zip(zip: &mut ZipArchive<File>, name: String) -> ZipFile {
    zip.by_name(&name)
        .unwrap_or_else(|v| panic!("Could not get `{}` out of zip file. Error:\n{}", name, v))
}
fn write_read_to_file<T: Read>(mut reader: T, file_path: &Path, buf: &mut Vec<u8>) {
    buf.clear();
    reader
        .read_to_end(buf)
        .unwrap_or_else(|_| panic!("Could not read {:?}", file_path));
    File::create(file_path)
        .unwrap_or_else(|_| panic!("Could not create {:?}", file_path))
        .write_all(buf)
        .unwrap_or_else(|_| panic!("Could not write to {:?}", file_path))
}
