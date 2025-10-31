//! Compile tests for error reporting

#[test]
fn test_const_fn_error() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile/fail/const_fn.rs");
}

#[test]
fn test_extern_fn_error() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile/fail/extern_fn.rs");
}

#[test]
fn test_async_fn_works() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/async_fn.rs");
}

#[test]
fn test_unsafe_fn_works() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/unsafe_fn.rs");
}
