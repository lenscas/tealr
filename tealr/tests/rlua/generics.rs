use std::collections::HashMap;

use tealr::{
    rlu::{generics::X, rlua::ToLua, TealData, TealDataMethods, TypedFunction},
    Direction, RluaUserData, TypeName, TypeWalker,
};

#[derive(Clone, RluaUserData, TypeName)]
struct Example {}

//now, implement TealData. This tells mlua what methods are available and tealr what the types are
impl TealData for Example {
    //implement your methods/functions
    fn add_methods<'lua, T: TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method(
            "generic_function_callback",
            |lua, _, fun: TypedFunction<X, X>| {
                let param = X::from("nice!".to_lua(lua)?);
                let res = fun.call(param)?;
                Ok(res)
            },
        );
        methods.add_method("generic_array", |_, _, x: Vec<X>| Ok(x));
        methods.add_method("generic_hashmap", |_, _, x: HashMap<String, X>| Ok((x, 8)));
        methods.add_method("just_generics", |_, _, x: X| Ok(x));
        methods.add_method("non_generic_container", |_, _, x: Vec<String>| Ok(x))
    }
}

#[test]
fn make_generic() {
    let file_contents = TypeWalker::new()
        .process_type::<Example>(Direction::ToLua)
        .generate_global("test")
        .expect("oh no :(");

    assert_eq!(file_contents, "global record test\n\trecord Example\n\t\tuserdata\n\n\t\t-- Pure methods\n\t\tgeneric_function_callback: function<X>(Example,function(X):(X)):(X)\n\n\t\tgeneric_array: function<X>(Example,{X}):({X})\n\n\t\tgeneric_hashmap: function<X>(Example,{string:X}):(({string:X}),(integer))\n\n\t\tjust_generics: function<X>(Example,X):(X)\n\n\t\tnon_generic_container: function(Example,{string}):({string})\n\n\n\tend\nend\nreturn test");
}
