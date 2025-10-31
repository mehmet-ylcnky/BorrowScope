# Section 11: Understanding syn and quote

## Learning Objectives

By the end of this section, you will:
- Master syn's AST types and parsing
- Understand quote's code generation
- Parse complex Rust syntax patterns
- Generate code with proper hygiene
- Use interpolation effectively
- Handle spans for error messages
- Practice with real-world examples

## Prerequisites

- Completed Section 10
- Understanding of Rust syntax
- Familiarity with pattern matching

---

## The syn Crate

### What is syn?

**syn** is a parsing library for Rust syntax. It converts `TokenStream` into structured AST (Abstract Syntax Tree).

```rust
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn my_macro(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse TokenStream into ItemFn
    let function = parse_macro_input!(item as ItemFn);
    
    // Now we can work with structured data
    let fn_name = &function.sig.ident;
    
    quote! { #function }.into()
}
```

### Key AST Types

#### Item Types

```rust
use syn::{Item, ItemFn, ItemStruct, ItemEnum};

// Parse any item
let item: Item = syn::parse(tokens)?;

match item {
    Item::Fn(func) => { /* Function */ },
    Item::Struct(s) => { /* Struct */ },
    Item::Enum(e) => { /* Enum */ },
    _ => { /* Other items */ }
}
```

#### Expression Types

```rust
use syn::{Expr, ExprLit, ExprPath, ExprReference};

let expr: Expr = syn::parse(tokens)?;

match expr {
    Expr::Lit(lit) => { /* Literal: 5, "hello" */ },
    Expr::Path(path) => { /* Variable: x, foo::bar */ },
    Expr::Reference(r) => { /* Borrow: &x, &mut x */ },
    Expr::Call(call) => { /* Function call */ },
    _ => { /* Many more types */ }
}
```

#### Statement Types

```rust
use syn::{Stmt, Local};

let stmt: Stmt = syn::parse(tokens)?;

match stmt {
    Stmt::Local(local) => { /* let x = 5; */ },
    Stmt::Expr(expr, _) => { /* Expression statement */ },
    Stmt::Item(item) => { /* Item in function */ },
    _ => {}
}
```

---

## Parsing with syn

### Example 1: Parse a Function

```rust
use syn::{parse_quote, ItemFn};

// Parse from tokens
let function: ItemFn = parse_quote! {
    fn example(x: i32, y: i32) -> i32 {
        x + y
    }
};

// Access function components
let fn_name = &function.sig.ident;           // "example"
let inputs = &function.sig.inputs;           // Parameters
let output = &function.sig.output;           // Return type
let block = &function.block;                 // Function body

println!("Function name: {}", fn_name);
```

### Example 2: Parse a Let Statement

```rust
use syn::{parse_quote, Local};

let local: Local = parse_quote! {
    let x = 5;
};

// Extract pattern (variable name)
if let syn::Pat::Ident(pat_ident) = &local.pat {
    let var_name = &pat_ident.ident;
    println!("Variable: {}", var_name);  // "x"
}

// Extract initializer
if let Some(init) = &local.init {
    let expr = &init.expr;
    println!("Initializer: {}", quote! { #expr });  // "5"
}
```

### Example 3: Parse a Borrow Expression

```rust
use syn::{parse_quote, Expr, ExprReference};

let expr: Expr = parse_quote! { &x };

if let Expr::Reference(reference) = expr {
    let is_mutable = reference.mutability.is_some();
    let borrowed = &reference.expr;
    
    println!("Mutable: {}", is_mutable);      // false
    println!("Borrowed: {}", quote! { #borrowed });  // "x"
}
```

### Example 4: Parse Complex Patterns

```rust
use syn::{parse_quote, Pat};

// Simple identifier
let pat: Pat = parse_quote! { x };
assert!(matches!(pat, Pat::Ident(_)));

// Mutable identifier
let pat: Pat = parse_quote! { mut x };
assert!(matches!(pat, Pat::Ident(_)));

// Tuple pattern
let pat: Pat = parse_quote! { (x, y) };
assert!(matches!(pat, Pat::Tuple(_)));

// Struct pattern
let pat: Pat = parse_quote! { Point { x, y } };
assert!(matches!(pat, Pat::Struct(_)));
```

---

## The quote Crate

### What is quote?

**quote** generates Rust code from templates. It converts structured data back into `TokenStream`.

```rust
use quote::quote;

let name = "example";
let value = 42;

let code = quote! {
    fn #name() -> i32 {
        #value
    }
};

// Generates: fn example() -> i32 { 42 }
```

### Basic Interpolation

#### Simple Values

```rust
use quote::quote;
use syn::Ident;

let name = Ident::new("my_function", proc_macro2::Span::call_site());
let value = 42;

let code = quote! {
    fn #name() -> i32 {
        return #value;
    }
};
```

#### Expressions

```rust
use quote::quote;
use syn::{parse_quote, Expr};

let expr: Expr = parse_quote! { x + y };

let code = quote! {
    let result = #expr;
};

// Generates: let result = x + y;
```

#### Multiple Items

```rust
use quote::quote;

let items = vec!["a", "b", "c"];

let code = quote! {
    let values = vec![
        #(#items),*
    ];
};

// Generates: let values = vec!["a", "b", "c"];
```

---

## Interpolation Patterns

### Pattern 1: Repetition with Comma

```rust
use quote::quote;

let names = vec!["x", "y", "z"];

let code = quote! {
    (#(#names),*)
};

// Generates: (x, y, z)
```

### Pattern 2: Repetition with Semicolon

```rust
use quote::quote;

let statements = vec![
    quote! { let x = 1; },
    quote! { let y = 2; },
    quote! { let z = 3; },
];

let code = quote! {
    {
        #(#statements)*
    }
};

// Generates:
// {
//     let x = 1;
//     let y = 2;
//     let z = 3;
// }
```

### Pattern 3: Conditional Generation

```rust
use quote::quote;

let include_debug = true;

let code = quote! {
    fn example() {
        println!("Hello");
        
        #(
            if #include_debug {
                println!("Debug info");
            }
        )*
    }
};
```

### Pattern 4: Nested Interpolation

```rust
use quote::quote;

let fields = vec![("x", "i32"), ("y", "i32")];

let code = quote! {
    struct Point {
        #(
            #(#fields.0): #(#fields.1),
        )*
    }
};
```

---

## Practical Examples for BorrowScope

### Example 1: Transform Let Statement

```rust
use quote::quote;
use syn::{parse_quote, Local, Ident};

fn transform_let(local: &Local) -> proc_macro2::TokenStream {
    // Extract variable name
    let var_name = if let syn::Pat::Ident(pat_ident) = &local.pat {
        &pat_ident.ident
    } else {
        return quote! { #local };  // Return unchanged if complex pattern
    };
    
    // Extract initializer
    let init_expr = if let Some(init) = &local.init {
        &init.expr
    } else {
        return quote! { #local };  // Return unchanged if no initializer
    };
    
    // Generate tracking call
    quote! {
        let #var_name = borrowscope_runtime::track_new(
            stringify!(#var_name),
            #init_expr
        );
    }
}

// Test it
#[test]
fn test_transform_let() {
    let local: Local = parse_quote! { let s = String::from("hello"); };
    let transformed = transform_let(&local);
    
    let expected = quote! {
        let s = borrowscope_runtime::track_new(
            stringify!(s),
            String::from("hello")
        );
    };
    
    assert_eq!(
        transformed.to_string().replace(" ", ""),
        expected.to_string().replace(" ", "")
    );
}
```

### Example 2: Transform Borrow Expression

```rust
use quote::quote;
use syn::{parse_quote, Expr, ExprReference};

fn transform_borrow(
    var_name: &syn::Ident,
    expr: &Expr
) -> proc_macro2::TokenStream {
    if let Expr::Reference(reference) = expr {
        let is_mutable = reference.mutability.is_some();
        let borrowed_expr = &reference.expr;
        
        if is_mutable {
            quote! {
                borrowscope_runtime::track_borrow_mut(
                    stringify!(#var_name),
                    &mut #borrowed_expr
                )
            }
        } else {
            quote! {
                borrowscope_runtime::track_borrow(
                    stringify!(#var_name),
                    &#borrowed_expr
                )
            }
        }
    } else {
        quote! { #expr }
    }
}

// Test it
#[test]
fn test_transform_borrow() {
    let var_name: syn::Ident = parse_quote!(r);
    let expr: Expr = parse_quote!(&s);
    
    let transformed = transform_borrow(&var_name, &expr);
    
    let expected = quote! {
        borrowscope_runtime::track_borrow(
            stringify!(r),
            &s
        )
    };
    
    assert_eq!(
        transformed.to_string().replace(" ", ""),
        expected.to_string().replace(" ", "")
    );
}
```

