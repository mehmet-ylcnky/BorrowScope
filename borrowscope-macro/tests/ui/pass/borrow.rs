use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn example() {
    let x = String::from("hello");
    let y = &x;
    println!("{}", y);
}

fn main() {
    example();
}
