use std::marker::PhantomData;

use tealr::{
    ToTypename, TypeBody, TypeWalker,
    mlu::{TealData, TealDataMethods, TypedFunction, UserData},
};

#[derive(Clone, UserData, ToTypename)]
struct Test;
impl TealData for Test {
    fn add_methods<T: TealDataMethods<Self>>(methods: &mut T) {
        methods
            .document_type("a test type")
            .document("a test method")
            .add_method_mut("test", |_, _, v: u8| Ok(v + 1));
        methods
            .document("This method is overloaded")
            .add_overload::<(Test, String), String>()
            .add_overload::<Test, ()>()
            .hide_main()
            .add_method("overloaded_method", |_, _, a: mlua::Value| {
                let mlua::Value::String(a) = a else {
                    return Ok(None);
                };
                Ok(Some(a))
            });
    }

    fn add_fields<F: tealr::mlu::TealDataFields<Self>>(fields: &mut F) {
        fields.document("a test field");
        fields.add_field_function_get("test2", |_, _| Ok(2));
    }
}

#[derive(tealr::mlu::FromToLua, ToTypename)]
struct Foo {
    ///test a field
    a: u8,
}

#[derive(Clone, UserData, ToTypename)]
struct All;
impl TealData for All {
    fn add_fields<F: tealr::mlu::TealDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("test", |_, _| Ok(Test));
    }
}
///just a test enum
#[derive(tealr::mlu::FromToLua, ToTypename)]
enum TestEnum {
    ///test a variant
    First,
    Second,
}

struct Generic<A> {
    _a: PhantomData<A>,
}

impl<A: ToTypename> ToTypename for Generic<A> {
    fn to_typename() -> tealr::Type {
        tealr::Type::new_single_with_generics(
            "Generic",
            tealr::KindOfType::External,
            vec![A::to_typename()],
        )
    }
}
impl<A: ToTypename> TypeBody for Generic<A> {
    fn get_type_body() -> tealr::TypeGenerator {
        let mut a = tealr::RecordGenerator::new::<Self>(false);
        <Self as TealData>::add_fields(&mut a);
        <Self as TealData>::add_methods(&mut a);
        a.is_user_data = true;
        tealr::TypeGenerator::Record(Box::new(a))
    }
}
impl<X: ToTypename> tealr::mlu::mlua::UserData for Generic<X> {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        let mut x = tealr::mlu::UserDataWrapper::from_user_data_fields(fields);
        <Self as TealData>::add_fields(&mut x);
    }

    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        <Self as TealData>::add_methods(&mut tealr::mlu::UserDataWrapper::from_user_data_methods(
            methods,
        ));
    }
}

impl<A: ToTypename> TealData for Generic<A> {
    fn add_methods<T: TealDataMethods<Self>>(methods: &mut T) {
        methods
            .document_type("a test type")
            .document("a test method")
            .add_method("test", |_, _, v: u8| Ok(v + 1));
    }
}

#[test]
fn help_function() {
    let types = TypeWalker::new()
        .process_type::<Test>()
        .process_type::<Foo>()
        .process_type::<TestEnum>()
        .process_type::<Generic<tealr::mlu::generics::A>>()
        .process_type_inline::<All>();

    let lua = tealr::mlu::mlua::Lua::new();

    let help = tealr::mlu::generate_help(&lua, types).unwrap();
    lua.globals().set("help", help).unwrap();
    lua.globals()
        .set(
            "assert_eq",
            TypedFunction::<(mlua::Value, mlua::Value), ()>::from_rust(
                |_, (a, b)| {
                    assert_eq!(a, b);
                    Ok(())
                },
                &lua,
            )
            .unwrap(),
        )
        .unwrap();
    lua.load(
        "
    local no_params = help()
    assert_eq(no_params,\"Methods:\\nFields:\\ntest : Test\")
    assert_eq(no_params,help(\"All\"))
    print(help(\"Test\",\"test\"))
    assert_eq(help(\"Test\"), \"a test type\\n\\nMethods:\\ntest : function(self:Test , integer):(integer)\\noverloaded_method : function(Test , string):(string)\\nFields:\\ntest2 : integer\")
    assert_eq(help(\"Test\",\"test\"), \"test : function(self:Test , integer):(integer)\\na test method\")
    assert_eq(help(\"Generic\"), \"a test type\\n\\nMethods:\\ntest : function(self:Generic<A> , integer):(integer)\\nFields:\\n\")
    assert_eq(help(\"TestE\"), \"just a test enum\\n\\nVariants:\\nFirst\\nSecond\\n\")
    assert_eq(help(\"Test\",\"overloaded_method\"), \"overloaded_method : function(Test , string):(string)\\n                    function(Test)\\nThis method is overloaded\")
    ",
    )
    .exec()
    .unwrap();
}
