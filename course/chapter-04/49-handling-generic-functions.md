# Section 49: Handling Generic Functions

## Learning Objectives

By the end of this section, you will:
- Transform generic functions correctly
- Handle type parameters
- Deal with lifetime parameters
- Process where clauses
- Preserve generic constraints

## Prerequisites

- Completed Section 48 (Code Optimization)
- Understanding of Rust generics
- Familiarity with trait bounds

---

## Generic Function Basics

```rust
fn example<T>(value: T) -> T {
    let x = value;
    x
}
```

**Challenge:** Type `T` is unknown at macro expansion time.

---

## Detection

```rust
impl OwnershipVisitor {
    fn is_generic(&self, func: &ItemFn) -> bool {
        !func.sig.generics.params.is_empty()
    }
    
    fn extract_type_params(&self, func: &ItemFn) -> Vec<String> {
        func.sig.generics.params.iter()
            .filter_map(|param| {
                if let syn::GenericParam::Type(type_param) = param {
                    Some(type_param.ident.to_string())
                } else {
                    None
                }
            })
            .collect()
    }
}
```

---

## Transformation Strategy

### Simple Generic Function

**Input:**
```rust
#[track_ownership]
fn example<T>(value: T) -> T {
    let x = value;
    x
}
```

**Output:**
```rust
fn example<T>(value: T) -> T {
    let x = borrowscope_runtime::track_new(
        1,
        "x",
        std::any::type_name::<T>(),  // Runtime type name
        "line:3:9",
        value
    );
    borrowscope_runtime::track_drop(1, "scope_end");
    x
}
```

**Key insight:** Use `std::any::type_name::<T>()` to get type name at runtime.

---

## Implementation

```rust
impl OwnershipVisitor {
    fn transform_generic_function(&mut self, func: &mut ItemFn) {
        // Check if function is generic
        if !self.is_generic(func) {
            // Normal transformation
            self.visit_item_fn_mut(func);
            return;
        }
        
        // Extract type parameters
        let type_params = self.extract_type_params(func);
        
        // Store for use in transformations
        self.current_type_params = type_params;
        
        // Transform function body
        self.visit_block_mut(&mut func.block);
        
        // Clear type parameters
        self.current_type_params.clear();
    }
    
    fn generate_type_name_expr(&self, pat: &Pat) -> Expr {
        // Check if we have a type annotation
        if let Pat::Type(pat_type) = pat {
            let ty = &pat_type.ty;
            
            // Check if it's a generic type parameter
            if let Type::Path(type_path) = &**ty {
                if let Some(ident) = type_path.path.get_ident() {
                    let type_name = ident.to_string();
                    
                    if self.current_type_params.contains(&type_name) {
                        // It's a generic parameter - use type_name at runtime
                        return syn::parse_quote! {
                            std::any::type_name::<#ty>()
                        };
                    }
                }
            }
            
            // Concrete type - use string literal
            let type_str = quote!(#ty).to_string();
            return syn::parse_quote! { #type_str };
        }
        
        // No type annotation - use "inferred"
        syn::parse_quote! { "inferred" }
    }
}
```

---

## Type Name Generation

```rust
impl OwnershipVisitor {
    fn transform_local_generic(&mut self, local: &mut Local) {
        if let Some(init) = &mut local.init {
            let id = self.next_id();
            let var_name = self.extract_var_name(&local.pat);
            let location = self.get_location(local.pat.span());
            
            // Generate type name expression
            let type_name_expr = self.generate_type_name_expr(&local.pat);
            
            self.var_ids.insert(var_name.clone(), id);
            
            if let Some(current_scope) = self.scope_stack.last_mut() {
                current_scope.push(id);
            }
            
            let original_expr = &init.expr;
            
            let new_expr: Expr = syn::parse_quote! {
                borrowscope_runtime::track_new(
                    #id,
                    #var_name,
                    #type_name_expr,  // Runtime type name
                    #location,
                    #original_expr
                )
            };
            
            *init.expr = new_expr;
        }
    }
}
```

---

## Lifetime Parameters

```rust
fn example<'a>(x: &'a str) -> &'a str {
    let y = x;
    y
}
```

**Transformation:**
```rust
fn example<'a>(x: &'a str) -> &'a str {
    let y = borrowscope_runtime::track_new(
        1,
        "y",
        "&str",
        "line:2:9",
        x
    );
    borrowscope_runtime::track_drop(1, "scope_end");
    y
}
```

**Note:** Lifetimes don't affect transformation - they're compile-time only.

---

## Where Clauses

```rust
fn example<T>(value: T) -> T
where
    T: Clone + Debug,
{
    let x = value.clone();
    x
}
```

**Transformation:** Preserve where clause:

```rust
fn example<T>(value: T) -> T
where
    T: Clone + Debug,
{
    let x = borrowscope_runtime::track_new(
        1,
        "x",
        std::any::type_name::<T>(),
        "line:5:9",
        value.clone()
    );
    borrowscope_runtime::track_drop(1, "scope_end");
    x
}
```

**Implementation:**
```rust
impl OwnershipVisitor {
    fn visit_item_fn_mut(&mut self, func: &mut ItemFn) {
        // Preserve generics and where clause
        // Only transform the body
        self.visit_block_mut(&mut func.block);
    }
}
```

