//! Compile tests for method call transformations

#[test]
fn test_immutable_method() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/immutable_method.rs");
}

#[test]
fn test_mutable_method() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/mutable_method.rs");
}

#[test]
fn test_chained_methods() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/chained_methods.rs");
}

#[test]
fn test_method_with_args() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/method_with_args.rs");
}

#[test]
fn test_consuming_method() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/consuming_method.rs");
}

#[test]
fn test_method_on_temporary() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/method_on_temporary.rs");
}
