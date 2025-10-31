# Section 38: Injecting track_new Calls

## Learning Objectives

By the end of this section, you will:
- Extract type information from AST
- Generate source location strings
- Handle type inference correctly
- Transform complex initializers
- Preserve all Rust semantics

## Prerequisites

- Completed Section 37 (AST Visitor)
- Understanding of Rust type system
- Familiarity with proc_macro2::Span

---

## The Complete Transformation

We need to transform:

```rust
let x: i32 = 42;
```

Into:

```rust
let x: i32 = borrowscope_runtime::track_new(
    1,              // ID
    "x",            // name
    "i32",          // type
    "main.rs:5:9",  // location
    42              // value
);
```

---

## Extracting Type Information

### Strategy 1: Use Type Annotation

If the user provides a type annotation, use it:

```rust
impl OwnershipVisitor {
    fn extract_type_name(&self, local: &Local) -> String {
        // Check if there's a type annotation
        if let Pat::Type(pat_type) = &local.pat {
            return quote!(#pat_type.ty).to_string();
        }
        
        // No annotation - use placeholder
        "unknown".to_string()
    }
}
```

**Example:**
```rust
let x: i32 = 42;  // Type is "i32"
let y = 42;       // Type is "unknown"
```

### Strategy 2: Stringify the Type

For better output, use `std::any::type_name`:

```rust
// In the generated code
let x = borrowscope_runtime::track_new(
    1,
    "x",
    std::any::type_name::<i32>(),  // Runtime type name
    "main.rs:5:9",
    42
);
```

**Problem:** This requires the type to be known at macro expansion time.

### Strategy 3: Use String Literal

Best approach for proc macros:

```rust
impl OwnershipVisitor {
    fn extract_type_string(&self, local: &Local) -> String {
        match &local.pat {
            Pat::Type(pat_type) => {
                // Convert type to string
                quote!(#pat_type.ty).to_string()
                    .replace(" ", "")  // Remove whitespace
            }
            _ => "inferred".to_string(),
        }
    }
}
```

---

## Extracting Source Location

Use `proc_macro2::Span` to get source location:

```rust
use proc_macro2::Span;

impl OwnershipVisitor {
    fn get_location(&self, span: Span) -> String {
        let start = span.start();
        format!("{}:{}:{}", 
            "file.rs",  // We'll improve this
            start.line,
            start.column
        )
    }
}
```

**Limitation:** proc_macro2 doesn't provide file names in stable Rust.

**Workaround:** Use a placeholder or require nightly features.

---

## Complete Implementation

Update `borrowscope-macro/src/visitor.rs`:

```rust
use syn::{
    visit_mut::{self, VisitMut},
    Expr, Stmt, Block, Local, Pat, ItemFn, Type,
};
use quote::quote;
use proc_macro2::Span;

pub struct OwnershipVisitor {
    next_id: usize,
    scope_depth: usize,
}

impl OwnershipVisitor {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            scope_depth: 0,
        }
    }
    
    fn next_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
    
    fn extract_var_name(&self, pat: &Pat) -> String {
        match pat {
            Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
            Pat::Type(pat_type) => self.extract_var_name(&pat_type.pat),
            _ => "unknown".to_string(),
        }
    }
    
    fn extract_type_name(&self, pat: &Pat) -> String {
        match pat {
            Pat::Type(pat_type) => {
                quote!(#pat_type.ty).to_string()
                    .replace(" ", "")
            }
            _ => "inferred".to_string(),
        }
    }
    
    fn get_location(&self, span: Span) -> String {
        let start = span.start();
        format!("line:{}:{}", start.line, start.column)
    }
}

impl VisitMut for OwnershipVisitor {
    fn visit_item_fn_mut(&mut self, func: &mut ItemFn) {
        self.visit_block_mut(&mut func.block);
    }
    
    fn visit_block_mut(&mut self, block: &mut Block) {
        self.scope_depth += 1;
        
        for stmt in &mut block.stmts {
            self.visit_stmt_mut(stmt);
        }
        
        self.scope_depth -= 1;
    }
    
    fn visit_stmt_mut(&mut self, stmt: &mut Stmt) {
        match stmt {
            Stmt::Local(local) => {
                self.transform_local(local);
            }
            _ => {
                visit_mut::visit_stmt_mut(self, stmt);
            }
        }
    }
    
    fn transform_local(&mut self, local: &mut Local) {
        // Only transform if there's an initializer
        if let Some(init) = &mut local.init {
            let id = self.next_id();
            let var_name = self.extract_var_name(&local.pat);
            let type_name = self.extract_type_name(&local.pat);
            let location = self.get_location(local.pat.span());
            
            let original_expr = &init.expr;
            
            // Generate track_new call
            let new_expr: Expr = syn::parse_quote! {
                borrowscope_runtime::track_new(
                    #id,
                    #var_name,
                    #type_name,
                    #location,
                    #original_expr
                )
            };
            
            *init.expr = new_expr;
        }
        
        // Visit nested expressions
        visit_mut::visit_local_mut(self, local);
    }
}
```

---

## Testing

Create `borrowscope-macro/tests/track_new_test.rs`:

