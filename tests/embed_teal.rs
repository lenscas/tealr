use tealr::embed_compiler;

#[test]
fn legacy_syntax() -> mlua::Result<()> {
    let compiler = embed_compiler!("v0.13.1");
    let code = compiler("tests/test_embedded_compiler");
    let res: u8 = mlua::Lua::new()
        .load(&code)
        .set_name("embedded_compiler_legacy")
        .eval()?;
    assert_eq!(res, 5);
    Ok(())
}
#[test]
fn new_syntax_github() -> mlua::Result<()> {
    let compiler = embed_compiler!(GitHub(version = "v0.13.1"));
    let code = compiler("tests/test_embedded_compiler");
    let res: u8 = mlua::Lua::new()
        .load(&code)
        .set_name("embedded_compiler_legacy")
        .eval()?;
    assert_eq!(res, 5);
    Ok(())
}

#[test]
fn new_version_luarocks() -> mlua::Result<()> {
    let compiler = embed_compiler!(Luarocks(version = "v0.13.1"));
    let code = compiler("tests/test_embedded_compiler");
    let res: u8 = mlua::Lua::new()
        .load(&code)
        .set_name("embedded_compiler_legacy")
        .eval()?;
    assert_eq!(res, 5);
    Ok(())
}
#[test]
fn new_syntax_from_local_discover() -> mlua::Result<()> {
    let compiler = embed_compiler!(Local());

    let code = compiler("tests/test_embedded_compiler");
    let res: u8 = mlua::Lua::new()
        .load(&code)
        .set_name("embedded_compiler_legacy")
        .eval()?;
    assert_eq!(res, 5);
    Ok(())
}
