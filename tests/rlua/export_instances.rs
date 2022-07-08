use std::borrow::Cow;

use rlua::ToLua;
use tealr::{
    create_union_rlua,
    rlu::{rlua::FromLua, TealData, TealDataMethods, TypedFunction, UserData},
    TypeName, TypeWalker,
};

create_union_rlua!(enum X = String | f32 | bool);

#[derive(Clone, UserData, TypeName)]
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

struct Export;
impl tealr::rlu::ExportInstances for Export {
    fn add_instances<'lua, T: tealr::rlu::InstanceCollector<'lua>>(
        instance_collector: &mut T,
    ) -> rlua::Result<()> {
        instance_collector.add_instance(Cow::Borrowed("test"), |_| Ok(Example {}))?;
        instance_collector.document_instance("a simple function that does a + 1");
        instance_collector.document_instance("it is just for testing purposes");
        instance_collector.add_instance(Cow::Borrowed("example_a"), |context| {
            tealr::rlu::TypedFunction::from_rust(|_, a: i32| Ok(a + 1), context)
        })?;
        Ok(())
    }
}

#[test]
fn test_limited() {
    let file_contents = TypeWalker::new()
        .process_type::<Example>()
        .document_global_instance::<Export>()
        .unwrap()
        .generate_global("Test")
        .expect("oh no :(");

    assert_eq!(file_contents, "global record Test\n\trecord Example\n\t\tuserdata\n\n\t\t-- Pure methods\n\t\tlimited_callback: function(Example,function(string | number | boolean):(string | number | boolean)):(string | number | boolean)\n\n\t\tlimited_array: function(Example,{string | number | boolean}):({string | number | boolean})\n\n\t\tlimited_simple: function(Example,string | number | boolean):(string | number | boolean)\n\n\n\tend\nend\nglobal test: Test.Example\n--a simple function that does a + 1\n\n--it is just for testing purposes\n\nglobal example_a: function(integer):(integer)\nreturn Test");
    let res: bool = rlua::Lua::new()
        .context(|ctx| {
            tealr::rlu::set_global_env::<Export>(ctx)?;

            let code = "
            assert(example_a(2) == 3)
        return test:limited_simple(true)
        ";
            ctx.load(code).eval()
        })
        .unwrap();
    assert!(res);
}
