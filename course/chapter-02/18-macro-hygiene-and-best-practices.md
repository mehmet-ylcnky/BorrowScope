# Section 18: Macro Hygiene and Best Practices

## Learning Objectives

By the end of this section, you will:
- Understand macro hygiene
- Avoid name collisions
- Follow procedural macro best practices
- Handle edge cases properly
- Write maintainable macro code

## Prerequisites

- Completed Section 17
- Understanding of scope and namespaces

---

## Macro Hygiene

### What is Hygiene?

**Hygiene** prevents macros from accidentally capturing or conflicting with user variables.

**Problem without hygiene:**
```rust
// Macro generates:
let x = 5;

// User code:
let x = 10;
// Which x is which?
```

### Rust's Solution

Rust uses **spans** to track where code came from:
- Macro-generated code has macro's span
- User code has user's span
- They don't conflict even with same names

---

## Avoiding Name Collisions

### Use Unique Prefixes

```rust
// ❌ Bad: Generic name
let temp = value;

// ✅ Good: Unique prefix
let __borrowscope_temp = value;
```

### Implementation

```rust
use syn::Ident;
use proc_macro2::Span;

/// Generate unique temporary name
pub fn generate_unique_name(prefix: &str, counter: usize) -> Ident {
    let name = format!("__borrowscope_{}_{}", prefix, counter);
    Ident::new(&name, Span::call_site())
}

#[test]
fn test_unique_names() {
    let name1 = generate_unique_name("temp", 0);
    let name2 = generate_unique_name("temp", 1);
    
    assert_eq!(name1.to_string(), "__borrowscope_temp_0");
    assert_eq!(name2.to_string(), "__borrowscope_temp_1");
}
```

---

## Best Practices

### 1. Preserve User Code

```rust
// ✅ Good: Preserve attributes
#(#attrs)*
#vis #sig {
    #block
}

// ❌ Bad: Lose attributes
fn #name() {
    #block
}
```

### 2. Minimal Transformation

```rust
// ✅ Good: Only transform what's needed
if should_transform(expr) {
    transform(expr)
} else {
    expr // Leave unchanged
}

// ❌ Bad: Transform everything
transform_all(expr)
```

### 3. Clear Error Messages

```rust
// ✅ Good: Specific error with span
return Err(Error::new_spanned(
    &func.sig.asyncness,
    "trace_borrow does not support async functions"
));

// ❌ Bad: Generic error
return Err(Error::new(
    Span::call_site(),
    "Invalid function"
));
```

---

## Complete Best Practices Guide

### File: `borrowscope-macro/src/best_practices.rs`

```rust
//! Best practices for macro development

use syn::{ItemFn, Error};
use proc_macro2::Span;

/// Validate function before transformation
pub fn validate_before_transform(func: &ItemFn) -> syn::Result<()> {
    // Check async
    if func.sig.asyncness.is_some() {
        return Err(Error::new_spanned(
            &func.sig.asyncness,
            "trace_borrow does not support async functions yet"
        ));
    }
    
    // Check unsafe
    if func.sig.unsafety.is_some() {
        return Err(Error::new_spanned(
            &func.sig.unsafety,
            "trace_borrow cannot be used on unsafe functions"
        ));
    }
    
    // Check const
    if func.sig.constness.is_some() {
        return Err(Error::new_spanned(
            &func.sig.constness,
            "trace_borrow cannot be used on const functions"
        ));
    }
    
    Ok(())
}

/// Check if transformation is safe
pub fn is_safe_to_transform(func: &ItemFn) -> bool {
    // No async, unsafe, or const
    func.sig.asyncness.is_none() &&
    func.sig.unsafety.is_none() &&
    func.sig.constness.is_none()
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_validate_simple_function() {
        let func: ItemFn = parse_quote! {
            fn example() {}
        };
        assert!(validate_before_transform(&func).is_ok());
    }

    #[test]
    fn test_validate_async_function() {
        let func: ItemFn = parse_quote! {
            async fn example() {}
        };
        assert!(validate_before_transform(&func).is_err());
    }
}
```

---

## Key Takeaways

✅ **Hygiene** - Use unique prefixes  
✅ **Preservation** - Keep user code intact  
✅ **Validation** - Check before transforming  
✅ **Error messages** - Clear and specific  
✅ **Minimal changes** - Only transform what's needed  

---

**Previous:** [17-code-generation-with-quote.md](./17-code-generation-with-quote.md)  
**Next:** [19-testing-procedural-macros.md](./19-testing-procedural-macros.md)

**Progress:** 10/12 ⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬜⬜
