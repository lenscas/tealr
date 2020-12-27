# tealr
A wrapper around [rlua](https://crates.io/crates/rlua) to help with embedding teal

tealr adds some traits that replace/extend those from [rlua](https://crates.io/crates/rlua), allowing it to generate the `.d.tl` files needed for teal.
It also contains some macro's to make it easier to load/execute teal scripts. Without having to compile them yourself first.

## Small example
```rust
#[derive(Clone, UserData, TypeRepresentation)]
struct Example {}
impl TealData for Example {
    //implement your methods/functions
    fn add_methods<'lua, T: TealDataMethods<'lua, Self>>(methods: &mut T) {}
}

fn main() -> Result<()> {
    let file_contents = TypeWalker::new()
        .proccess_type::<Example>()
        .generate_global("test")
        .expect("oh no :(");
    //save the file
    println!("{}\n ", file_contents);
}

```
You can find longer ones with comments on what each call does [here](https://github.com/lenscas/tealr/tree/master/tealr/examples)

## Future plans
Tealr can already help with 2 ways to run teal scripts.
It can compile inline teal code at the same time as your rust code
It can also embed the teal compiler for you, allowing you to execute external teal scripts like normal lua scripts.
There is a third method I want tealr to help with. In this mode, it will compile a teal project, pack it into 1 file and embed it into the project.
