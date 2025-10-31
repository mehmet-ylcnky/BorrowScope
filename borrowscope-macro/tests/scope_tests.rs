//! Integration tests for scope boundary handling (Section 41)

#[test]
fn test_simple_scope() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/simple_scope.rs");
}

#[test]
fn test_nested_scopes() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/nested_scopes.rs");
}

#[test]
fn test_lifo_drop_order() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/lifo_drops.rs");
}

#[test]
fn test_empty_scope() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/empty_scope.rs");
}

#[test]
fn test_multiple_nested() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/multiple_nested.rs");
}

#[test]
fn test_scope_with_moves() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/scope_with_moves.rs");
}