### Example 3: Insert Drop Calls

```rust
use quote::quote;
use syn::{parse_quote, Block, Stmt};

fn insert_drop_calls(
    block: &Block,
    variables: &[syn::Ident]
) -> proc_macro2::TokenStream {
    let stmts = &block.stmts;
    
    // Generate drop calls
    let drop_calls = variables.iter().map(|var| {
        quote! {
            borrowscope_runtime::track_drop(stringify!(#var));
        }
    });
    
    quote! {
        {
            #(#stmts)*
            
            // Insert drop calls at end
            #(#drop_calls)*
        }
    }
}

// Test it
#[test]
fn test_insert_drop_calls() {
    let block: Block = parse_quote! {
        {
            let x = 5;
            println!("{}", x);
        }
    };
    
    let vars = vec![
        parse_quote!(x),
    ];
    
    let transformed = insert_drop_calls(&block, &vars);
    
    // Should contain drop call
    assert!(transformed.to_string().contains("track_drop"));
}
```

---

## Working with Spans

### What are Spans?

**Spans** track source code locations for error messages.

```rust
use proc_macro2::Span;
use syn::Ident;

// Create identifier with span
let ident = Ident::new("my_var", Span::call_site());

// Get span from existing token
let span = ident.span();

// Create error with span
let error = syn::Error::new(span, "Invalid identifier");
```

### Example: Error with Span

```rust
use syn::{parse_quote, ItemFn, Error};
use quote::quote;

fn validate_function(func: &ItemFn) -> Result<(), Error> {
    // Check if function has parameters
    if !func.sig.inputs.is_empty() {
        return Err(Error::new_spanned(
            &func.sig.inputs,
            "trace_borrow cannot be used on functions with parameters"
        ));
    }
    
    Ok(())
}

// Test it
#[test]
fn test_validate_function() {
    let func: ItemFn = parse_quote! {
        fn example(x: i32) {
            println!("{}", x);
        }
    };
    
    let result = validate_function(&func);
    assert!(result.is_err());
}
```

### Example: Preserve Spans

```rust
use quote::quote;
use syn::{parse_quote, Expr};

fn wrap_expression(expr: &Expr) -> proc_macro2::TokenStream {
    // Preserve the span of the original expression
    let span = expr.span();
    
    quote_spanned! { span =>
        borrowscope_runtime::track_expr(#expr)
    }
}
```

---

## Advanced Patterns

### Pattern 1: Recursive Transformation

```rust
use syn::visit_mut::{self, VisitMut};
use syn::{Expr, ExprReference};
use quote::quote;

struct BorrowTransformer;

impl VisitMut for BorrowTransformer {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        // Transform borrows
        if let Expr::Reference(reference) = expr {
            let borrowed = &reference.expr;
            let is_mut = reference.mutability.is_some();
            
            *expr = if is_mut {
                syn::parse_quote! {
                    borrowscope_runtime::track_borrow_mut("temp", &mut #borrowed)
                }
            } else {
                syn::parse_quote! {
                    borrowscope_runtime::track_borrow("temp", &#borrowed)
                }
            };
        }
        
        // Continue visiting children
        visit_mut::visit_expr_mut(self, expr);
    }
}

// Test it
#[test]
fn test_recursive_transformation() {
    let mut expr: Expr = parse_quote! { &x };
    
    let mut transformer = BorrowTransformer;
    transformer.visit_expr_mut(&mut expr);
    
    assert!(quote! { #expr }.to_string().contains("track_borrow"));
}
```

### Pattern 2: Collecting Information

