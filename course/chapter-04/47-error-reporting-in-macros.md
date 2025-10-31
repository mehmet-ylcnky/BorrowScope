# Section 47: Error Reporting in Macros

## Learning Objectives

By the end of this section, you will:
- Generate helpful error messages
- Use span information for precise errors
- Implement compile_error! for fatal errors
- Provide suggestions and hints
- Create user-friendly diagnostics

## Prerequisites

- Completed Section 46 (Macro Expansion)
- Understanding of compiler errors
- Familiarity with proc_macro2::Span

---

## Error Reporting Basics

Procedural macros can report errors in two ways:

1. **Compile errors** - Stop compilation
2. **Warnings** - Allow compilation to continue

---

## Using Spans

Spans point to specific code locations:

```rust
use proc_macro2::Span;
use syn::spanned::Spanned;

impl OwnershipVisitor {
    fn report_error(&self, span: Span, message: &str) {
        // Create a compile error at the given span
        let error = syn::Error::new(span, message);
        // Store for later emission
    }
}
```

---

## Error Types

### 1. Unsupported Syntax

```rust
impl OwnershipVisitor {
    fn check_supported(&self, item: &ItemFn) -> Result<(), syn::Error> {
        // Check for unsupported features
        if item.sig.asyncness.is_some() {
            return Err(syn::Error::new(
                item.sig.span(),
                "async functions are not fully supported yet"
            ));
        }
        
        if item.sig.unsafety.is_some() {
            // Warning, not error
            eprintln!("Warning: unsafe functions have limited tracking");
        }
        
        Ok(())
    }
}
```

### 2. Invalid Patterns

```rust
impl OwnershipVisitor {
    fn validate_pattern(&self, pat: &Pat) -> Result<(), syn::Error> {
        match pat {
            Pat::Wild(_) => {
                Err(syn::Error::new(
                    pat.span(),
                    "wildcard patterns cannot be tracked"
                ))
            }
            Pat::Rest(_) => {
                Err(syn::Error::new(
                    pat.span(),
                    "rest patterns (..) are not supported"
                ))
            }
            _ => Ok(())
        }
    }
}
```

### 3. Conflicting Attributes

```rust
impl OwnershipVisitor {
    fn check_attributes(&self, item: &ItemFn) -> Result<(), syn::Error> {
        let has_test = item.attrs.iter().any(|attr| {
            attr.path().is_ident("test")
        });
        
        let has_bench = item.attrs.iter().any(|attr| {
            attr.path().is_ident("bench")
        });
        
        if has_test && has_bench {
            return Err(syn::Error::new(
                item.sig.span(),
                "cannot use #[track_ownership] with both #[test] and #[bench]"
            ));
        }
        
        Ok(())
    }
}
```

---

## Implementation

```rust
use syn::Error as SynError;
use proc_macro2::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn track_ownership(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut func = match syn::parse2::<ItemFn>(item.clone()) {
        Ok(f) => f,
        Err(e) => return e.to_compile_error(),
    };
    
    // Validate input
    if let Err(e) = validate_function(&func) {
        return e.to_compile_error();
    }
    
    // Transform
    let mut visitor = OwnershipVisitor::new();
    
    match visitor.transform_function(&mut func) {
        Ok(()) => quote! { #func },
        Err(e) => e.to_compile_error(),
    }
}

fn validate_function(func: &ItemFn) -> Result<(), SynError> {
    // Check for unsupported features
    if func.sig.constness.is_some() {
        return Err(SynError::new(
            func.sig.span(),
            "const functions cannot be tracked"
        ));
    }
    
    if func.sig.abi.is_some() {
        return Err(SynError::new(
            func.sig.span(),
            "extern functions cannot be tracked"
        ));
    }
    
    Ok(())
}

impl OwnershipVisitor {
    fn transform_function(&mut self, func: &mut ItemFn) -> Result<(), SynError> {
        // Validate before transforming
        self.validate_function_body(&func.block)?;
        
        // Transform
        self.visit_item_fn_mut(func);
        
        Ok(())
    }
    
    fn validate_function_body(&self, block: &Block) -> Result<(), SynError> {
        for stmt in &block.stmts {
            self.validate_statement(stmt)?;
        }
        Ok(())
    }
    
    fn validate_statement(&self, stmt: &Stmt) -> Result<(), SynError> {
        match stmt {
            Stmt::Item(_) => {
                return Err(SynError::new(
                    stmt.span(),
                    "nested items are not supported"
                ));
            }
            _ => Ok(())
        }
    }
}
```

---

## Helpful Error Messages

### Bad Error

```rust
Err(SynError::new(span, "error"))
```

**Output:**
```
error: error
 --> src/main.rs:5:9
```

### Good Error

```rust
Err(SynError::new(
    span,
    "cannot track wildcard pattern `_`\n\
     help: use a named variable instead: `let x = ...`"
))
```

**Output:**
```
error: cannot track wildcard pattern `_`
 --> src/main.rs:5:9
  |
5 |     let _ = 42;
  |         ^
  |
  = help: use a named variable instead: `let x = ...`
```

---

## Multiple Errors

Collect all errors before returning:

