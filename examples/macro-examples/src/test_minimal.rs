use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;

#[trace_borrow]
fn test_simple_unwrap_or_else() {
    let opt: Option<i32> = None;
    let _value = opt.unwrap_or_else(|| 42);
}

fn main() {
    reset();
    test_simple_unwrap_or_else();
    print_summary();
}
