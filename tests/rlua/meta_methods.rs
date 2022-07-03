use tealr::{
    rlu::{rlua::MetaMethod, TealData, TealDataMethods, UserData},
    TypeName, TypeWalker,
};

#[derive(Clone, UserData, TypeName)]
struct Example {
    add: i8,
}

//now, implement TealData. This tells rlua what methods are available and tealr what the types are
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
        .generate_global("test")
        .expect("oh no :(");
    assert_eq!(file_contents, "global record test\n\trecord Example\n\t\tuserdata\n\n\t\t-- Meta methods\n\t\t__add: function(Example,integer):(Example)\n\n\n\tend\nend\nreturn test");
}
