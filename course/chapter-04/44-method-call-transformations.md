# Section 44: Method Call Transformations

## Learning Objectives

By the end of this section, you will:
- Transform method call syntax
- Handle self borrows (implicit &self)
- Track chained method calls
- Detect method call patterns in AST
- Preserve method call semantics

## Prerequisites

- Completed Section 43 (Control Flow)
- Understanding of method call syntax
- Familiarity with receiver expressions

---

## Method Call Syntax

```rust
let s = String::from("hello");
let len = s.len();        // Method call
let upper = s.to_uppercase();  // Consumes self
```

**AST representation:**
```rust
ExprMethodCall {
    receiver: Expr,      // s
    method: Ident,       // len
    args: Vec<Expr>,     // []
}
```

---

## Self Borrows

Methods implicitly borrow self:

```rust
impl String {
    fn len(&self) -> usize { ... }           // Borrows immutably
    fn push(&mut self, ch: char) { ... }     // Borrows mutably
    fn into_bytes(self) -> Vec<u8> { ... }   // Consumes self
}
```

**Challenge:** Detect the borrow type from method signature.

---

## Detection Strategy

```rust
impl OwnershipVisitor {
    fn detect_method_call(&self, expr: &Expr) -> Option<MethodCallInfo> {
        if let Expr::MethodCall(method_call) = expr {
            Some(MethodCallInfo {
                receiver: &method_call.receiver,
                method_name: method_call.method.to_string(),
                args: &method_call.args,
            })
        } else {
            None
        }
    }
    
    fn infer_self_borrow_type(&self, method_name: &str) -> SelfBorrowType {
        // Heuristics for common methods
        match method_name {
            // Immutable borrows
            "len" | "is_empty" | "as_str" | "as_bytes" | "get" | "iter" => {
                SelfBorrowType::Immutable
            }
            // Mutable borrows
            "push" | "pop" | "insert" | "remove" | "clear" | "push_str" => {
                SelfBorrowType::Mutable
            }
            // Consuming methods
            "into_bytes" | "into_iter" | "into_boxed_str" => {
                SelfBorrowType::Consuming
            }
            // Default: assume immutable borrow
            _ => SelfBorrowType::Immutable
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum SelfBorrowType {
    Immutable,
    Mutable,
    Consuming,
}

struct MethodCallInfo<'a> {
    receiver: &'a Expr,
    method_name: String,
    args: &'a Punctuated<Expr, Token![,]>,
}
```

---

## Transformation Strategy

### Immutable Method Call

**Input:**
```rust
let s = String::from("hello");
let len = s.len();
```

**Output:**
```rust
let s = track_new(1, "s", "String", "line:1:9", String::from("hello"));
let len = track_new(
    2,
    "len",
    "usize",
    "line:2:9",
    track_borrow(3, 1, false, "line:2:11", &s).len()
);
```

### Mutable Method Call

**Input:**
```rust
let mut s = String::from("hello");
s.push('!');
```

**Output:**
```rust
let mut s = track_new(1, "s", "String", "line:1:13", String::from("hello"));
track_borrow_mut(2, 1, true, "line:2:1", &mut s).push('!');
```

### Consuming Method Call

**Input:**
```rust
let s = String::from("hello");
let bytes = s.into_bytes();
```

**Output:**
```rust
let s = track_new(1, "s", "String", "line:1:9", String::from("hello"));
track_move(1, 2, "line:2:13");
let bytes = s.into_bytes();
```

---

## Implementation

```rust
impl VisitMut for OwnershipVisitor {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        match expr {
            Expr::MethodCall(method_call) => {
                self.transform_method_call(method_call);
            }
            _ => {
                visit_mut::visit_expr_mut(self, expr);
            }
        }
    }
}

impl OwnershipVisitor {
    fn transform_method_call(&mut self, method_call: &mut syn::ExprMethodCall) {
        let method_name = method_call.method.to_string();
        let borrow_type = self.infer_self_borrow_type(&method_name);
        
        // Extract receiver variable ID
        let receiver_id = self.extract_receiver_id(&method_call.receiver);
        
        match borrow_type {
            SelfBorrowType::Immutable => {
                self.wrap_with_immutable_borrow(method_call, receiver_id);
            }
            SelfBorrowType::Mutable => {
                self.wrap_with_mutable_borrow(method_call, receiver_id);
            }
            SelfBorrowType::Consuming => {
                // Insert track_move before the call
                // This requires statement-level transformation
                // For now, just visit the receiver
                self.visit_expr_mut(&mut method_call.receiver);
            }
        }
        
        // Visit arguments
        for arg in &mut method_call.args {
            self.visit_expr_mut(arg);
        }
    }
    
    fn wrap_with_immutable_borrow(&mut self, method_call: &mut syn::ExprMethodCall, receiver_id: usize) {
        let borrow_id = self.next_id();
        let location = self.get_location(method_call.span());
        
        // Clone the receiver
        let receiver = method_call.receiver.clone();
        
        // Wrap receiver with track_borrow
        let wrapped_receiver: Expr = syn::parse_quote! {
            borrowscope_runtime::track_borrow(
                #borrow_id,
                #receiver_id,
                false,
                #location,
                &#receiver
            )
        };
        
        // Replace receiver
        method_call.receiver = Box::new(wrapped_receiver);
    }
    
    fn wrap_with_mutable_borrow(&mut self, method_call: &mut syn::ExprMethodCall, receiver_id: usize) {
        let borrow_id = self.next_id();
        let location = self.get_location(method_call.span());
        
        let receiver = method_call.receiver.clone();
        
        let wrapped_receiver: Expr = syn::parse_quote! {
            borrowscope_runtime::track_borrow_mut(
                #borrow_id,
                #receiver_id,
                true,
                #location,
                &mut #receiver
            )
        };
        
        method_call.receiver = Box::new(wrapped_receiver);
    }
    
    fn extract_receiver_id(&self, receiver: &Expr) -> usize {
        if let Expr::Path(path) = receiver {
            if let Some(ident) = path.path.get_ident() {
                let var_name = ident.to_string();
                return *self.var_ids.get(&var_name).unwrap_or(&0);
            }
        }
        0
    }
}
```

