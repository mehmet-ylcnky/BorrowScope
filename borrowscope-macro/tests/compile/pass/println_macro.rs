use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn test_println_macro() {
    let name = String::from("World");
    println!("Hello, {}!", name);
    assert_eq!(name, "World");
}

fn main() {
    test_println_macro();
}
