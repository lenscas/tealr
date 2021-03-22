fn run_find_command(is_local: bool) -> String {
    let mut command = Command::new("luarocks");
    command.arg("which").arg("tl.tl");
    if is_local {
        command.arg("--local");
    }
    let command = command.output().unwrap_or_else(|e| {
        panic!(
            "Could not execute `luarocks which tl.tl` to discover location of tl.tl. Error:\n{}",
            e
        )
    });
    let stdout = String::from_utf8(command.stdout).unwrap();
    if !command.status.success() {
        if is_local {
            return run_find_command(true);
        } else {
            panic!(
                "`luarocks which tl.tl` did not exit successfully. Status code : {}\n StdErr :\n{}]\n\nstdOut:\n{}",
                command.status,
                String::from_utf8(command.stderr).unwrap(),
                stdout
            );
        }
    }
    stdout
        .split(|v| v == '\n')
        .next()
        .expect("Did not get the expected output from luarocks")
        .to_string()
}

use std::{fs::read_to_string, process::Command};
pub(crate) fn discover_tl_tl() -> String {
    run_find_command(true)
}
pub(crate) fn get_local_teal(path: String) -> String {
    let build_dir = tempfile::tempdir().expect("Could not get temp dir to build teal");

    let mut compiler = Command::new("tl")
        .current_dir(build_dir.path())
        .args(&["gen", "-o", "output.lua", "--skip-compat53"])
        .arg(path)
        .spawn()
        .expect("could not run lua to compile teal without compat");

    if !compiler
        .wait()
        .expect("Could not wait for compiler")
        .success()
    {
        panic!("Could not compile teal without compatibility library")
    }
    read_to_string(build_dir.path().join("output.lua")).expect("Could not read compiled compiler")
}
