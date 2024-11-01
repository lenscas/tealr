use mlua::IntoLua;
use tealr::{
    create_union_mlua,
    mlu::{mlua::FromLua, TealData, TealDataMethods, TypedFunction, UserData},
    ToTypename, TypeWalker,
};

create_union_mlua!(enum X = String | f32 | bool);

#[derive(Clone, UserData, ToTypename)]
struct Example {}

//now, implement TealData. This tells mlua what methods are available and tealr what the types are
impl TealData for Example {
    //implement your methods/functions
    fn add_methods<T: TealDataMethods<Self>>(methods: &mut T) {
        methods.add_method("limited_callback", |lua, _, fun: TypedFunction<X, X>| {
            let param = X::from_lua("nice!".into_lua(lua)?, lua)?;
            let res = fun.call(param)?;
            Ok(res)
        });
        methods.add_method("limited_array", |_, _, x: Vec<X>| Ok(x));
        methods.add_method("limited_simple", |_, _, x: X| Ok(x));
    }
}

#[derive(Default)]
struct Export;
impl tealr::mlu::ExportInstances for Export {
    fn add_instances<T: tealr::mlu::InstanceCollector>(
        self,
        instance_collector: &mut T,
    ) -> mlua::Result<()> {
        instance_collector.add_instance("test", |_| Ok(Example {}))?
            .document_instance("a simple function that does a + 1")
            .document_instance("it is just for testing purposes")
            .add_instance("example_a", |context| {
                TypedFunction::from_rust(|_, a: i32| Ok(a + 1), context)
            })?
            .document_instance("A simple generic function to make sure generic functions in global context stay working")
            .add_instance("example_generic", |context| {
                TypedFunction::from_rust(|_, a: tealr::mlu::generics::X| Ok(a), context)
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
        .to_json()
        .expect("oh no :(");

    let generated: serde_json::Value = serde_json::from_str(&file_contents).unwrap();
    let mut original: serde_json::Value =
        serde_json::from_str(include_str!("export_instances.json")).unwrap();

    let mut x = original
        .get_mut("tealr_version_used")
        .expect("missing tealr_version_used in original");
    if let serde_json::Value::String(x) = &mut x {
        *x = tealr::get_tealr_version().to_string();
    }
    assert_eq!(generated, original);

    let lua = mlua::Lua::new();
    tealr::mlu::set_global_env(Export {}, &lua).unwrap();
    let code = "
            assert(example_a(2) == 3)
        return test:limited_simple(true)
        ";
    let res: bool = lua.load(code).eval().unwrap();
    assert!(res);
}
