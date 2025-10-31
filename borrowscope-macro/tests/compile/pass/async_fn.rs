use borrowscope_macro::trace_borrow;

#[trace_borrow]
async fn example() -> i32 {
    let x = 42;
    x
}

fn main() {}
