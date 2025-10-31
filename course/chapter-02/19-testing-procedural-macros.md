# Section 19: Testing Procedural Macros

## Learning Objectives

By the end of this section, you will:
- Test procedural macros comprehensively
- Use trybuild for compile tests
- Implement snapshot testing
- Test error cases
- Achieve high test coverage

## Prerequisites

- Completed Section 18
- Understanding of Rust testing

---

## Testing Strategy

### Test Pyramid

```
     /\
    /E2E\      10% - End-to-end with runtime
   /______\
  /        \
 /Integration\ 30% - Macro + generated code
/__________\
/            \
/  Unit Tests  \ 60% - Individual functions
/________________\
```

---

## Unit Tests

### File: `borrowscope-macro/src/lib.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use syn::parse_quote;

    #[test]
    fn test_simple_transformation() {
        let input = quote! {
            fn example() {
                let x = 5;
            }
        };
        
        let output = trace_borrow(Default::default(), input.into());
        let output_str = output.to_string();
        
        assert!(output_str.contains("track_new"));
        assert!(output_str.contains("track_drop"));
    }
}
```

---

## Compile Tests with trybuild

### File: `borrowscope-macro/tests/compile_tests.rs`

```rust
#[test]
fn compile_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/pass/*.rs");
    t.compile_fail("tests/ui/fail/*.rs");
}
```

### Pass Tests

**File: `tests/ui/pass/simple.rs`**
```rust
use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn example() {
    let x = 5;
}

fn main() {}
```

### Fail Tests

**File: `tests/ui/fail/async.rs`**
```rust
use borrowscope_macro::trace_borrow;

#[trace_borrow]
async fn example() {
    let x = 5;
}

fn main() {}
```

---

## Snapshot Testing

### File: `borrowscope-macro/tests/snapshots.rs`

```rust
use insta::assert_snapshot;

#[test]
fn test_snapshot_simple() {
    let input = quote::quote! {
        fn example() {
            let x = 5;
        }
    };
    
    let output = borrowscope_macro::trace_borrow(
        Default::default(),
        input.into()
    );
    
    assert_snapshot!(output.to_string());
}
```

---

## Integration Tests

### File: `borrowscope-macro/tests/integration.rs`

```rust
use borrowscope_macro::trace_borrow;
use borrowscope_runtime::{reset, get_events};

#[test]
fn test_with_runtime() {
    #[trace_borrow]
    fn example() {
        let x = 5;
    }
    
    reset();
    example();
    
    let events = get_events();
    assert!(!events.is_empty());
}
```

---

## Key Takeaways

✅ **Unit tests** - Test individual functions  
✅ **Compile tests** - Verify compilation  
✅ **Snapshot tests** - Track output changes  
✅ **Integration tests** - Test with runtime  
✅ **High coverage** - Test all paths  

---

**Previous:** [18-macro-hygiene-and-best-practices.md](./18-macro-hygiene-and-best-practices.md)  
**Next:** [20-testing-procedural-macros.md](./20-testing-procedural-macros.md)

**Progress:** 11/12 ⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬜
