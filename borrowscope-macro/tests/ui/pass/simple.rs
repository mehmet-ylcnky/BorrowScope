use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn example() {
    let x = 5;
}

fn main() {
    example();
}
