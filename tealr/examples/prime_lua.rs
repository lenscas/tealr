//tealr is able to embed the compiler, allowing you to require external teal files as if they where normal lua files

use tealr::embed_compiler;

fn main() -> rlua::Result<()> {
    let compiler = embed_compiler!("v0.9.0");
    let res = rlua::Lua::new().context(|ctx| {
        let code = compiler("examples/basic_teal_file");
        let res: u8 = ctx.load(&code).set_name("embedded_compiler")?.eval()?;
        Ok(res)
    })?;
    println!("got:{}", res);
    Ok(())
}
