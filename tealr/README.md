# tealr
A wrapper around [rlua](https://crates.io/crates/rlua) to help with embedding teal

tealr adds some traits that replace/extend those from [rlua](https://crates.io/crates/rlua), allowing it to generate the `.d.tl` files needed for teal.

## Small example
```rust
#[derive(Clone,Copy,UserData)]
struct Example {}
impl TealData for Example {
    fn get_type_name() -> String {
        String::from("Example")
    }
fn main() -> Result<()> {
    let file_contents = TypeWalker::new() 
        .proccess_type::<Example>()
        .generate("test")
        .expect("oh no :(");
    println!("{}\n ", file_contents);
    Ok(())
}
```
You can find longer ones [here](https://github.com/lenscas/tealr/tree/master/tealr/examples)

## Future plans
Its possible for lua to load .tl files directly after it loaded the compiler. I would like to make use of this and expose methods that already perpare the lua vm in this way.

This should make it pretty much as easy to work with teal as with lua. However, I am not sure if doing this breaks any rules from rlua. As such, some research is required.