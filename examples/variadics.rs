use mlua::Variadic;
use tealr::{
    mlu::{TealData, TypedFunction},
    ToTypename, TypeWalker,
};
use tealr_derive::MluaUserData;

#[derive(ToTypename, MluaUserData)]
struct Foo();
impl TealData for Foo {
    fn add_methods<T: tealr::mlu::TealDataMethods<Self>>(methods: &mut T) {
        methods.add_method("get_variadic", |_, _, args: Variadic<i8>| {
            let x: i8 = args.iter().sum();
            Ok(x)
        });
        methods.add_method("return_variadic", |_, _, ()| {
            Ok(Variadic::from(vec![1, 2, 3, 4, 5]))
        });
        methods.add_method(
            "take_variadic_lambda",
            |_, _, a: TypedFunction<Variadic<i8>, i8>| a.call(Variadic::from(vec![1, 2, 3, 4, 5])),
        );
        methods.add_method("return_variadic_lambda", |lua, _, ()| {
            TypedFunction::from_rust(|_, args: Variadic<i8>| Ok(args.iter().sum::<i8>()), lua)
        });
    }
}

fn main() {
    let file_contents = TypeWalker::new().process_type::<Foo>().to_json().unwrap();
    println!("{}\n ", file_contents);
}
