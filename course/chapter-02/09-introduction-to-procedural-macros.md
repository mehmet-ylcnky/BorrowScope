# Section 9: Introduction to Procedural Macros

## Learning Objectives

By the end of this section, you will:
- Understand what procedural macros are and how they work
- Know the three types of procedural macros
- Understand when to use procedural macros vs declarative macros
- Grasp the compilation process for proc macros
- Write your first simple procedural macro
- Understand TokenStream and how macros transform code

## Prerequisites

- Completed Chapter 1
- Understanding of Rust syntax
- Basic knowledge of declarative macros (macro_rules!)
- Familiarity with the compilation process

---

## What Are Procedural Macros?

### Definition

**Procedural macros** are Rust functions that take code as input and produce code as output. They run at **compile time** and can:
- Parse Rust syntax
- Analyze code structure
- Generate new code
- Transform existing code

### Declarative vs Procedural Macros

**Declarative Macros (macro_rules!):**
```rust
macro_rules! vec {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x);
            )*
            temp_vec
        }
    };
}
```

**Procedural Macros:**
```rust
#[proc_macro_attribute]
pub fn trace_borrow(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse, analyze, transform
    item
}
```

**Key Differences:**

| Feature | Declarative | Procedural |
|---------|-------------|------------|
| Syntax | Pattern matching | Rust functions |
| Power | Limited | Full Rust power |
| Complexity | Simple patterns | Complex transformations |
| Parsing | Built-in | Manual (syn crate) |
| Use case | Simple code generation | Complex AST manipulation |

---

## Three Types of Procedural Macros

### 1. Function-like Macros

**Syntax:** Look like function calls
```rust
sql!(SELECT * FROM users WHERE id = 1)
```

**Definition:**
```rust
#[proc_macro]
pub fn sql(input: TokenStream) -> TokenStream {
    // Parse SQL, generate Rust code
    input
}
```

**Use cases:**
- Domain-specific languages (DSL)
- Custom syntax
- Code generation from strings

### 2. Derive Macros

**Syntax:** Used with `#[derive(...)]`
```rust
#[derive(Debug, Clone, MyCustomDerive)]
struct Point {
    x: i32,
    y: i32,
}
```

**Definition:**
```rust
#[proc_macro_derive(MyCustomDerive)]
pub fn my_custom_derive(input: TokenStream) -> TokenStream {
    // Generate trait implementation
    input
}
```

**Use cases:**
- Automatic trait implementations
- Boilerplate reduction
- Code generation from struct/enum definitions

### 3. Attribute Macros

**Syntax:** Used as attributes
```rust
#[trace_borrow]
fn example() {
    let s = String::from("hello");
}
```

**Definition:**
```rust
#[proc_macro_attribute]
pub fn trace_borrow(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Transform the function
    item
}
```

**Use cases:**
- Function transformation
- Code instrumentation
- Adding behavior to items

**For BorrowScope:** We'll use **attribute macros** to transform functions.

---

## How Procedural Macros Work

### The Compilation Pipeline

```
Source Code
    ↓
[Lexer] → Tokens
    ↓
[Parser] → AST
    ↓
[Macro Expansion] ← Procedural Macros Run Here
    ↓
[Name Resolution]
    ↓
[Type Checking]
    ↓
[Borrow Checking]
    ↓
[MIR Generation]
    ↓
[LLVM IR]
    ↓
Machine Code
```

**Procedural macros run during macro expansion:**
1. Compiler encounters `#[trace_borrow]`
2. Calls your macro function
3. Passes code as `TokenStream`
4. Your macro returns transformed `TokenStream`
5. Compiler continues with transformed code

### TokenStream

**What is TokenStream?**

A stream of tokens representing Rust code:

```rust
// This code:
let x = 5;

// Becomes these tokens:
[
    Ident("let"),
    Ident("x"),
    Punct('='),
    Literal(5),
    Punct(';'),
]
```

**TokenStream in Rust:**
```rust
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn my_macro(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // item is the code being annotated
    // Return transformed code
    item
}
```

---

## Your First Procedural Macro

