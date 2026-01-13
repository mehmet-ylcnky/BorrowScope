//! Test case for variable name disambiguation
//! 
//! This example has multiple variables with the same name in different functions
//! and scopes. The analyzer should correctly disambiguate them using function_name
//! and decl_index.

use std::rc::Rc;
use std::sync::Arc;
use std::cell::RefCell;
use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;

#[trace_borrow]
fn function_a() {
    // 'data' is Rc<i32> here
    let data = Rc::new(42);
    let clone = Rc::clone(&data);
    println!("function_a: {:?}", clone);
}

#[trace_borrow]
fn function_b() {
    // 'data' is Arc<String> here - different type!
    let data = Arc::new(String::from("hello"));
    let clone = Arc::clone(&data);
    println!("function_b: {:?}", clone);
}

#[trace_borrow]
fn function_c() {
    // 'data' is RefCell<Vec<i32>> here - yet another type!
    let data = RefCell::new(vec![1, 2, 3]);
    let borrow = data.borrow();
    println!("function_c: {:?}", borrow);
}

#[trace_borrow]
fn shadowing_test() {
    // Multiple 'x' variables in same function (shadowing)
    let x = 1;           // decl_index 0: i32
    let x = "hello";     // decl_index 1: &str  
    let x = vec![1, 2];  // decl_index 2: Vec<i32>
    let x = Rc::new(x);  // decl_index 3: Rc<Vec<i32>>
    println!("{:?}", x);
}

#[trace_borrow]
fn nested_scopes() {
    let outer = 1;
    {
        let inner = 2;
        let outer = "shadowed"; // Same name as outer scope
        println!("{} {}", inner, outer);
    }
    println!("{}", outer);
}

fn main() {
    reset();
    
    function_a();
    function_b();
    function_c();
    shadowing_test();
    nested_scopes();
    
    // Print summary
    print_summary();
    
    // Print events for verification
    let events = get_events();
    println!("\n=== Events ===");
    for event in events.iter().take(20) {
        println!("{:?}", event);
    }
}
