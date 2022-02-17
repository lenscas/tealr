use rlua::ToLua;
use tealr::{
    create_union_rlua,
    rlu::{rlua::FromLua, TealData, TealDataMethods, TypedFunction},
    Direction, RluaUserData, TypeName, TypeWalker,
};

create_union_rlua!(enum X = String | f32 | bool);

#[derive(Clone, RluaUserData, TypeName)]
struct Example {}

//now, implement TealData. This tells mlua what methods are available and tealr what the types are
impl TealData for Example {
    //implement your methods/functions
    fn add_methods<'lua, T: TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method("limited_callback", |lua, _, fun: TypedFunction<X, X>| {
            let param = X::from_lua("nice!".to_lua(lua)?, lua)?;
            let res = fun.call(param)?;
            Ok(res)
        });
        methods.add_method("limited_array", |_, _, x: Vec<X>| Ok(x));
        methods.add_method("limited_simple", |_, _, x: X| Ok(x));
    }
}

#[test]
fn test_limited() {
    let file_contents = TypeWalker::new()
        .process_type::<Example>(Direction::ToLua)
        .generate_global("test")
        .expect("oh no :(");

    assert_eq!(file_contents, "global record test\n\trecord Example\n\t\tuserdata\n\n\t\t-- Pure methods\n\t\tlimited_callback: function(Example,function(string | number | boolean):(string | number | boolean)):(string | number | boolean)\n\n\t\tlimited_array: function(Example,{string | number | boolean}):({string | number | boolean})\n\n\t\tlimited_simple: function(Example,string | number | boolean):(string | number | boolean)\n\n\n\tend\nend\nreturn test");
    let res: bool = rlua::Lua::new()
        .context(|ctx| {
            let globals = ctx.globals();
            globals.set("test", Example {}).unwrap();
            let code = "
        return test:limited_simple(true)
        ";
            ctx.load(code).eval()
        })
        .unwrap();
    assert!(res);
}
