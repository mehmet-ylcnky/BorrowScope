use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;

#[trace_borrow]
pub fn test_copy_vs_move() {
    // Copy types - should generate New events, not Move
    let x = 42i32;
    let y = x;  // Copy, not move
    let z = y;  // Copy, not move
    
    // Non-copy types - should generate Move events
    let s1 = String::from("hello");
    let s2 = s1;  // Move
    
    let v1 = vec![1, 2, 3];
    let v2 = v1;  // Move
    
    println!("x={}, y={}, z={}, s2={}, v2={:?}", x, y, z, s2, v2);
}
