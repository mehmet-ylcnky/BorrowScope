# Section 46: Macro Expansion Considerations

## Learning Objectives

By the end of this section, you will:
- Handle nested macro expansions
- Understand macro hygiene implications
- Deal with macro-generated identifiers
- Preserve macro expansion order
- Work with already-expanded code

## Prerequisites

- Completed Section 45 (Closures)
- Understanding of Rust macro system
- Familiarity with macro hygiene

---

## Macro Expansion Order

**Key insight:** By the time our procedural macro runs, most declarative macros are already expanded.

```rust
#[track_ownership]
fn example() {
    let v = vec![1, 2, 3];  // vec! already expanded
}
```

**What we see:**
```rust
fn example() {
    let v = <[_]>::into_vec(box [1, 2, 3]);
}
```

---

## Macro Hygiene

Rust macros are hygienic - they don't accidentally capture variables:

```rust
macro_rules! create_var {
    () => {
        let x = 42;  // This x is in a different hygiene context
    };
}

fn example() {
    create_var!();
    let x = 100;  // Different x
}
```

**After expansion:**
```rust
fn example() {
    let x_hygiene_1 = 42;
    let x = 100;
}
```

**Impact on tracking:** We see the expanded identifiers, which may have hygiene markers.

---

## Detection Strategy

```rust
impl OwnershipVisitor {
    fn is_macro_generated(&self, ident: &Ident) -> bool {
        // Check if identifier has hygiene information
        // In practice, we can't easily detect this
        false
    }
    
    fn is_common_macro_pattern(&self, expr: &Expr) -> bool {
        // Detect patterns from common macros
        match expr {
            Expr::Call(call) => {
                if let Expr::Path(path) = &*call.func {
                    let path_str = quote!(#path).to_string();
                    
                    // vec! expansion
                    if path_str.contains("into_vec") {
                        return true;
                    }
                    
                    // format! expansion
                    if path_str.contains("format_args") {
                        return true;
                    }
                }
            }
            _ => {}
        }
        false
    }
}
```

---

## Handling Common Macros

### vec!

**Original:**
```rust
let v = vec![1, 2, 3];
```

**Expanded:**
```rust
let v = <[_]>::into_vec(box [1, 2, 3]);
```

**Our transformation:**
```rust
let v = track_new(1, "v", "Vec<i32>", "line:1:9", <[_]>::into_vec(box [1, 2, 3]));
```

**Works automatically!** We just track the expanded form.

### println!

**Original:**
```rust
println!("Hello, {}", name);
```

**Expanded:**
```rust
{
    ::std::io::_print(::std::fmt::Arguments::new_v1(
        &["Hello, "],
        &[::std::fmt::ArgumentV1::new_display(&name)]
    ));
}
```

**Our transformation:** Track the `name` variable usage.

### format!

**Original:**
```rust
let s = format!("Value: {}", x);
```

**Expanded:**
```rust
let s = ::std::fmt::format(::std::fmt::Arguments::new_v1(...));
```

**Our transformation:** Track `s` creation normally.

---

## Implementation

```rust
impl OwnershipVisitor {
    fn handle_macro_expansion(&mut self, expr: &mut Expr) {
        // Check for common macro patterns
        if self.is_vec_macro(expr) {
            self.handle_vec_macro(expr);
        } else if self.is_format_macro(expr) {
            self.handle_format_macro(expr);
        } else {
            // Default: treat as normal expression
            visit_mut::visit_expr_mut(self, expr);
        }
    }
    
    fn is_vec_macro(&self, expr: &Expr) -> bool {
        if let Expr::Call(call) = expr {
            if let Expr::Path(path) = &*call.func {
                let path_str = quote!(#path).to_string();
                return path_str.contains("into_vec");
            }
        }
        false
    }
    
    fn handle_vec_macro(&mut self, expr: &mut Expr) {
        // vec! creates a Vec, so we can infer the type
        // Just track it normally
        visit_mut::visit_expr_mut(self, expr);
    }
    
    fn is_format_macro(&self, expr: &Expr) -> bool {
        if let Expr::Call(call) = expr {
            if let Expr::Path(path) = &*call.func {
                let path_str = quote!(#path).to_string();
                return path_str.contains("fmt") && path_str.contains("format");
            }
        }
        false
    }
    
    fn handle_format_macro(&mut self, expr: &mut Expr) {
        // format! returns String
        // Track normally
        visit_mut::visit_expr_mut(self, expr);
    }
}
```

---

## Nested Macros

```rust
macro_rules! outer {
    () => {
        inner!()
    };
}

macro_rules! inner {
    () => {
        let x = 42;
    };
}

#[track_ownership]
fn example() {
    outer!();
}
```

