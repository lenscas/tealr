use rlua::MetaMethod;
use tealr::{
    rlu::{TealData, TealDataMethods},
    RluaUserData, TypeName, TypeWalker,
};

#[derive(Clone, RluaUserData, TypeName)]
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
        .process_type::<Example>(tealr::Direction::ToLua)
        .generate_global("test")
        .expect("oh no :(");
    assert_eq!(file_contents, "global record test\n\trecord Example\n\t\tuserdata\n\t\t-- Meta methods\n\t\tmetamethod __add: function(Example, integer):(Example)\n\n\tend\nend\nreturn test");
}
