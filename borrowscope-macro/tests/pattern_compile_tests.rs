//! Compile tests for pattern handling

#[test]
fn test_tuple_pattern() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/tuple_pattern.rs");
}

#[test]
fn test_nested_tuple_pattern() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/nested_tuple_pattern.rs");
}

#[test]
fn test_struct_pattern() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/struct_pattern.rs");
}

#[test]
fn test_mixed_pattern() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/mixed_pattern.rs");
}

#[test]
fn test_tuple_with_move() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/tuple_with_move.rs");
}

#[test]
fn test_pattern_in_scope() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/pattern_in_scope.rs");
}
