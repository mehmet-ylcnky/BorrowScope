//! Compile tests for control flow handling

#[test]
fn test_if_expression() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/if_expression.rs");
}

#[test]
fn test_if_else() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/if_else.rs");
}

#[test]
fn test_match_expression() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/match_expression.rs");
}

#[test]
fn test_for_loop() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/for_loop.rs");
}

#[test]
fn test_while_loop() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/while_loop.rs");
}

#[test]
fn test_loop_with_break() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/loop_with_break.rs");
}