**After full expansion:**
```rust
fn example() {
    let x = 42;
}
```

**Our transformation:** Works normally - we see the final expanded code.

---

## Procedural Macros

```rust
#[derive(Debug, Clone)]
struct Point {
    x: i32,
    y: i32,
}

#[track_ownership]
fn example() {
    let p = Point { x: 1, y: 2 };
}
```

**Expansion order:**
1. `#[derive(Debug, Clone)]` generates impl blocks
2. `#[track_ownership]` sees the struct definition and impl blocks

**Impact:** We don't see inside the generated impl blocks, which is fine.

---

## Macro Hygiene Implications

### Hygienic Identifiers

```rust
macro_rules! make_var {
    ($name:ident) => {
        let $name = 42;
    };
}

#[track_ownership]
fn example() {
    make_var!(x);
    println!("{}", x);
}
```

**After expansion:**
```rust
fn example() {
    let x = 42;  // Same hygiene context
    println!("{}", x);
}
```

**Our transformation:** Works normally.

### Unhygienic Macros

Some macros intentionally break hygiene:

```rust
macro_rules! declare_x {
    () => {
        let x = 42;
    };
}
```

**Our transformation:** We track whatever identifiers we see.

---

## Testing

```rust
#[test]
fn test_vec_macro() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            let v = vec![1, 2, 3];
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    
    // vec! is already expanded
    assert!(output.contains("track_new"));
}

#[test]
fn test_format_macro() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            let x = 42;
            let s = format!("Value: {}", x);
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    
    // Both x and s should be tracked
    assert!(output.matches("track_new").count() >= 2);
}
```

---

## Debugging Macro Expansions

Use `cargo expand` to see what our macro receives:

```bash
# Expand all macros
cargo expand

# Expand specific function
cargo expand example

# Expand with our macro applied
cargo expand --lib
```

**Example output:**
```rust
fn example() {
    let v = <[_]>::into_vec(box [1, 2, 3]);
}
```

---

## Best Practices

### 1. Work with Expanded Code

Don't try to detect original macro invocations. Work with what you receive.

### 2. Test with Common Macros

```rust
#[test]
fn test_common_macros() {
    test_with_vec_macro();
    test_with_format_macro();
    test_with_println_macro();
    test_with_assert_macro();
}
```

### 3. Document Macro Behavior

```rust
/// Note: This function uses vec! macro which expands to Vec::from.
/// The tracking will show the expanded form.
#[track_ownership]
fn example() {
    let v = vec![1, 2, 3];
}
```

### 4. Handle Macro-Generated Patterns

```rust
impl OwnershipVisitor {
    fn is_likely_macro_generated(&self, expr: &Expr) -> bool {
        // Check for patterns that indicate macro generation
        match expr {
            Expr::Call(call) => {
                // Check for fully qualified paths (common in macros)
                if let Expr::Path(path) = &*call.func {
                    let path_str = quote!(#path).to_string();
                    return path_str.starts_with("::");
                }
            }
            _ => {}
        }
        false
    }
}
```

---

## Limitations

### Can't Preserve Macro Names

```rust
// User writes:
let v = vec![1, 2, 3];

// We see:
let v = <[_]>::into_vec(box [1, 2, 3]);

// Error messages show expanded form
```

**Impact:** Error messages and visualization show expanded code, not original macros.

### Can't Track Macro Invocations

```rust
// Can't tell how many times a macro was invoked
for i in 0..3 {
    println!("Iteration {}", i);  // Each println! is expanded
}
```

**Impact:** Can't provide macro-specific insights.

### Can't Handle Compile-Time Evaluation

```rust
const N: usize = 10;
let arr = [0; N];  // N is evaluated at compile time
```

**Impact:** We see the result, not the computation.

---

## Key Takeaways

✅ **Macros expand first** - We see expanded code  
✅ **Hygiene is preserved** - Identifiers may have hygiene markers  
✅ **Work with expanded form** - Don't try to detect original macros  
✅ **Test common macros** - Ensure compatibility  
✅ **Document behavior** - Explain what users will see  

---

## Further Reading

- [Macros](https://doc.rust-lang.org/book/ch19-06-macros.html)
- [Macro hygiene](https://doc.rust-lang.org/reference/macros-by-example.html#hygiene)
- [cargo-expand](https://github.com/dtolnay/cargo-expand)
- [Procedural macros](https://doc.rust-lang.org/reference/procedural-macros.html)

---

**Previous:** [45-closure-capture-analysis.md](./45-closure-capture-analysis.md)  
**Next:** [47-error-reporting-in-macros.md](./47-error-reporting-in-macros.md)

**Progress:** 11/15 ⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬜⬜⬜⬜
