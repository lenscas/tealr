use std::collections::HashMap;

use tealr::{
    rlu::{generics::X, rlua::ToLua, TealData, TealDataMethods, TypedFunction, UserData},
    TypeName, TypeWalker,
};

#[derive(Clone, UserData, TypeName)]
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
        .process_type::<Example>()
        .to_json_pretty()
        .unwrap();

    let new_value: serde_json::Value = serde_json::from_str(&file_contents).unwrap();
    let old_value: serde_json::Value =
        serde_json::from_str(include_str!("./generics.json")).unwrap();
    assert_eq!(new_value, old_value);
}
