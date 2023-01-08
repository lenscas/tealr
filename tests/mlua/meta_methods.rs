use mlua::MetaMethod;
use tealr::{
    mlu::{TealData, TealDataMethods, UserData},
    TypeName, TypeWalker,
};

#[derive(Clone, UserData, TypeName)]
struct Example {
    add: i8,
}

//now, implement TealData. This tells Mlua what methods are available and tealr what the types are
impl TealData for Example {
    //implement your methods/functions
    fn add_methods<'lua, T: TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_meta_method(MetaMethod::Add, |_, me, other: i8| {
            Ok(Example {
                add: me.add + other,
            })
        })
    }
}
#[test]
fn test() {
    //create .d.tl file and compare against expected
    let file_contents = TypeWalker::new()
        .process_type::<Example>()
        .to_json()
        .expect("oh no :(");

    let new_value: serde_json::Value = serde_json::from_str(&file_contents).unwrap();
    let old_value: serde_json::Value =
        serde_json::from_str(include_str!("./meta_methods.json")).unwrap();
    assert_eq!(new_value, old_value);
}
