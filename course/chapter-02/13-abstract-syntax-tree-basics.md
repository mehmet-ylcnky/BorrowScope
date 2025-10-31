# Section 13: Abstract Syntax Tree Basics

## Learning Objectives

By the end of this section, you will:
- Understand AST structure and hierarchy
- Navigate complex AST nodes
- Pattern match on different node types
- Traverse AST trees effectively
- Identify ownership patterns in AST
- Prepare for AST transformation

## Prerequisites

- Completed Section 12
- Understanding of syn and quote
- Familiarity with Rust syntax

---

## What is an Abstract Syntax Tree?

### Definition

An **Abstract Syntax Tree (AST)** is a tree representation of source code structure.

**Example:**
```rust
let x = 5 + 3;
```

**AST:**
```
Local
├── Pat: Ident("x")
└── Init
    └── Expr: Binary
        ├── left: Lit(5)
        ├── op: Add
        └── right: Lit(3)
```

### Why AST?

**Without AST (tokens):**
```
[Ident("let"), Ident("x"), Punct('='), Lit(5), Punct('+'), Lit(3), Punct(';')]
```
Hard to understand structure!

**With AST:**
```rust
Local {
    pat: Pat::Ident("x"),
    init: Some(Expr::Binary {
        left: Expr::Lit(5),
        op: BinOp::Add,
        right: Expr::Lit(3),
    })
}
```
Clear structure!

---

## AST Node Hierarchy

### Top-Level: Items

```rust
use syn::Item;

pub enum Item {
    Fn(ItemFn),           // Function
    Struct(ItemStruct),   // Struct
    Enum(ItemEnum),       // Enum
    Mod(ItemMod),         // Module
    Use(ItemUse),         // Use statement
    // ... many more
}
```

### Function: ItemFn

```rust
use syn::ItemFn;

pub struct ItemFn {
    pub attrs: Vec<Attribute>,      // #[attributes]
    pub vis: Visibility,            // pub, pub(crate), etc.
    pub sig: Signature,             // Function signature
    pub block: Box<Block>,          // Function body
}
```

### Signature

```rust
use syn::Signature;

pub struct Signature {
    pub ident: Ident,               // Function name
    pub inputs: Punctuated<FnArg>,  // Parameters
    pub output: ReturnType,         // Return type
    pub generics: Generics,         // <T, U>
    // ... more fields
}
```

### Block (Function Body)

```rust
use syn::Block;

pub struct Block {
    pub stmts: Vec<Stmt>,           // Statements in the block
}
```

---

## Statement Types

### Stmt Enum

```rust
use syn::Stmt;

pub enum Stmt {
    Local(Local),           // let x = 5;
    Item(Item),             // fn inner() {}
    Expr(Expr, Option<Token![;]>),  // expression;
    Macro(StmtMacro),       // println!(...);
}
```

### Local (Let Statement)

```rust
use syn::Local;

pub struct Local {
    pub pat: Pat,                   // Variable pattern
    pub init: Option<LocalInit>,    // Initializer
    // ... more fields
}

pub struct LocalInit {
    pub expr: Box<Expr>,            // Initialization expression
    pub diverge: Option<...>,       // else branch
}
```

**Example:**
```rust
let x = 5;

// AST:
Local {
    pat: Pat::Ident(PatIdent {
        ident: Ident("x"),
        ...
    }),
    init: Some(LocalInit {
        expr: Expr::Lit(ExprLit {
            lit: Lit::Int(5),
        }),
    }),
}
```

---

## Expression Types

### Expr Enum (Simplified)

```rust
use syn::Expr;

pub enum Expr {
    Lit(ExprLit),               // 5, "hello", true
    Path(ExprPath),             // x, foo::bar
    Reference(ExprReference),   // &x, &mut x
    Call(ExprCall),             // foo(x, y)
    MethodCall(ExprMethodCall), // x.foo()
    Binary(ExprBinary),         // x + y
    Unary(ExprUnary),           // !x, -x
    Block(ExprBlock),           // { ... }
    If(ExprIf),                 // if x { }
    Match(ExprMatch),           // match x { }
    // ... 40+ more variants
}
```

### ExprReference (Borrows)

