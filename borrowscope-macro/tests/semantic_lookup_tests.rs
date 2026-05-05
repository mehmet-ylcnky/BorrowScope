//! Test that semantic lookup works when analyzer data is present

use borrowscope_macro::trace_borrow;

// This test requires running the analyzer first to generate type-info.json
// For now, we just verify it compiles and runs with heuristic fallback

#[test]
fn test_vec_push_with_semantic_lookup() {
    #[trace_borrow]
    fn example() {
        let mut v = Vec::new();
        v.push(1); // Should detect mutable borrow (semantic or heuristic)
        v.push(2);
    }

    example();
}

#[test]
fn test_vec_len_with_semantic_lookup() {
    #[trace_borrow]
    fn example() {
        let v = vec![1, 2, 3];
        let _len = v.len(); // Should detect immutable borrow (semantic or heuristic)
    }

    example();
}

#[test]
fn test_string_as_str_with_semantic_lookup() {
    #[trace_borrow]
    fn example() {
        let s = String::from("hello");
        let _slice = s.as_str(); // Should detect immutable borrow (semantic or heuristic)
    }

    example();
}

#[test]
fn test_option_unwrap_with_semantic_lookup() {
    #[trace_borrow]
    fn example() {
        let opt = Some(42);
        let _val = opt.unwrap(); // Should detect consuming (semantic or heuristic)
    }

    example();
}