```rust
impl OwnershipVisitor {
    fn validate_all(&self, func: &ItemFn) -> Result<(), Vec<SynError>> {
        let mut errors = Vec::new();
        
        // Check multiple things
        if let Err(e) = self.check_attributes(func) {
            errors.push(e);
        }
        
        if let Err(e) = self.check_signature(func) {
            errors.push(e);
        }
        
        if let Err(e) = self.check_body(func) {
            errors.push(e);
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

// Combine multiple errors
fn combine_errors(errors: Vec<SynError>) -> SynError {
    let mut combined = errors[0].clone();
    for error in errors.into_iter().skip(1) {
        combined.combine(error);
    }
    combined
}
```

---

## Warning Messages

Warnings don't stop compilation:

```rust
impl OwnershipVisitor {
    fn emit_warning(&self, span: Span, message: &str) {
        // Use eprintln! for warnings
        eprintln!(
            "warning: {}\n  --> {}:{}:{}",
            message,
            span.start().line,
            span.start().column,
            span.end().column
        );
    }
    
    fn check_for_warnings(&self, func: &ItemFn) {
        // Warn about potentially problematic patterns
        if func.sig.asyncness.is_some() {
            self.emit_warning(
                func.sig.span(),
                "async functions have limited tracking support"
            );
        }
        
        if func.sig.unsafety.is_some() {
            self.emit_warning(
                func.sig.span(),
                "unsafe functions cannot verify borrow safety"
            );
        }
    }
}
```

---

## Testing Error Messages

```rust
#[test]
fn test_error_const_fn() {
    let input = quote! {
        #[track_ownership]
        const fn example() {
            let x = 42;
        }
    };
    
    let result = track_ownership(TokenStream::new(), input);
    let output = result.to_string();
    
    assert!(output.contains("const functions cannot be tracked"));
}

#[test]
fn test_error_nested_item() {
    let input = quote! {
        #[track_ownership]
        fn example() {
            fn nested() {}
        }
    };
    
    let result = track_ownership(TokenStream::new(), input);
    let output = result.to_string();
    
    assert!(output.contains("nested items are not supported"));
}
```

---

## Compile Tests

Use `trybuild` to test error messages:

```rust
// tests/compile_fail/const_fn.rs
use borrowscope_macro::track_ownership;

#[track_ownership]
const fn example() {
    let x = 42;
}

fn main() {}
```

```rust
// tests/compile_test.rs
#[test]
fn compile_fail_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/*.rs");
}
```

**Expected output file:**
```
// tests/compile_fail/const_fn.stderr
error: const functions cannot be tracked
 --> tests/compile_fail/const_fn.rs:3:1
  |
3 | const fn example() {
  | ^^^^^
```

---

## User-Friendly Messages

### Example 1: Suggest Fix

```rust
if matches!(pat, Pat::Wild(_)) {
    return Err(SynError::new(
        pat.span(),
        "cannot track wildcard pattern `_`\n\
         \n\
         help: give the variable a name:\n\
         \n\
         let x = ...;\n\
         \n\
         or use `#[allow(unused_variables)]` if intentionally unused"
    ));
}
```

### Example 2: Explain Why

```rust
if func.sig.constness.is_some() {
    return Err(SynError::new(
        func.sig.span(),
        "const functions cannot be tracked\n\
         \n\
         reason: tracking requires runtime operations,\n\
         but const functions are evaluated at compile time"
    ));
}
```

### Example 3: Point to Documentation

```rust
if func.sig.asyncness.is_some() {
    return Err(SynError::new(
        func.sig.span(),
        "async functions are not fully supported\n\
         \n\
         note: basic tracking works, but await points are not tracked\n\
         see: https://docs.rs/borrowscope/latest/borrowscope/#async-support"
    ));
}
```

---

## Error Recovery

Try to continue after errors when possible:

```rust
impl OwnershipVisitor {
    fn transform_with_recovery(&mut self, func: &mut ItemFn) -> TokenStream {
        let mut errors = Vec::new();
        
        // Try to transform each statement
        for stmt in &mut func.block.stmts {
            if let Err(e) = self.transform_statement(stmt) {
                errors.push(e);
                // Continue with next statement
            }
        }
        
        if errors.is_empty() {
            quote! { #func }
        } else {
            // Return original code + errors
            let error_tokens = errors.into_iter()
                .map(|e| e.to_compile_error())
                .collect::<Vec<_>>();
            
            quote! {
                #(#error_tokens)*
                #func
            }
        }
    }
}
```

---

## Key Takeaways

✅ **Use spans** - Point to exact error location  
✅ **Be helpful** - Explain why and suggest fixes  
✅ **Collect errors** - Show multiple errors at once  
✅ **Test errors** - Use trybuild for compile tests  
✅ **Recover when possible** - Don't stop at first error  

---

## Further Reading

- [syn::Error](https://docs.rs/syn/latest/syn/struct.Error.html)
- [Compiler error format](https://doc.rust-lang.org/rustc/json.html)
- [trybuild](https://docs.rs/trybuild/)
- [Error handling in proc macros](https://docs.rs/syn/latest/syn/parse/index.html#error-reporting)

---

**Previous:** [46-macro-expansion-considerations.md](./46-macro-expansion-considerations.md)  
**Next:** [48-optimizing-generated-code.md](./48-optimizing-generated-code.md)

**Progress:** 12/15 ⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬜⬜⬜