### Step 1: Create a Simple Macro

Let's create a macro that does nothing (identity macro):

**File: `borrowscope-macro/src/lib.rs`**

```rust
use proc_macro::TokenStream;

/// A simple attribute macro that returns input unchanged
///
/// # Example
///
/// ```ignore
/// #[identity]
/// fn hello() {
///     println!("Hello, world!");
/// }
/// ```
#[proc_macro_attribute]
pub fn identity(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Simply return the input unchanged
    item
}
```

### Step 2: Test the Macro

**File: `borrowscope-macro/tests/identity_test.rs`**

```rust
use borrowscope_macro::identity;

#[identity]
fn test_function() {
    println!("This function is unchanged");
}

#[test]
fn test_identity_macro() {
    test_function();
    // If this compiles and runs, the macro works
}
```

### Step 3: Build and Test

```bash
cargo build -p borrowscope-macro
cargo test -p borrowscope-macro
```

**Expected output:**
```
   Compiling borrowscope-macro v0.1.0
    Finished test [unoptimized + debuginfo] target(s) in 2.34s
     Running tests/identity_test.rs

running 1 test
test test_identity_macro ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

---

## Understanding TokenStream in Detail

### Inspecting TokenStream

Let's create a macro that prints what it receives:

```rust
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn debug_tokens(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Print the tokens (only works during compilation)
    eprintln!("Received tokens: {}", item);
    
    // Return unchanged
    item
}
```

**Usage:**
```rust
#[debug_tokens]
fn example() {
    let x = 5;
}
```

**Compile output:**
```
Received tokens: fn example() { let x = 5; }
```

### TokenStream Structure

```rust
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn analyze_tokens(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Convert to string to see structure
    let code = item.to_string();
    eprintln!("Code: {}", code);
    
    // Iterate over tokens
    for token in item.clone() {
        eprintln!("Token: {:?}", token);
    }
    
    item
}
```

---

## Macro Hygiene

### What is Hygiene?

**Hygiene** ensures macros don't accidentally capture or conflict with variables in the calling scope.

**Problem without hygiene:**
```rust
// Macro generates:
let x = 5;

// User code:
let x = 10;
#[my_macro]
fn test() {
    println!("{}", x);  // Which x? 5 or 10?
}
```

**Rust's solution:**
- Procedural macros use **spans** to track where code came from
- Variables from macro don't conflict with user variables
- Each has its own "hygiene context"

### Spans

```rust
use proc_macro::{TokenStream, Span};

#[proc_macro_attribute]
pub fn with_span(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Each token has a span (source location)
    for token in item.clone() {
        let span = token.span();
        eprintln!("Token at: {:?}", span);
    }
    
    item
}
```

**Spans preserve:**
- File name
- Line number
- Column number
- Hygiene context

---

## When to Use Procedural Macros

### Good Use Cases

✅ **Code generation from attributes**
```rust
#[derive(Serialize, Deserialize)]
struct User { ... }
```

✅ **DSLs (Domain-Specific Languages)**
```rust
html! {
    <div class="container">
        <h1>"Hello"</h1>
    </div>
}
```

✅ **Code instrumentation**
```rust
#[trace_borrow]  // ← BorrowScope!
fn example() { ... }
```

✅ **Boilerplate reduction**
```rust
#[async_trait]
trait MyTrait { ... }
```

### Bad Use Cases

❌ **Simple patterns** - Use declarative macros
```rust
// Use macro_rules! instead
macro_rules! vec { ... }
```

❌ **Runtime logic** - Use regular functions
```rust
// Don't use macros for this
fn add(a: i32, b: i32) -> i32 { a + b }
```

❌ **Type-level programming** - Use traits and generics
```rust
// Use traits instead
trait Add<T> { ... }
```

---

## Procedural Macro Limitations

### What You Can't Do

❌ **Access type information**
```rust
// Can't know if T implements Clone
#[proc_macro_attribute]
pub fn needs_clone<T>(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // No access to type system here!
    item
}
```

❌ **Perform borrow checking**
```rust
// Can't validate borrows at macro expansion time
// Borrow checker runs later
```

