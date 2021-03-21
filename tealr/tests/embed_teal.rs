use tealr::embed_compiler;

#[test]
fn legacy_syntax() -> rlua::Result<()> {
    let compiler = embed_compiler!("v0.13.1");
    let res = rlua::Lua::new().context(|ctx| {
        let code = compiler("tests/test_embedded_compiler");
        let res: u8 = ctx
            .load(&code)
            .set_name("embedded_compiler_legacy")?
            .eval()?;
        Ok(res)
    })?;
    assert_eq!(res, 5);
    Ok(())
}
#[test]
fn new_syntax_github() -> rlua::Result<()> {
    let compiler = embed_compiler!(GitHub(version = "v0.13.1"));
    let res = rlua::Lua::new().context(|ctx| {
        let code = compiler("tests/test_embedded_compiler");
        let res: u8 = ctx
            .load(&code)
            .set_name("embedded_compiler_legacy")?
            .eval()?;
        Ok(res)
    })?;
    assert_eq!(res, 5);
    Ok(())
}

#[test]
fn new_version_luarocks() -> rlua::Result<()> {
    let compiler = embed_compiler!(Luarocks(version = "v0.13.1"));
    let res = rlua::Lua::new().context(|ctx| {
        let code = compiler("tests/test_embedded_compiler");
        let res: u8 = ctx
            .load(&code)
            .set_name("embedded_compiler_legacy")?
            .eval()?;
        Ok(res)
    })?;
    assert_eq!(res, 5);
    Ok(())
}
#[test]
fn new_syntax_from_local_discover() -> rlua::Result<()> {
    let compiler = embed_compiler!(Local());
    let res = rlua::Lua::new().context(|ctx| {
        let code = compiler("tests/test_embedded_compiler");
        let res: u8 = ctx
            .load(&code)
            .set_name("embedded_compiler_legacy")?
            .eval()?;
        Ok(res)
    })?;
    assert_eq!(res, 5);
    Ok(())
}
