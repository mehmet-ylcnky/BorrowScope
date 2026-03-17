use borrowscope_macro::trace_borrow;
use std::rc::Rc;
use std::sync::Arc;
use std::cell::{Cell, RefCell};

#[trace_borrow]
pub fn test_all_smart_pointers() {
    // Smart pointers that should have initializer_kind
    let rc1 = Rc::new(42);
    let arc1 = Arc::new(42);
    let box1 = Box::new(42);
    let cell1 = Cell::new(42);
    let refcell1 = RefCell::new(42);
    
    // Clones
    let rc2 = Rc::clone(&rc1);
    let arc2 = Arc::clone(&arc1);
    
    // Regular variables
    let x = 42;
    let s = String::from("hello");
    let v = vec![1, 2, 3];
    
    println!("Smart pointers: rc1={}, arc1={}, box1={}, cell1={}, refcell1={:?}", 
             rc1, arc1, box1, cell1.get(), refcell1);
    println!("Clones: rc2={}, arc2={}", rc2, arc2);
    println!("Regular: x={}, s={}, v={:?}", x, s, v);
}

