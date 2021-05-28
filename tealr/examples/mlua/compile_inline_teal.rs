//This example shows how to build inline teal code at compile time
//This is useful if you want to write a bit of teal/lua code directly in your application that gets passed to RLua.
//NOTE: At this point it requires you to have teal installed and accessible as `tl` at compile time.

use mlua::{Lua, Result};

use tealr::compile_inline_teal;

//This example using `compile_inline_teal` which takes in some teal code and compiles it.
fn main() -> Result<()> {
    let lua = Lua::new();

    let code = compile_inline_teal!(
        "
function add(param1 :number, param2:number):number
return param1 + param2
end
local concat = require('examples/rlua/basic_type').concat
print(concat('a','b'))
return add(1,2)
        "
    );

    let result: String = lua
        .load(code)
        .set_name("compile inline teal example")?
        .eval()?;
    println!("output:{}", result);

    Ok(())
}