❌ **Execute arbitrary code**
```rust
// Can't read files, make network requests, etc.
// (Well, you can, but you shouldn't!)
```

### What You Can Do

✅ **Parse syntax**
✅ **Generate code**
✅ **Transform AST**
✅ **Validate syntax patterns**
✅ **Emit compile errors**

---

## Error Handling in Macros

### Emitting Compile Errors

```rust
use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn must_be_function(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the item
    let input = syn::parse_macro_input!(item as syn::Item);
    
    // Check if it's a function
    match input {
        syn::Item::Fn(_) => {
            // It's a function, proceed
            quote! { #input }.into()
        }
        _ => {
            // Not a function, emit error
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "This attribute can only be applied to functions"
            )
            .to_compile_error()
            .into()
        }
    }
}
```

**Usage:**
```rust
#[must_be_function]
struct NotAFunction;  // ❌ Compile error!

#[must_be_function]
fn this_works() { }   // ✅ OK
```

---

## The syn and quote Crates

### Why We Need Them

**Problem:** `TokenStream` is just a stream of tokens, hard to work with.

**Solution:** 
- **syn** - Parse `TokenStream` into structured AST
- **quote** - Generate `TokenStream` from Rust code

### Basic Example

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn add_println(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse as a function
    let input = parse_macro_input!(item as ItemFn);
    
    // Get function name
    let fn_name = &input.sig.ident;
    
    // Generate new code
    let output = quote! {
        #input
        
        // Add a helper function
        fn print_name() {
            println!("Function name: {}", stringify!(#fn_name));
        }
    };
    
    output.into()
}
```

**We'll dive deep into syn and quote in the next sections.**

---

## BorrowScope's Macro Strategy

### What We Need to Do

1. **Parse** the function with `#[trace_borrow]`
2. **Identify** variable declarations, borrows, moves
3. **Inject** tracking calls
4. **Preserve** original behavior

### High-Level Approach

```rust
#[proc_macro_attribute]
pub fn trace_borrow(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // 1. Parse the function
    let mut function = parse_macro_input!(item as ItemFn);
    
    // 2. Visit and transform the function body
    let mut visitor = BorrowVisitor::new();
    visitor.visit_item_fn_mut(&mut function);
    
    // 3. Generate transformed code
    quote! { #function }.into()
}
```

**We'll implement this step-by-step in upcoming sections.**

---

## Practical Example: Adding Logging

Let's create a macro that adds logging to functions:

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// Add entry/exit logging to a function
#[proc_macro_attribute]
pub fn log_calls(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    
    let fn_name = &input.sig.ident;
    let fn_block = &input.block;
    let fn_sig = &input.sig;
    let fn_vis = &input.vis;
    let fn_attrs = &input.attrs;
    
    let output = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_sig {
            println!("Entering: {}", stringify!(#fn_name));
            
            let result = (|| #fn_block)();
            
            println!("Exiting: {}", stringify!(#fn_name));
            
            result
        }
    };
    
    output.into()
}
```

**Usage:**
```rust
#[log_calls]
fn greet(name: &str) {
    println!("Hello, {}!", name);
}

fn main() {
    greet("Alice");
}
```

**Output:**
```
Entering: greet
Hello, Alice!
Exiting: greet
```

---

## Testing Procedural Macros

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    
    #[test]
    fn test_identity_macro() {
        let input = quote! {
            fn test() {
                let x = 5;
            }
        };
        
        let output = identity(Default::default(), input.into());
        
        // Output should be valid Rust code
        assert!(!output.is_empty());
    }
}
```

### Integration Tests with trybuild

**File: `tests/compile_pass.rs`**

```rust
#[test]
fn test_compile_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/pass/*.rs");
}
```

**File: `tests/ui/pass/simple.rs`**

```rust
use borrowscope_macro::trace_borrow;

#[trace_borrow]
fn example() {
    let x = 5;
}

fn main() {
    example();
}
```

---

## Common Pitfalls

### Pitfall 1: Forgetting to Return TokenStream

```rust
#[proc_macro_attribute]
pub fn my_macro(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // ❌ Forgot to return!
    let parsed = syn::parse_macro_input!(item as ItemFn);
    // Function ends without returning
}
```

**Fix:**
```rust
#[proc_macro_attribute]
pub fn my_macro(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let parsed = syn::parse_macro_input!(item as ItemFn);
    quote! { #parsed }.into()  // ✅ Return TokenStream
}
```

### Pitfall 2: Not Handling Errors

```rust
#[proc_macro_attribute]
pub fn my_macro(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let parsed = syn::parse::<ItemFn>(item).unwrap();  // ❌ Panics on error!
    quote! { #parsed }.into()
}
```

**Fix:**
```rust
#[proc_macro_attribute]
pub fn my_macro(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let parsed = match syn::parse::<ItemFn>(item) {
        Ok(p) => p,
        Err(e) => return e.to_compile_error().into(),  // ✅ Proper error
    };
    quote! { #parsed }.into()
}
```

### Pitfall 3: Modifying Input Incorrectly

```rust
#[proc_macro_attribute]
pub fn my_macro(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // ❌ Loses original attributes, visibility, etc.
    quote! {
        fn new_function() { }
    }.into()
}
```

**Fix:**
```rust
#[proc_macro_attribute]
pub fn my_macro(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    // ✅ Preserve everything
    quote! { #input }.into()
}
```

---

## Key Takeaways

### Procedural Macros Fundamentals

✅ **Run at compile time** - Transform code before compilation  
✅ **Three types** - Function-like, derive, attribute  
✅ **Work with TokenStream** - Stream of tokens  
✅ **Use syn to parse** - Convert tokens to AST  
✅ **Use quote to generate** - Convert AST to tokens  

### When to Use

✅ **Code generation** - Reduce boilerplate  
✅ **DSLs** - Custom syntax  
✅ **Instrumentation** - Add behavior to code  
✅ **Derive traits** - Automatic implementations  

### Limitations

❌ **No type information** - Runs before type checking  
❌ **No borrow checking** - Runs before borrow checker  
❌ **Syntax only** - Can only work with syntax  

### BorrowScope Application

✅ **Attribute macro** - `#[trace_borrow]`  
✅ **Transform functions** - Inject tracking calls  
✅ **Preserve semantics** - Code behaves the same  
✅ **Parse with syn** - Understand code structure  
✅ **Generate with quote** - Create new code  

---

## Exercises

### Exercise 1: Identity Macro

Implement and test the identity macro from this section.

### Exercise 2: Debug Macro

Create a macro that prints the function name when called:

```rust
#[debug_name]
fn my_function() {
    // Should print: "Calling: my_function"
}
```

### Exercise 3: Count Lines

Create a macro that counts lines in a function:

```rust
#[count_lines]
fn example() {
    let x = 5;
    let y = 10;
    println!("{}", x + y);
}
// Should print: "Function has 3 lines"
```

---

## Further Reading

### Official Documentation

1. **The Rust Reference - Procedural Macros**
   - https://doc.rust-lang.org/reference/procedural-macros.html

2. **The Little Book of Rust Macros**
   - https://danielkeep.github.io/tlborm/book/

3. **Proc Macro Workshop**
   - https://github.com/dtolnay/proc-macro-workshop

### Crates

1. **syn** - https://docs.rs/syn
2. **quote** - https://docs.rs/quote
3. **proc-macro2** - https://docs.rs/proc-macro2

---

## What's Next?

In **Section 10: Creating the Macro Crate**, we'll:
- Set up the borrowscope-macro crate properly
- Configure dependencies (syn, quote)
- Understand proc-macro crate structure
- Set up testing infrastructure
- Create the foundation for our `#[trace_borrow]` macro

---

**Previous Section:** [Chapter 1 Complete](../chapter-01/08-development-environment-optimization.md)  
**Next Section:** [10-creating-the-macro-crate.md](./10-creating-the-macro-crate.md)

**Chapter Progress:** 1/12 sections complete ⬛⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜

---

*"Macros are code that writes code. Master them, and you master metaprogramming." 🎭*
