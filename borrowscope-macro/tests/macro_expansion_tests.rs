//! Compile tests for macro expansion handling

#[test]
fn test_vec_macro() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/vec_macro.rs");
}

#[test]
fn test_format_macro() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/format_macro.rs");
}

#[test]
fn test_println_macro() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/println_macro.rs");
}

#[test]
fn test_assert_macro() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/assert_macro.rs");
}

#[test]
fn test_multiple_macros() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/multiple_macros.rs");
}

#[test]
fn test_nested_macros() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/pass/nested_macros.rs");
}