```rust
use syn::ExprReference;

pub struct ExprReference {
    pub mutability: Option<Token![mut]>,  // &mut vs &
    pub expr: Box<Expr>,                  // Borrowed expression
}
```

**Examples:**
```rust
// Immutable borrow
&x
// AST:
ExprReference {
    mutability: None,
    expr: Expr::Path(x),
}

// Mutable borrow
&mut x
// AST:
ExprReference {
    mutability: Some(mut),
    expr: Expr::Path(x),
}
```

### ExprCall (Function Calls)

```rust
use syn::ExprCall;

pub struct ExprCall {
    pub func: Box<Expr>,                // Function being called
    pub args: Punctuated<Expr>,         // Arguments
}
```

**Example:**
```rust
String::from("hello")

// AST:
ExprCall {
    func: Expr::Path(String::from),
    args: [Expr::Lit("hello")],
}
```

---

## Pattern Types

### Pat Enum

```rust
use syn::Pat;

pub enum Pat {
    Ident(PatIdent),        // x, mut x
    Tuple(PatTuple),        // (x, y)
    Struct(PatStruct),      // Point { x, y }
    TupleStruct(PatTupleStruct),  // Some(x)
    Wild(PatWild),          // _
    // ... more variants
}
```

### PatIdent (Simple Variable)

```rust
use syn::PatIdent;

pub struct PatIdent {
    pub ident: Ident,               // Variable name
    pub mutability: Option<Token![mut]>,  // mut keyword
    pub subpat: Option<...>,        // @ pattern
}
```

**Examples:**
```rust
// Simple
let x = 5;
// Pat::Ident { ident: "x", mutability: None }

// Mutable
let mut x = 5;
// Pat::Ident { ident: "x", mutability: Some(mut) }
```

---

## Practical Examples

### Example 1: Parse and Inspect Function

```rust
use syn::{parse_quote, ItemFn};

fn inspect_function() {
    let func: ItemFn = parse_quote! {
        pub fn example(x: i32) -> i32 {
            let y = x + 1;
            y
        }
    };
    
    // Function name
    println!("Name: {}", func.sig.ident);
    
    // Visibility
    println!("Public: {}", matches!(func.vis, syn::Visibility::Public(_)));
    
    // Parameters
    println!("Params: {}", func.sig.inputs.len());
    
    // Return type
    println!("Returns: {}", !matches!(func.sig.output, syn::ReturnType::Default));
    
    // Statements
    println!("Statements: {}", func.block.stmts.len());
}

#[test]
fn test_inspect_function() {
    inspect_function();
}
```

### Example 2: Find All Variables

```rust
use syn::{parse_quote, ItemFn, Stmt, Pat};

fn find_variables(func: &ItemFn) -> Vec<String> {
    let mut variables = Vec::new();
    
    for stmt in &func.block.stmts {
        if let Stmt::Local(local) = stmt {
            if let Pat::Ident(pat_ident) = &local.pat {
                variables.push(pat_ident.ident.to_string());
            }
        }
    }
    
    variables
}

#[test]
fn test_find_variables() {
    let func: ItemFn = parse_quote! {
        fn example() {
            let x = 5;
            let y = 10;
            let z = x + y;
        }
    };
    
    let vars = find_variables(&func);
    assert_eq!(vars, vec!["x", "y", "z"]);
}
```

### Example 3: Find All Borrows

```rust
use syn::{parse_quote, ItemFn, Stmt, Expr, Local};

fn find_borrows(func: &ItemFn) -> Vec<(String, bool)> {
    let mut borrows = Vec::new();
    
    for stmt in &func.block.stmts {
        if let Stmt::Local(local) = stmt {
            if let Some(init) = &local.init {
                if let Expr::Reference(reference) = init.expr.as_ref() {
                    if let Pat::Ident(pat_ident) = &local.pat {
                        let var_name = pat_ident.ident.to_string();
                        let is_mutable = reference.mutability.is_some();
                        borrows.push((var_name, is_mutable));
                    }
                }
            }
        }
    }
    
    borrows
}

#[test]
fn test_find_borrows() {
    let func: ItemFn = parse_quote! {
        fn example() {
            let s = String::from("hello");
            let r1 = &s;
            let r2 = &mut s;
        }
    };
    
    let borrows = find_borrows(&func);
    assert_eq!(borrows.len(), 2);
    assert_eq!(borrows[0], ("r1".to_string(), false));
    assert_eq!(borrows[1], ("r2".to_string(), true));
}
```