---

## Multiple Type Parameters

```rust
fn example<T, U>(t: T, u: U) -> (T, U) {
    let x = t;
    let y = u;
    (x, y)
}
```

**Transformation:**
```rust
fn example<T, U>(t: T, u: U) -> (T, U) {
    let x = borrowscope_runtime::track_new(
        1,
        "x",
        std::any::type_name::<T>(),
        "line:2:9",
        t
    );
    let y = borrowscope_runtime::track_new(
        2,
        "y",
        std::any::type_name::<U>(),
        "line:3:9",
        u
    );
    borrowscope_runtime::track_drop(2, "scope_end");
    borrowscope_runtime::track_drop(1, "scope_end");
    (x, y)
}
```

---

## Testing

```rust
#[test]
fn test_generic_function() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut func: syn::ItemFn = parse_quote! {
        fn example<T>(value: T) -> T {
            let x = value;
            x
        }
    };
    
    visitor.visit_item_fn_mut(&mut func);
    
    let output = func.to_token_stream().to_string();
    
    assert!(output.contains("track_new"));
    assert!(output.contains("type_name"));
}

#[test]
fn test_generic_with_bounds() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut func: syn::ItemFn = parse_quote! {
        fn example<T: Clone>(value: T) -> T {
            let x = value.clone();
            x
        }
    };
    
    visitor.visit_item_fn_mut(&mut func);
    
    let output = func.to_token_stream().to_string();
    
    // Should preserve Clone bound
    assert!(output.contains("Clone"));
}
```

---

## Integration Test

```rust
use borrowscope_runtime::*;

fn generic_example<T>(value: T) -> T {
    let x = track_new(1, "x", std::any::type_name::<T>(), "test.rs:1:1", value);
    track_drop(1, "scope_end");
    x
}

#[test]
fn test_generic_with_i32() {
    reset_tracker();
    
    let result = generic_example(42);
    assert_eq!(result, 42);
    
    let events = get_events();
    
    match &events[0] {
        Event::New { type_name, .. } => {
            assert_eq!(type_name, "i32");
        }
        _ => panic!("Expected New event"),
    }
}

#[test]
fn test_generic_with_string() {
    reset_tracker();
    
    let result = generic_example(String::from("hello"));
    assert_eq!(result, "hello");
    
    let events = get_events();
    
    match &events[0] {
        Event::New { type_name, .. } => {
            assert!(type_name.contains("String"));
        }
        _ => panic!("Expected New event"),
    }
}
```

---

## Edge Cases

### Case 1: Generic Struct

```rust
struct Wrapper<T> {
    value: T,
}

#[track_ownership]
fn example<T>(w: Wrapper<T>) -> T {
    let x = w.value;
    x
}
```

**Solution:** Use `std::any::type_name::<T>()` for the inner type.

### Case 2: Associated Types

```rust
fn example<T: Iterator>(iter: T) -> Option<T::Item> {
    let x = iter.next();
    x
}
```

**Solution:** Use `std::any::type_name::<T::Item>()`.

### Case 3: Const Generics

```rust
fn example<const N: usize>(arr: [i32; N]) -> [i32; N] {
    let x = arr;
    x
}
```

**Solution:** Const generics don't affect type tracking.

---

## Limitations

### Can't Infer Generic Types

```rust
fn example<T>(value: T) {
    let x = value;  // Type is T, but we don't know what T is
}
```

**Solution:** Use `std::any::type_name::<T>()` at runtime.

### Type Names May Be Verbose

```rust
std::any::type_name::<Vec<String>>()
// Returns: "alloc::vec::Vec<alloc::string::String>"
```

**Solution:** Accept verbose names, or implement type name simplification.

---

## Type Name Simplification

```rust
fn simplify_type_name(full_name: &str) -> String {
    // Remove module paths
    full_name.split("::").last().unwrap_or(full_name).to_string()
}

// Usage in runtime
pub fn track_new<T>(id: usize, name: &str, type_name: &str, location: &str, value: T) -> T {
    let simplified = simplify_type_name(type_name);
    // Use simplified name
    value
}
```

---

## Key Takeaways

✅ **Preserve generics** - Don't modify type parameters  
✅ **Use type_name<T>()** - Get runtime type information  
✅ **Handle where clauses** - Keep trait bounds intact  
✅ **Test with multiple types** - Verify generic behavior  
✅ **Accept verbose names** - Or implement simplification  

---

## Further Reading

- [Generics](https://doc.rust-lang.org/book/ch10-00-generics.html)
- [std::any::type_name](https://doc.rust-lang.org/std/any/fn.type_name.html)
- [Where clauses](https://doc.rust-lang.org/reference/items/generics.html#where-clauses)
- [Const generics](https://doc.rust-lang.org/reference/items/generics.html#const-generics)

---

**Previous:** [48-optimizing-generated-code.md](./48-optimizing-generated-code.md)  
**Next:** [50-integration-testing-macro-runtime.md](./50-integration-testing-macro-runtime.md)

**Progress:** 14/15 ⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬜
