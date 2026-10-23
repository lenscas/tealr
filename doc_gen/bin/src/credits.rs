pub(crate) fn show_credits() {
    let z = include_str!("../credits.md");
    println!("{z}")
}
