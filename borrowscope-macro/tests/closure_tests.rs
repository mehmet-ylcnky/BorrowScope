//! Compile tests for closure capture analysis

#[test]
fn test_closure_by_reference() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/closure_by_reference.rs");
}

#[test]
fn test_move_closure() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/move_closure.rs");
}

#[test]
fn test_closure_with_multiple_captures() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/closure_multiple_captures.rs");
}

#[test]
fn test_nested_closures() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/nested_closures.rs");
}

#[test]
fn test_closure_as_argument() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/closure_as_argument.rs");
}

#[test]
fn test_closure_with_method_call() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/closure_with_method.rs");
}