### Example 4: Identify Moves

```rust
use syn::{parse_quote, ItemFn, Stmt, Expr, Local};

fn find_potential_moves(func: &ItemFn) -> Vec<String> {
    let mut moves = Vec::new();
    
    for stmt in &func.block.stmts {
        if let Stmt::Local(local) = stmt {
            if let Some(init) = &local.init {
                // Check if initializer is a simple path (potential move)
                if let Expr::Path(path) = init.expr.as_ref() {
                    if let Pat::Ident(pat_ident) = &local.pat {
                        moves.push(pat_ident.ident.to_string());
                    }
                }
            }
        }
    }
    
    moves
}

#[test]
fn test_find_potential_moves() {
    let func: ItemFn = parse_quote! {
        fn example() {
            let s = String::from("hello");
            let t = s;  // Potential move
            let r = &t; // Not a move (borrow)
        }
    };
    
    let moves = find_potential_moves(&func);
    assert!(moves.contains(&"t".to_string()));
}
```

---

## AST Traversal Patterns

### Pattern 1: Recursive Descent

```rust
use syn::{Expr, ExprBinary, ExprLit};

fn count_literals(expr: &Expr) -> usize {
    match expr {
        Expr::Lit(_) => 1,
        Expr::Binary(ExprBinary { left, right, .. }) => {
            count_literals(left) + count_literals(right)
        }
        Expr::Paren(paren) => count_literals(&paren.expr),
        _ => 0,
    }
}

#[test]
fn test_count_literals() {
    let expr: Expr = parse_quote! { 1 + 2 + 3 };
    assert_eq!(count_literals(&expr), 3);
}
```

### Pattern 2: Visitor Pattern

```rust
use syn::visit::Visit;
use syn::{Expr, ItemFn};

struct LiteralCounter {
    count: usize,
}

impl<'ast> Visit<'ast> for LiteralCounter {
    fn visit_expr(&mut self, expr: &'ast Expr) {
        if matches!(expr, Expr::Lit(_)) {
            self.count += 1;
        }
        
        // Continue visiting children
        syn::visit::visit_expr(self, expr);
    }
}

#[test]
fn test_visitor_pattern() {
    let func: ItemFn = parse_quote! {
        fn example() {
            let x = 5;
            let y = 10;
            let z = x + 15;
        }
    };
    
    let mut counter = LiteralCounter { count: 0 };
    counter.visit_item_fn(&func);
    
    assert_eq!(counter.count, 3);
}
```

### Pattern 3: Mutable Visitor

```rust
use syn::visit_mut::{self, VisitMut};
use syn::{Expr, ExprLit, Lit, LitInt};

struct LiteralDoubler;

impl VisitMut for LiteralDoubler {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        if let Expr::Lit(ExprLit { lit: Lit::Int(lit_int), .. }) = expr {
            if let Ok(value) = lit_int.base10_parse::<i32>() {
                let doubled = value * 2;
                *lit_int = LitInt::new(
                    &doubled.to_string(),
                    lit_int.span()
                );
            }
        }
        
        // Continue visiting
        visit_mut::visit_expr_mut(self, expr);
    }
}

#[test]
fn test_mutable_visitor() {
    let mut expr: Expr = parse_quote! { 5 + 10 };
    
    let mut doubler = LiteralDoubler;
    doubler.visit_expr_mut(&mut expr);
    
    // Should be 10 + 20
    let result = quote::quote! { #expr }.to_string();
    assert!(result.contains("10"));
    assert!(result.contains("20"));
}
```

---

## BorrowScope-Specific Patterns

### Pattern 1: Identify Trackable Variables

