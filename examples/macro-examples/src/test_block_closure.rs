use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;

#[trace_borrow]
fn test_block_closure() {
    let opt: Option<i32> = None;
    let _value = opt.unwrap_or_else(|| {
        println!("Computing...");
        42
    });
}

fn main() {
    reset();
    test_block_closure();
    print_summary();
}
