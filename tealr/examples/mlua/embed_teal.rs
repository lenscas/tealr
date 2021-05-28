//tealr is able to embed the compiler, allowing you to require external teal files as if they where normal lua files

use tealr::embed_compiler;

fn main() -> mlua::Result<()> {
    let compiler = embed_compiler!("v0.9.0");
    let lua = mlua::Lua::new();
    let code = compiler("examples/mlua/basic_teal_file");
    let res: u8 = lua.load(&code).set_name("embedded_compiler")?.eval()?;
    println!("got:{}", res);
    Ok(())
}
