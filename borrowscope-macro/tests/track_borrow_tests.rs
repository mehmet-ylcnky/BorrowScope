//! Integration tests for track_borrow injection (Section 39)
//!
//! These tests verify that the macro correctly transforms borrow expressions

#[test]
fn test_immutable_borrow() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/immutable_borrow.rs");
}

#[test]
fn test_mutable_borrow() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/mutable_borrow.rs");
}

#[test]
fn test_multiple_borrows() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/multiple_borrows.rs");
}

#[test]
fn test_nested_borrows() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/nested_borrows.rs");
}

#[test]
fn test_borrow_in_function_call() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/borrow_in_call.rs");
}

#[test]
fn test_mixed_borrows() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/mixed_borrows.rs");
}