---

## Chained Method Calls

```rust
let result = s.trim().to_uppercase().len();
```

**Challenge:** Each method call borrows the result of the previous call.

**Transformation:**
```rust
let result = track_new(
    4,
    "result",
    "usize",
    "line:1:14",
    track_borrow(3, 2, false, "line:1:28",
        &track_borrow(2, 1, false, "line:1:16",
            &track_borrow(1, 0, false, "line:1:14", &s).trim()
        ).to_uppercase()
    ).len()
);
```

**Implementation:** Recursive transformation handles this automatically.

---

## Testing

```rust
#[test]
fn test_method_call_immutable() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            let s = String::from("hello");
            let len = s.len();
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    
    assert!(output.contains("track_borrow"));
    assert!(output.contains("false"));  // Immutable
}

#[test]
fn test_method_call_mutable() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            let mut s = String::from("hello");
            s.push('!');
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    
    assert!(output.contains("track_borrow_mut"));
    assert!(output.contains("true"));  // Mutable
}

#[test]
fn test_chained_method_calls() {
    let mut visitor = OwnershipVisitor::new();
    
    let mut block: syn::Block = parse_quote! {
        {
            let s = String::from("  hello  ");
            let result = s.trim().to_uppercase();
        }
    };
    
    visitor.visit_block_mut(&mut block);
    
    let output = block.to_token_stream().to_string();
    
    // Should have multiple track_borrow calls
    assert!(output.matches("track_borrow").count() >= 2);
}
```

---

## Integration Test

```rust
use borrowscope_runtime::*;

#[test]
fn test_method_call_tracking() {
    reset_tracker();
    
    let s = track_new(1, "s", "String", "test.rs:1:1", String::from("hello"));
    let len = track_new(
        2,
        "len",
        "usize",
        "test.rs:2:1",
        track_borrow(3, 1, false, "test.rs:2:11", &s).len()
    );
    
    assert_eq!(len, 5);
    
    let events = get_events();
    
    // Should have: New(s), New(len), Borrow(s)
    assert_eq!(events.len(), 3);
}
```

---

## Method Call Heuristics

Expand the heuristics for better detection:

```rust
impl OwnershipVisitor {
    fn infer_self_borrow_type(&self, method_name: &str) -> SelfBorrowType {
        // Immutable borrows (common patterns)
        if method_name.starts_with("as_") ||
           method_name.starts_with("to_") ||
           method_name.starts_with("is_") ||
           method_name.starts_with("get") ||
           matches!(method_name, 
               "len" | "capacity" | "iter" | "chars" | "bytes" |
               "lines" | "split" | "trim" | "contains" | "starts_with" |
               "ends_with" | "find" | "clone"
           ) {
            return SelfBorrowType::Immutable;
        }
        
        // Mutable borrows (common patterns)
        if method_name.starts_with("push") ||
           method_name.starts_with("pop") ||
           method_name.starts_with("insert") ||
           method_name.starts_with("remove") ||
           method_name.starts_with("append") ||
           matches!(method_name,
               "clear" | "truncate" | "extend" | "drain" |
               "sort" | "reverse" | "dedup" | "retain"
           ) {
            return SelfBorrowType::Mutable;
        }
        
        // Consuming methods (common patterns)
        if method_name.starts_with("into_") ||
           matches!(method_name, "unwrap" | "expect") {
            return SelfBorrowType::Consuming;
        }
        
        // Default: immutable borrow
        SelfBorrowType::Immutable
    }
}
```

---

## Edge Cases

### Case 1: Method on Temporary

```rust
let len = String::from("hello").len();
```

**Solution:** Don't track the temporary.

```rust
fn transform_method_call(&mut self, method_call: &mut syn::ExprMethodCall) {
    // Check if receiver is a simple variable
    if !self.is_simple_variable(&method_call.receiver) {
        // Don't track method calls on temporaries
        visit_mut::visit_expr_method_call_mut(self, method_call);
        return;
    }
    
    // ... normal transformation
}

fn is_simple_variable(&self, expr: &Expr) -> bool {
    matches!(expr, Expr::Path(_))
}
```

### Case 2: Method on Field

```rust
let len = point.x.abs();
```

**Solution:** Track the struct, not the field.

### Case 3: Generic Methods

```rust
let v = vec![1, 2, 3];
let first = v.get(0);
```

**Solution:** Heuristics work for common generic methods.

---

## Key Takeaways

✅ **Method calls implicitly borrow** - Detect and track  
✅ **Use heuristics** - Infer borrow type from method name  
✅ **Wrap receiver** - Insert tracking around receiver expression  
✅ **Handle chaining** - Recursive transformation works  
✅ **Skip temporaries** - Only track variable receivers  

---

## Further Reading

- [Method call syntax](https://doc.rust-lang.org/reference/expressions/method-call-expr.html)
- [Method resolution](https://doc.rust-lang.org/reference/expressions/method-call-expr.html#method-call-expressions)
- [Deref coercion](https://doc.rust-lang.org/book/ch15-02-deref.html)

---

**Previous:** [43-handling-control-flow.md](./43-handling-control-flow.md)  
**Next:** [45-closure-capture-analysis.md](./45-closure-capture-analysis.md)

**Progress:** 9/15 ⬛⬛⬛⬛⬛⬛⬛⬛⬛⬜⬜⬜⬜⬜⬜