```rust
use syn::visit::{self, Visit};
use syn::{Expr, Local, ItemFn};
use std::collections::HashSet;

struct VariableCollector {
    variables: HashSet<String>,
}

impl VariableCollector {
    fn new() -> Self {
        Self {
            variables: HashSet::new(),
        }
    }
}

impl<'ast> Visit<'ast> for VariableCollector {
    fn visit_local(&mut self, local: &'ast Local) {
        // Extract variable name
        if let syn::Pat::Ident(pat_ident) = &local.pat {
            self.variables.insert(pat_ident.ident.to_string());
        }
        
        // Continue visiting
        visit::visit_local(self, local);
    }
}

// Test it
#[test]
fn test_variable_collector() {
    let func: ItemFn = parse_quote! {
        fn example() {
            let x = 5;
            let y = 10;
            let z = x + y;
        }
    };
    
    let mut collector = VariableCollector::new();
    collector.visit_item_fn(&func);
    
    assert_eq!(collector.variables.len(), 3);
    assert!(collector.variables.contains("x"));
    assert!(collector.variables.contains("y"));
    assert!(collector.variables.contains("z"));
}
```

### Pattern 3: Conditional Transformation

```rust
use syn::{Expr, Local};
use quote::quote;

fn should_track(local: &Local) -> bool {
    // Don't track if variable starts with underscore
    if let syn::Pat::Ident(pat_ident) = &local.pat {
        !pat_ident.ident.to_string().starts_with('_')
    } else {
        false
    }
}

fn maybe_transform_let(local: &Local) -> proc_macro2::TokenStream {
    if should_track(local) {
        // Transform
        if let syn::Pat::Ident(pat_ident) = &local.pat {
            let var_name = &pat_ident.ident;
            if let Some(init) = &local.init {
                let init_expr = &init.expr;
                return quote! {
                    let #var_name = borrowscope_runtime::track_new(
                        stringify!(#var_name),
                        #init_expr
                    );
                };
            }
        }
    }
    
    // Return unchanged
    quote! { #local }
}

// Test it
#[test]
fn test_conditional_transformation() {
    let local1: Local = parse_quote! { let x = 5; };
    let local2: Local = parse_quote! { let _unused = 5; };
    
    let result1 = maybe_transform_let(&local1);
    let result2 = maybe_transform_let(&local2);
    
    assert!(result1.to_string().contains("track_new"));
    assert!(!result2.to_string().contains("track_new"));
}
```

---

## Complete Example: Function Transformer

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, visit_mut::VisitMut};

struct FunctionTransformer {
    variables: Vec<syn::Ident>,
}

impl FunctionTransformer {
    fn new() -> Self {
        Self {
            variables: Vec::new(),
        }
    }
}

impl VisitMut for FunctionTransformer {
    fn visit_local_mut(&mut self, local: &mut syn::Local) {
        // Extract variable name
        if let syn::Pat::Ident(pat_ident) = &local.pat {
            let var_name = pat_ident.ident.clone();
            self.variables.push(var_name.clone());
            
            // Transform if has initializer
            if let Some(init) = &local.init {
                let init_expr = &init.expr;
                
                // Replace with tracking call
                *local = syn::parse_quote! {
                    let #var_name = borrowscope_runtime::track_new(
                        stringify!(#var_name),
                        #init_expr
                    );
                };
            }
        }
        
        // Continue visiting
        syn::visit_mut::visit_local_mut(self, local);
    }
}

