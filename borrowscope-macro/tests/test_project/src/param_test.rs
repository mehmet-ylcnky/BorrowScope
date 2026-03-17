use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;

#[trace_borrow]
pub fn test_function_param(x: i32, s: String) {
    println!("x = {}, s = {}", x, s);
}
