use mlua::MetaMethod;
use tealr::{
    mlu::{TealData, TealDataMethods, UserData},
    ToTypename, TypeWalker,
};

#[derive(Clone, UserData, ToTypename)]
struct Example {
    add: i8,
}

//now, implement TealData. This tells Mlua what methods are available and tealr what the types are
impl TealData for Example {
    //implement your methods/functions
    fn add_methods<T: TealDataMethods<Self>>(methods: &mut T) {
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
    let mut old_value: serde_json::Value =
        serde_json::from_str(include_str!("meta_methods.json")).unwrap();
    let mut x = old_value
        .get_mut("tealr_version_used")
        .expect("missing tealr_version_used in original");
    if let serde_json::Value::String(x) = &mut x {
        *x = tealr::get_tealr_version().to_string();
    }

    assert_eq!(new_value, old_value);
}