#[proc_macro_attribute]
pub fn track_variables(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut function = parse_macro_input!(item as ItemFn);
    
    // Transform the function
    let mut transformer = FunctionTransformer::new();
    transformer.visit_item_fn_mut(&mut function);
    
    // Add drop calls at end
    let variables = &transformer.variables;
    let original_block = &function.block;
    
    function.block = syn::parse_quote! {
        {
            #original_block
            
            // Add drop tracking
            #(
                borrowscope_runtime::track_drop(stringify!(#variables));
            )*
        }
    };
    
    quote! { #function }.into()
}

// Test it
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_track_variables() {
        let input = quote! {
            fn example() {
                let x = 5;
                let y = 10;
                println!("{} {}", x, y);
            }
        };
        
        let output = track_variables(Default::default(), input.into());
        let output_str = output.to_string();
        
        assert!(output_str.contains("track_new"));
        assert!(output_str.contains("track_drop"));
    }
}
```

---

## Debugging Tips

### Tip 1: Print Generated Code

```rust
use quote::quote;

let code = quote! {
    fn example() {
        let x = 5;
    }
};

// Pretty print
let syntax_tree = syn::parse_file(&code.to_string()).unwrap();
let formatted = prettyplease::unparse(&syntax_tree);
println!("{}", formatted);
```

### Tip 2: Use cargo-expand

```bash
cargo expand --lib
```

Shows the expanded macro output.

### Tip 3: Debug with eprintln!

```rust
#[proc_macro_attribute]
pub fn my_macro(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let function = parse_macro_input!(item as ItemFn);
    
    eprintln!("Function name: {}", function.sig.ident);
    eprintln!("Parameters: {}", function.sig.inputs.len());
    
    quote! { #function }.into()
}
```

### Tip 4: Test Incrementally

```rust
#[test]
fn test_parse() {
    let input = quote! { let x = 5; };
    let local: Local = syn::parse2(input).unwrap();
    // Test parsing works
}

#[test]
fn test_transform() {
    let local: Local = parse_quote! { let x = 5; };
    let transformed = transform_let(&local);
    // Test transformation works
}

#[test]
fn test_generate() {
    let code = quote! { fn test() {} };
    let _: ItemFn = syn::parse2(code).unwrap();
    // Test generation works
}
```

---

## Common Pitfalls

### Pitfall 1: Forgetting to Visit Children

```rust
impl VisitMut for MyVisitor {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        // Transform expr
        *expr = transform(expr);
        
        // ❌ Forgot to visit children!
        // Children won't be transformed
    }
}

// Fix:
impl VisitMut for MyVisitor {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        // Transform expr
        *expr = transform(expr);
        
        // ✅ Visit children
        visit_mut::visit_expr_mut(self, expr);
    }
}
```

### Pitfall 2: Incorrect Interpolation

```rust
// ❌ Wrong: Interpolates as string
let name = "example";
let code = quote! {
    fn #name() {}
};
// Generates: fn "example"() {}

// ✅ Correct: Use Ident
let name = Ident::new("example", Span::call_site());
let code = quote! {
    fn #name() {}
};
// Generates: fn example() {}
```

### Pitfall 3: Losing Spans

```rust
// ❌ Creates new span
let new_expr: Expr = syn::parse_quote! { x + 1 };

// ✅ Preserves span
let new_expr = {
    let span = original_expr.span();
    syn::parse_quote_spanned! { span => x + 1 }
};
```

---

## Key Takeaways

### syn Fundamentals

✅ **Parses TokenStream to AST** - Structured data  
✅ **Many AST types** - Item, Expr, Stmt, Pat, etc.  
✅ **Pattern matching** - Match on AST variants  
✅ **Visitor pattern** - Traverse and transform  
✅ **Spans** - Track source locations  

### quote Fundamentals

✅ **Generates code** - AST to TokenStream  
✅ **Interpolation** - `#var` syntax  
✅ **Repetition** - `#()*` patterns  
✅ **Hygiene** - Automatic span handling  
✅ **Type-safe** - Compile-time checking  

### Best Practices

✅ **Parse early** - Validate input immediately  
✅ **Transform carefully** - Preserve spans  
✅ **Generate cleanly** - Use quote! macro  
✅ **Test thoroughly** - Unit test each part  
✅ **Debug incrementally** - Test parse, transform, generate separately  

---

## Exercises

### Exercise 1: Parse and Print

Parse this code and print all variable names:
```rust
fn example() {
    let x = 5;
    let y = 10;
    let z = x + y;
}
```

### Exercise 2: Transform Expressions

Transform all additions to use a custom `add` function:
```rust
// Input: x + y
// Output: add(x, y)
```

### Exercise 3: Generate Code

Generate a struct with fields from a vector:
```rust
let fields = vec![("x", "i32"), ("y", "i32")];
// Generate: struct Point { x: i32, y: i32 }
```

---

## What's Next?

In **Section 12: Parsing Function Attributes**, we'll:
- Parse the `#[trace_borrow]` attribute
- Handle attribute arguments
- Validate function signatures
- Extract function metadata
- Prepare for transformation

---

**Previous Section:** [10-creating-the-macro-crate.md](./10-creating-the-macro-crate.md)  
**Next Section:** [12-parsing-function-attributes.md](./12-parsing-function-attributes.md)

**Chapter Progress:** 3/12 sections complete ⬛⬛⬛⬜⬜⬜⬜⬜⬜⬜⬜⬜

---

*"Understanding syn and quote is the key to mastering procedural macros." 🔑*
