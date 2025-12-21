//! Integration tests for track_move injection (Section 40)

#[test]
fn test_simple_move() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/simple_move.rs");
}

#[test]
fn test_move_chain() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/move_chain.rs");
}

#[test]
fn test_move_string() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/move_string.rs");
}

#[test]
fn test_move_vec() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/move_vec.rs");
}

#[test]
fn test_copy_type_assignment() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/copy_type_assignment.rs");
}

#[test]
fn test_mixed_move_and_new() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/mixed_move_new.rs");
}
