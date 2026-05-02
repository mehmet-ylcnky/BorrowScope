//! Expression tracking: struct, tuple, array, range, cast, closure
//!
//! Run with: cargo run --example expressions

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;

// Struct definitions for examples
#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Debug)]
struct Rectangle {
    top_left: Point,
    bottom_right: Point,
}

#[trace_borrow]
fn struct_basic() {
    let p = Point { x: 10, y: 20 };
    println!("Point: {:?}", p);
}

#[trace_borrow]
fn struct_nested() {
    let rect = Rectangle {
        top_left: Point { x: 0, y: 0 },
        bottom_right: Point { x: 100, y: 100 },
    };
    println!("Rectangle: {:?}", rect);
}

#[trace_borrow]
fn struct_update() {
    let p1 = Point { x: 10, y: 20 };
    let p2 = Point { x: 30, ..p1 };
    println!("p1: {:?}, p2: {:?}", p1, p2);
}

#[trace_borrow]
fn tuple_basic() {
    let t = (1, "hello", 3.14);
    println!("Tuple: {:?}", t);
}

#[trace_borrow]
fn tuple_nested() {
    let t = ((1, 2), (3, 4), (5, 6));
    println!("Nested tuple: {:?}", t);
}

#[trace_borrow]
fn tuple_destructure() {
    let t = (10, 20, 30);
    let (a, b, c) = t;
    println!("a={}, b={}, c={}", a, b, c);
}

#[trace_borrow]
fn array_basic() {
    let arr = [1, 2, 3, 4, 5];
    println!("Array: {:?}", arr);
}

#[trace_borrow]
fn array_repeat() {
    let arr = [0; 10]; // [0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    println!("Repeat array: {:?}", arr);
}

#[trace_borrow]
fn array_nested() {
    let matrix = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];
    println!("Matrix: {:?}", matrix);
}

#[trace_borrow]
fn range_basic() {
    let r1 = 0..10;      // Range
    let r2 = 0..=10;     // RangeInclusive
    println!("Range: {:?}", r1);
    println!("RangeInclusive: {:?}", r2);
}

#[trace_borrow]
fn range_in_for() {
    for i in 0..5 {
        println!("i = {}", i);
    }
}

#[trace_borrow]
fn range_slice() {
    let arr = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let slice = &arr[2..5];
    println!("Slice: {:?}", slice);
}

#[trace_borrow]
fn cast_numeric() {
    let x: i32 = 42;
    let y = x as i64;
    let z = x as f64;
    println!("i32: {}, i64: {}, f64: {}", x, y, z);
}

#[trace_borrow]
fn cast_truncate() {
    let big: i32 = 1000;
    let small = big as i8; // Truncation
    println!("i32: {}, i8: {}", big, small);
}

#[trace_borrow]
fn cast_char() {
    let c = 'A';
    let n = c as u32;
    println!("char '{}' as u32: {}", c, n);
    
    let n2: u32 = 66;
    let c2 = n2 as u8 as char;
    println!("u32 {} as char: '{}'", n2, c2);
}

#[trace_borrow]
fn closure_basic() {
    let add = |a, b| a + b;
    let result = add(2, 3);
    println!("2 + 3 = {}", result);
}

#[trace_borrow]
fn closure_capture_ref() {
    let x = 10;
    let print_x = || println!("x = {}", x);
    print_x();
    println!("x is still: {}", x);
}

#[trace_borrow]
fn closure_capture_move() {
    let s = String::from("hello");
    let consume = move || println!("Consumed: {}", s);
    consume();
    // s is no longer valid here
}

#[trace_borrow]
fn closure_capture_mut() {
    let mut count = 0;
    let mut increment = || {
        count += 1;
        println!("Count: {}", count);
    };
    increment();
    increment();
    increment();
}

#[trace_borrow]
fn closure_as_argument() {
    let numbers = vec![1, 2, 3, 4, 5];
    let doubled: Vec<_> = numbers.iter().map(|x| x * 2).collect();
    println!("Doubled: {:?}", doubled);
}

#[trace_borrow]
fn closure_returning() {
    fn make_adder(x: i32) -> impl Fn(i32) -> i32 {
        move |y| x + y
    }
    
    let add_5 = make_adder(5);
    println!("5 + 10 = {}", add_5(10));
}

fn main() {
    println!("=== Struct Basic ===");
    reset();
    struct_basic();
    print_events("struct_basic");

    println!("\n=== Struct Nested ===");
    reset();
    struct_nested();
    print_events("struct_nested");

    println!("\n=== Struct Update ===");
    reset();
    struct_update();
    print_events("struct_update");

    println!("\n=== Tuple Basic ===");
    reset();
    tuple_basic();
    print_events("tuple_basic");

    println!("\n=== Tuple Nested ===");
    reset();
    tuple_nested();
    print_events("tuple_nested");

    println!("\n=== Tuple Destructure ===");
    reset();
    tuple_destructure();
    print_events("tuple_destructure");

    println!("\n=== Array Basic ===");
    reset();
    array_basic();
    print_events("array_basic");

    println!("\n=== Array Repeat ===");
    reset();
    array_repeat();
    print_events("array_repeat");

    println!("\n=== Array Nested ===");
    reset();
    array_nested();
    print_events("array_nested");

    println!("\n=== Range Basic ===");
    reset();
    range_basic();
    print_events("range_basic");

    println!("\n=== Range in For ===");
    reset();
    range_in_for();
    print_events("range_in_for");

    println!("\n=== Range Slice ===");
    reset();
    range_slice();
    print_events("range_slice");

    println!("\n=== Cast Numeric ===");
    reset();
    cast_numeric();
    print_events("cast_numeric");

    println!("\n=== Cast Truncate ===");
    reset();
    cast_truncate();
    print_events("cast_truncate");

    println!("\n=== Cast Char ===");
    reset();
    cast_char();
    print_events("cast_char");

    println!("\n=== Closure Basic ===");
    reset();
    closure_basic();
    print_events("closure_basic");

    println!("\n=== Closure Capture Ref ===");
    reset();
    closure_capture_ref();
    print_events("closure_capture_ref");

    println!("\n=== Closure Capture Move ===");
    reset();
    closure_capture_move();
    print_events("closure_capture_move");

    println!("\n=== Closure Capture Mut ===");
    reset();
    closure_capture_mut();
    print_events("closure_capture_mut");

    println!("\n=== Closure as Argument ===");
    reset();
    closure_as_argument();
    print_events("closure_as_argument");

    println!("\n=== Closure Returning ===");
    reset();
    closure_returning();
    print_events("closure_returning");
}

fn print_events(name: &str) {
    let events = get_events();
    println!("{} generated {} events:", name, events.len());
    for (i, event) in events.iter().enumerate() {
        println!("  {}: {:?}", i + 1, event);
    }
}