```rust
use borrowscope_macro::OwnershipVisitor;
use syn::{parse_quote, visit_mut::VisitMut};
use quote::ToTokens;

#[test]
fn test_simple_variable() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut stmt: syn::Stmt = parse_quote! {
        let x = 42;
    };
    
    visitor.visit_stmt_mut(&mut stmt);
    
    let output = stmt.to_token_stream().to_string();
    
    assert!(output.contains("track_new"));
    assert!(output.contains("\"x\""));
    assert!(output.contains("42"));
}

#[test]
fn test_typed_variable() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut stmt: syn::Stmt = parse_quote! {
        let x: i32 = 42;
    };
    
    visitor.visit_stmt_mut(&mut stmt);
    
    let output = stmt.to_token_stream().to_string();
    
    assert!(output.contains("track_new"));
    assert!(output.contains("\"x\""));
    assert!(output.contains("i32"));
}

#[test]
fn test_complex_expression() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut stmt: syn::Stmt = parse_quote! {
        let x = 1 + 2 * 3;
    };
    
    visitor.visit_stmt_mut(&mut stmt);
    
    let output = stmt.to_token_stream().to_string();
    
    assert!(output.contains("track_new"));
    assert!(output.contains("1 + 2 * 3"));
}

#[test]
fn test_string_literal() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut stmt: syn::Stmt = parse_quote! {
        let s = String::from("hello");
    };
    
    visitor.visit_stmt_mut(&mut stmt);
    
    let output = stmt.to_token_stream().to_string();
    
    assert!(output.contains("track_new"));
    assert!(output.contains("String :: from"));
}

#[test]
fn test_unique_ids() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            let x = 1;
            let y = 2;
            let z = 3;
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    
    // Should have three different IDs
    assert!(output.contains("track_new (1"));
    assert!(output.contains("track_new (2"));
    assert!(output.contains("track_new (3"));
}
```

Run tests:

```bash
cargo test --package borrowscope-macro track_new_test
```

---

## Integration Test

Create `borrowscope-macro/tests/integration/track_new_integration.rs`:

```rust
use borrowscope_runtime::*;

#[test]
fn test_track_new_integration() {
    reset_tracker();
    
    // Simulate what the macro generates
    let x = track_new(1, "x", "i32", "test.rs:1:1", 42);
    
    assert_eq!(x, 42);
    
    let events = get_events();
    assert_eq!(events.len(), 1);
    
    match &events[0] {
        Event::New { id, name, type_name, .. } => {
            assert_eq!(*id, 1);
            assert_eq!(name, "x");
            assert_eq!(type_name, "i32");
        }
        _ => panic!("Expected New event"),
    }
}

#[test]
fn test_multiple_variables() {
    reset_tracker();
    
    let x = track_new(1, "x", "i32", "test.rs:1:1", 42);
    let y = track_new(2, "y", "i32", "test.rs:2:1", 100);
    
    assert_eq!(x, 42);
    assert_eq!(y, 100);
    
    let events = get_events();
    assert_eq!(events.len(), 2);
}
```

---

## Handling Edge Cases

### Case 1: No Initializer

```rust
let x;  // Uninitialized
x = 42;
```

**Solution:** Don't transform declarations without initializers.

```rust
fn transform_local(&mut self, local: &mut Local) {
    if local.init.is_none() {
        return;  // Skip uninitialized variables
    }
    // ... rest of transformation
}
```

### Case 2: Mutable Variables

```rust
let mut x = 42;
```

**Solution:** Preserve the `mut` keyword.

```rust
// The transformation doesn't affect mutability
let mut x = track_new(1, "x", "i32", "test.rs:1:1", 42);
```

### Case 3: Complex Types

```rust
let v: Vec<String> = vec!["hello".to_string()];
```

**Solution:** Type extraction handles this automatically.

```rust
let type_name = self.extract_type_name(&local.pat);
// type_name = "Vec<String>"
```

### Case 4: Generic Types

```rust
fn example<T>(value: T) {
    let x = value;
}
```

**Solution:** Type name will be "inferred" or "T".

---

## Example Output

**Input:**
```rust
#[track_ownership]
fn example() {
    let x: i32 = 42;
    let y = String::from("hello");
    let mut z = vec![1, 2, 3];
}
```

**Output:**
```rust
fn example() {
    let x: i32 = borrowscope_runtime::track_new(
        1,
        "x",
        "i32",
        "line:3:9",
        42
    );
    let y = borrowscope_runtime::track_new(
        2,
        "y",
        "inferred",
        "line:4:9",
        String::from("hello")
    );
    let mut z = borrowscope_runtime::track_new(
        3,
        "z",
        "inferred",
        "line:5:13",
        vec![1, 2, 3]
    );
}
```

---

## Performance Considerations

### Inline Tracking Functions

The runtime functions are marked `#[inline(always)]`:

```rust
#[inline(always)]
pub fn track_new<T>(id: usize, name: &str, type_name: &str, location: &str, value: T) -> T {
    // ... tracking logic ...
    value
}
```

This ensures zero overhead in release builds.

### String Allocation

Each tracking call allocates strings for name, type, and location.

**Optimization:** Use `&'static str` when possible:

```rust
// Macro generates string literals
track_new(1, "x", "i32", "line:3:9", 42)
//           ^^^   ^^^^^   ^^^^^^^^^^
//           All are &'static str
```

---

## Key Takeaways

✅ **Extract type from annotations** - Use Pat::Type  
✅ **Generate source locations** - Use Span::start()  
✅ **Preserve semantics** - Wrap expressions, don't replace  
✅ **Handle edge cases** - Uninitialized, mutable, generic  
✅ **Test thoroughly** - Unit and integration tests  

---

## Further Reading

- [syn::Pat documentation](https://docs.rs/syn/latest/syn/enum.Pat.html)
- [proc_macro2::Span](https://docs.rs/proc-macro2/latest/proc_macro2/struct.Span.html)
- [Rust type system](https://doc.rust-lang.org/book/ch10-00-generics.html)

---

**Previous:** [37-implementing-the-ast-visitor.md](./37-implementing-the-ast-visitor.md)  
**Next:** [39-injecting-track-borrow-calls.md](./39-injecting-track-borrow-calls.md)

**Progress:** 3/15 ⬛⬛⬛⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜
