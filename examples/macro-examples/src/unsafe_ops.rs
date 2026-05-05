//! Unsafe operations tracking: unsafe blocks, raw pointers, transmute
//!
//! Run with: cargo run --example unsafe_ops

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;

#[trace_borrow]
fn unsafe_block_basic() {
    let x = 42;
    unsafe {
        // Unsafe block entered
        println!("Inside unsafe block: {}", x);
    }
    // Unsafe block exited
}

#[trace_borrow]
fn raw_pointer_const() {
    let x = 10;
    let ptr = &x as *const i32;
    
    unsafe {
        println!("Value via raw pointer: {}", *ptr);
    }
}

#[trace_borrow]
fn raw_pointer_mut() {
    let mut x = 10;
    let ptr = &mut x as *mut i32;
    
    unsafe {
        *ptr = 20;
        println!("Modified value: {}", *ptr);
    }
    
    println!("x is now: {}", x);
}

#[trace_borrow]
fn raw_pointer_arithmetic() {
    let arr = [1, 2, 3, 4, 5];
    let ptr = arr.as_ptr();
    
    unsafe {
        for i in 0..5 {
            let val = *ptr.add(i);
            println!("arr[{}] = {}", i, val);
        }
    }
}

#[trace_borrow]
fn transmute_basic() {
    let x: u32 = 0x12345678;
    
    unsafe {
        let bytes: [u8; 4] = std::mem::transmute(x);
        println!("Bytes: {:02x?}", bytes);
    }
}

#[trace_borrow]
fn transmute_reference() {
    let x: i32 = -1;
    
    unsafe {
        let y: u32 = std::mem::transmute(x);
        println!("i32 {} as u32: {}", x, y);
    }
}


fn nested_unsafe() {
    unsafe {
        println!("Outer unsafe");
        unsafe {
            println!("Inner unsafe");
        }
        println!("Back to outer");
    }
}

#[trace_borrow]
fn unsafe_with_control_flow() {
    let data = vec![1, 2, 3, 4, 5];
    let ptr = data.as_ptr();
    
    unsafe {
        for i in 0..data.len() {
            let val = *ptr.add(i);
            if val == 3 {
                println!("Found 3, breaking");
                break;
            }
            println!("Value: {}", val);
        }
    }
}

#[trace_borrow]
fn slice_from_raw_parts() {
    let arr = [10, 20, 30, 40, 50];
    let ptr = arr.as_ptr();
    
    unsafe {
        let slice = std::slice::from_raw_parts(ptr, 3);
        println!("Slice from raw parts: {:?}", slice);
    }
}

#[trace_borrow]
fn box_into_raw() {
    let b = Box::new(42);
    let ptr = Box::into_raw(b);
    
    unsafe {
        println!("Value: {}", *ptr);
        // Reconstruct Box to properly deallocate
        let _ = Box::from_raw(ptr);
    }
}

#[trace_borrow]
fn null_pointer_check() {
    let ptr: *const i32 = std::ptr::null();
    
    if ptr.is_null() {
        println!("Pointer is null");
    } else {
        unsafe {
            println!("Value: {}", *ptr);
        }
    }
}

#[trace_borrow]
fn unsafe_cell_like() {
    use std::cell::UnsafeCell;
    
    let cell = UnsafeCell::new(10);
    
    unsafe {
        let ptr = cell.get();
        *ptr = 20;
        println!("Value: {}", *ptr);
    }
}

fn main() {
    println!("=== Unsafe Block Basic ===");
    reset();
    unsafe_block_basic();
    print_events("unsafe_block_basic");

    println!("\n=== Raw Pointer Const ===");
    reset();
    raw_pointer_const();
    print_events("raw_pointer_const");

    println!("\n=== Raw Pointer Mut ===");
    reset();
    raw_pointer_mut();
    print_events("raw_pointer_mut");

    println!("\n=== Raw Pointer Arithmetic ===");
    reset();
    raw_pointer_arithmetic();
    print_events("raw_pointer_arithmetic");

    println!("\n=== Transmute Basic ===");
    reset();
    transmute_basic();
    print_events("transmute_basic");

    println!("\n=== Transmute Reference ===");
    reset();
    transmute_reference();
    print_events("transmute_reference");

    println!("\n=== Nested Unsafe ===");
    reset();
    nested_unsafe();
    print_events("nested_unsafe");

    println!("\n=== Unsafe with Control Flow ===");
    reset();
    unsafe_with_control_flow();
    print_events("unsafe_with_control_flow");

    println!("\n=== Slice from Raw Parts ===");
    reset();
    slice_from_raw_parts();
    print_events("slice_from_raw_parts");

    println!("\n=== Box into Raw ===");
    reset();
    box_into_raw();
    print_events("box_into_raw");

    println!("\n=== Null Pointer Check ===");
    reset();
    null_pointer_check();
    print_events("null_pointer_check");

    println!("\n=== UnsafeCell Like ===");
    reset();
    unsafe_cell_like();
    print_events("unsafe_cell_like");
}

fn print_events(name: &str) {
    let events = get_events();
    println!("{} generated {} events:", name, events.len());
    for (i, event) in events.iter().enumerate() {
        println!("  {}: {:?}", i + 1, event);
    }
}