```rust
use syn::{Local, Pat, Expr};

fn is_trackable_variable(local: &Local) -> bool {
    // Must have simple identifier pattern
    if !matches!(local.pat, Pat::Ident(_)) {
        return false;
    }
    
    // Must have initializer
    if local.init.is_none() {
        return false;
    }
    
    // Don't track underscore variables
    if let Pat::Ident(pat_ident) = &local.pat {
        if pat_ident.ident.to_string().starts_with('_') {
            return false;
        }
    }
    
    true
}

#[test]
fn test_is_trackable() {
    let local1: Local = parse_quote! { let x = 5; };
    assert!(is_trackable_variable(&local1));
    
    let local2: Local = parse_quote! { let _unused = 5; };
    assert!(!is_trackable_variable(&local2));
    
    let local3: Local = parse_quote! { let (x, y) = (1, 2); };
    assert!(!is_trackable_variable(&local3));
}
```

### Pattern 2: Extract Borrow Information

```rust
use syn::{Expr, ExprReference};

#[derive(Debug, PartialEq)]
struct BorrowInfo {
    is_mutable: bool,
    borrowed_expr: String,
}

fn extract_borrow_info(expr: &Expr) -> Option<BorrowInfo> {
    if let Expr::Reference(reference) = expr {
        Some(BorrowInfo {
            is_mutable: reference.mutability.is_some(),
            borrowed_expr: quote::quote! { #reference.expr }.to_string(),
        })
    } else {
        None
    }
}

#[test]
fn test_extract_borrow_info() {
    let expr1: Expr = parse_quote! { &x };
    let info1 = extract_borrow_info(&expr1).unwrap();
    assert!(!info1.is_mutable);
    
    let expr2: Expr = parse_quote! { &mut x };
    let info2 = extract_borrow_info(&expr2).unwrap();
    assert!(info2.is_mutable);
}
```

### Pattern 3: Classify Expressions

```rust
use syn::Expr;

#[derive(Debug, PartialEq)]
enum ExprKind {
    Literal,
    Variable,
    Borrow,
    FunctionCall,
    MethodCall,
    Other,
}

fn classify_expr(expr: &Expr) -> ExprKind {
    match expr {
        Expr::Lit(_) => ExprKind::Literal,
        Expr::Path(_) => ExprKind::Variable,
        Expr::Reference(_) => ExprKind::Borrow,
        Expr::Call(_) => ExprKind::FunctionCall,
        Expr::MethodCall(_) => ExprKind::MethodCall,
        _ => ExprKind::Other,
    }
}

#[test]
fn test_classify_expr() {
    assert_eq!(classify_expr(&parse_quote!(5)), ExprKind::Literal);
    assert_eq!(classify_expr(&parse_quote!(x)), ExprKind::Variable);
    assert_eq!(classify_expr(&parse_quote!(&x)), ExprKind::Borrow);
    assert_eq!(classify_expr(&parse_quote!(foo())), ExprKind::FunctionCall);
}
```

---

## Complex AST Examples

### Example 1: Nested Blocks

```rust
use syn::{parse_quote, ItemFn, Stmt, Expr};

fn count_nested_blocks(func: &ItemFn) -> usize {
    let mut count = 0;
    
    fn count_in_stmts(stmts: &[Stmt], count: &mut usize) {
        for stmt in stmts {
            match stmt {
                Stmt::Expr(Expr::Block(block), _) => {
                    *count += 1;
                    count_in_stmts(&block.block.stmts, count);
                }
                _ => {}
            }
        }
    }
    
    count_in_stmts(&func.block.stmts, &mut count);
    count
}

#[test]
fn test_nested_blocks() {
    let func: ItemFn = parse_quote! {
        fn example() {
            {
                let x = 5;
                {
                    let y = 10;
                }
            }
        }
    };
    
    assert_eq!(count_nested_blocks(&func), 2);
}
```

### Example 2: Method Chains

```rust
use syn::{parse_quote, Expr, ExprMethodCall};

fn count_method_calls(expr: &Expr) -> usize {
    match expr {
        Expr::MethodCall(method_call) => {
            1 + count_method_calls(&method_call.receiver)
        }
        _ => 0,
    }
}

#[test]
fn test_method_chains() {
    let expr: Expr = parse_quote! {
        s.to_string().to_uppercase().trim()
    };
    
    assert_eq!(count_method_calls(&expr), 3);
}
```

### Example 3: Match Expressions

```rust
use syn::{parse_quote, Expr, ExprMatch};

fn count_match_arms(expr: &Expr) -> usize {
    if let Expr::Match(ExprMatch { arms, .. }) = expr {
        arms.len()
    } else {
        0
    }
}

#[test]
fn test_match_arms() {
    let expr: Expr = parse_quote! {
        match x {
            1 => "one",
            2 => "two",
            _ => "other",
        }
    };
    
    assert_eq!(count_match_arms(&expr), 3);
}
```

---

## Debugging AST

### Print AST Structure

```rust
use syn::{parse_quote, ItemFn};

fn print_ast_debug() {
    let func: ItemFn = parse_quote! {
        fn example() {
            let x = 5;
        }
    };
    
    println!("{:#?}", func);
}
```

### Pretty Print AST

```rust
use syn::{parse_quote, ItemFn};
use quote::quote;

fn pretty_print_ast() {
    let func: ItemFn = parse_quote! {
        fn example() {
            let x = 5;
        }
    };
    
    let tokens = quote! { #func };
    let syntax_tree = syn::parse_file(&tokens.to_string()).unwrap();
    let formatted = prettyplease::unparse(&syntax_tree);
    
    println!("{}", formatted);
}
```

### Visualize AST

```rust
use syn::{Stmt, Expr, Local};

fn visualize_stmt(stmt: &Stmt, indent: usize) {
    let prefix = "  ".repeat(indent);
    
    match stmt {
        Stmt::Local(local) => {
            println!("{}Local", prefix);
            println!("{}  pat: {:?}", prefix, local.pat);
            if let Some(init) = &local.init {
                println!("{}  init:", prefix);
                visualize_expr(&init.expr, indent + 2);
            }
        }
        Stmt::Expr(expr, _) => {
            println!("{}Expr", prefix);
            visualize_expr(expr, indent + 1);
        }
        _ => {
            println!("{}Other", prefix);
        }
    }
}

fn visualize_expr(expr: &Expr, indent: usize) {
    let prefix = "  ".repeat(indent);
    
    match expr {
        Expr::Lit(lit) => println!("{}Lit: {:?}", prefix, lit.lit),
        Expr::Path(path) => println!("{}Path: {:?}", prefix, path.path),
        Expr::Reference(r) => {
            println!("{}Reference (mut: {})", prefix, r.mutability.is_some());
            visualize_expr(&r.expr, indent + 1);
        }
        _ => println!("{}Other expr", prefix),
    }
}
```

---

## Key Takeaways

### AST Structure

✅ **Hierarchical** - Items → Statements → Expressions  
✅ **Typed** - Each node has specific type  
✅ **Recursive** - Expressions contain expressions  
✅ **Rich** - Preserves all syntax information  
✅ **Spans** - Tracks source locations  

### Navigation

✅ **Pattern matching** - Match on enum variants  
✅ **Visitor pattern** - Traverse systematically  
✅ **Recursive descent** - Handle nested structures  
✅ **Type-safe** - Compiler helps catch errors  

### BorrowScope Patterns

✅ **Identify variables** - Pat::Ident in Local  
✅ **Find borrows** - Expr::Reference  
✅ **Detect moves** - Expr::Path assignments  
✅ **Track scope** - Block nesting  
✅ **Classify expressions** - Match on Expr variants  

---

## Exercises

### Exercise 1: Count All Variables

Write a function that counts all variable declarations in a function, including nested blocks.

### Exercise 2: Find All Function Calls

Extract all function calls (both regular and method calls) from a function.

### Exercise 3: Build Variable Dependency Graph

Create a graph showing which variables depend on which (e.g., `let y = x + 1` means y depends on x).

---

## What's Next?

In **Section 14: Implementing Basic Attribute Macro**, we'll:
- Implement the actual trace_borrow transformation
- Transform let statements
- Inject tracking calls
- Handle simple cases
- Test the transformation

---

**Previous Section:** [12-parsing-function-attributes.md](./12-parsing-function-attributes.md)  
**Next Section:** [14-implementing-basic-attribute-macro.md](./14-implementing-basic-attribute-macro.md)

**Chapter Progress:** 5/12 sections complete ⬛⬛⬛⬛⬛⬜⬜⬜⬜⬜⬜⬜

---

*"Understanding the AST is understanding the code itself." 🌳*
