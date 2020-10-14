use rlua::{Lua, UserData, UserDataMethods,Result};
use teal_data::TealData;
use teal_data_methods::TealDataMethods;
use user_data_wrapper::UserDataWrapper;

mod teal_data_methods;
mod teal_multivalue;
mod teal_data;
mod user_data_wrapper;
mod type_printer;

struct Test {}
impl UserData for Test {
    fn add_methods<'lua, T: UserDataMethods<'lua, Self>>(methods: &mut T) {
        let mut x =UserDataWrapper::from_user_data_methods(methods);
        <Self as TealData>::add_methods(&mut x);
    }
}
impl TealData for Test {
    fn get_type_name() -> &'static str {
        "awesome!"
    }
    fn add_methods<'lua, T: TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method("test_method", |_,_,x : i8|{
            Ok(x)
        });
        methods.add_method_mut("test_method_mut", |_,_,x : (i8,i8)|{
            Ok(x.1)
        });
        methods.add_function("test_function", |_,x:i8|{
            Ok((x,2))
        });
        methods.add_function_mut("test_function_mut", |_,x : (i8,i8)|{
            Ok(x)
        })
    }
}
fn main() -> Result<()> {
    <Test as TealData>::add_methods(&mut type_printer::TypePrinter{});
    let lua = Lua::new();
    lua.context(|lua_ctx| {
        let globals = lua_ctx.globals();
        globals.set("test", Test{})?;
        let code = r"
print(test:test_method(1))
print(test:test_method_mut(2,3))
print(test.test_function(4))
print(test.test_function_mut(5,6))
        ";
        lua_ctx.load(code).set_name("test?")?.eval()?;
        Ok(())
    })?;
    Ok(())

}
