# Section 52: Lifetime Tracking Challenges

## Learning Objectives

By the end of this section, you will:
- Understand runtime vs compile-time lifetime differences
- Infer lifetime relationships from scope
- Recognize tracking limitations
- Design practical lifetime visualization
- Handle edge cases in lifetime tracking

## Prerequisites

- Completed Section 51 (Understanding Lifetimes)
- Understanding of scope and RAII
- Familiarity with borrow checker rules

---

## The Fundamental Challenge

**Compile-time:** Lifetimes are explicit or inferred by the compiler.

**Runtime:** Lifetimes don't exist. We only have:
- Variable creation/destruction
- Reference creation
- Scope boundaries

**Goal:** Infer lifetime relationships from runtime events.

---

## Approach: Scope-Based Inference

We can approximate lifetimes using scope:

```rust
{
    let x = 42;           // x's lifetime starts
    let r = &x;           // r's lifetime starts, borrows x
    println!("{}", r);
}                         // r's lifetime ends, then x's
```

**Inference:** If `r` borrows `x`, and `r` is dropped before `x`, then `r`'s lifetime is contained within `x`'s lifetime.

---

## Implementation Strategy

### Step 1: Track Borrow Relationships

Already implemented in Chapter 4:

```rust
Event::Borrow {
    id: 2,              // r's ID
    borrowed_id: 1,     // x's ID
    is_mutable: false,
    location: "line:3:13",
    timestamp: 1001,
}
```

### Step 2: Track Drop Order

Already implemented:

```rust
Event::Drop { id: 2, location: "scope_end", timestamp: 1002 }  // r dropped
Event::Drop { id: 1, location: "scope_end", timestamp: 1003 }  // x dropped
```

### Step 3: Infer Lifetime Relationships

Add to `borrowscope-runtime/src/graph.rs`:

```rust
#[derive(Debug, Clone)]
pub struct LifetimeRelation {
    /// The borrowing variable
    pub borrower_id: usize,
    /// The borrowed variable
    pub borrowed_id: usize,
    /// When the borrow started
    pub start_time: u64,
    /// When the borrow ended (None if still active)
    pub end_time: Option<u64>,
}

impl OwnershipGraph {
    /// Extract lifetime relationships from events
    pub fn lifetime_relations(&self) -> Vec<LifetimeRelation> {
        let mut relations = Vec::new();
        let mut active_borrows: HashMap<usize, (usize, u64)> = HashMap::new();
        
        for event in &self.events {
            match event {
                Event::Borrow { id, borrowed_id, timestamp, .. } => {
                    active_borrows.insert(*id, (*borrowed_id, *timestamp));
                }
                Event::Drop { id, timestamp, .. } => {
                    if let Some((borrowed_id, start_time)) = active_borrows.remove(id) {
                        relations.push(LifetimeRelation {
                            borrower_id: *id,
                            borrowed_id,
                            start_time,
                            end_time: Some(*timestamp),
                        });
                    }
                }
                _ => {}
            }
        }
        
        // Handle borrows that are still active
        for (borrower_id, (borrowed_id, start_time)) in active_borrows {
            relations.push(LifetimeRelation {
                borrower_id,
                borrowed_id,
                start_time,
                end_time: None,
            });
        }
        
        relations
    }
}
```

---

## Testing Lifetime Inference

Create `borrowscope-runtime/tests/lifetime_test.rs`:

```rust
use borrowscope_runtime::*;

#[test]
fn test_simple_lifetime() {
    reset_tracker();
    
    let x = track_new(1, "x", "i32", "test.rs:1:1", 42);
    let r = track_borrow(2, 1, false, "test.rs:2:1", &x);
    
    track_drop(2, "scope_end");
    track_drop(1, "scope_end");
    
    let events = get_events();
    let graph = OwnershipGraph::from_events(&events);
    let relations = graph.lifetime_relations();
    
    assert_eq!(relations.len(), 1);
    assert_eq!(relations[0].borrower_id, 2);
    assert_eq!(relations[0].borrowed_id, 1);
    assert!(relations[0].end_time.is_some());
}

#[test]
fn test_nested_lifetimes() {
    reset_tracker();
    
    let x = track_new(1, "x", "i32", "test.rs:1:1", 42);
    let r1 = track_borrow(2, 1, false, "test.rs:2:1", &x);
    let r2 = track_borrow(3, 1, false, "test.rs:3:1", &x);
    
    track_drop(3, "scope_end");  // r2 ends first
    track_drop(2, "scope_end");  // r1 ends second
    track_drop(1, "scope_end");  // x ends last
    
    let events = get_events();
    let graph = OwnershipGraph::from_events(&events);
    let relations = graph.lifetime_relations();
    
    assert_eq!(relations.len(), 2);
    
    // Both borrow x
    assert!(relations.iter().all(|r| r.borrowed_id == 1));
}
```

---

## Challenge 1: Multiple Borrows

```rust
fn example() {
    let x = 42;
    let r1 = &x;
    let r2 = &x;
}
```

**Question:** Do r1 and r2 have the same lifetime?

**Answer:** Not necessarily. They're independent borrows with potentially different lifetimes.

**Tracking:**
```
Borrow(r1, borrows=x)
Borrow(r2, borrows=x)
Drop(r2)
Drop(r1)
Drop(x)
```

**Visualization:** Show both as separate borrows of x.

---

## Challenge 2: Borrow of Borrow

```rust
fn example() {
    let x = 42;
    let r1 = &x;
    let r2 = &r1;  // Borrow of a borrow
}
```

**Tracking:**
```rust
let x = track_new(1, "x", "i32", "line:2:9", 42);
let r1 = track_borrow(2, 1, false, "line:3:14", &x);
let r2 = track_borrow(3, 2, false, "line:4:14", &r1);
```

**Inference:** r2's lifetime is contained in r1's, which is contained in x's.

---

## Challenge 3: Lifetime Elision

```rust
fn get_first(items: &[i32]) -> &i32 {
    &items[0]
}

fn main() {
    let vec = vec![1, 2, 3];
    let first = get_first(&vec);
}
```

**Problem:** The returned reference has the same lifetime as the input, but we don't see that at runtime.

**Solution:** Track that `first` is derived from a borrow of `vec`:

```rust
let vec = track_new(1, "vec", "Vec<i32>", "line:6:9", vec![1, 2, 3]);
let first = track_new(
    2,
    "first",
    "&i32",
    "line:7:9",
    get_first(track_borrow(3, 1, false, "line:7:19", &vec))
);
```

**Inference:** `first` depends on the borrow (id=3) of `vec` (id=1).

---

## Challenge 4: Struct Lifetimes

```rust
struct Wrapper<'a> {
    data: &'a str,
}

fn main() {
    let s = String::from("hello");
    let w = Wrapper { data: &s };
}
```

**Tracking:**
```rust
let s = track_new(1, "s", "String", "line:6:9", String::from("hello"));
let w = track_new(
    2,
    "w",
    "Wrapper",
    "line:7:9",
    Wrapper { data: track_borrow(3, 1, false, "line:7:28", &s) }
);
```

**Inference:** `w` contains a borrow of `s`, so `w`'s lifetime is tied to `s`.

---

## Challenge 5: Function Returns

```rust
fn create_ref(x: &i32) -> &i32 {
    x
}

fn main() {
    let x = 42;
    let r = create_ref(&x);
}
```

**Problem:** The function returns a reference, but we don't track the return.

**Solution:** Track the borrow passed to the function:

```rust
let x = track_new(1, "x", "i32", "line:6:9", 42);
let r = track_new(
    2,
    "r",
    "&i32",
    "line:7:9",
    create_ref(track_borrow(3, 1, false, "line:7:21", &x))
);
```

**Limitation:** We don't know that `r` is the same reference as the borrow (id=3).

---

## Practical Limitations

### Limitation 1: Can't Track All References

```rust
fn example() {
    let x = 42;
    foo(&x);  // We track the borrow
    // But we don't know what foo does with it
}
```

**Solution:** Track the borrow at the call site. Assume it ends when the function returns.

### Limitation 2: Can't Verify Lifetime Bounds

```rust
fn bounded<'a, 'b: 'a>(x: &'a i32, y: &'b i32) -> &'a i32 {
    y  // Valid because 'b: 'a
}
```

**Solution:** Trust the compiler. If it compiles, lifetimes are correct.

### Limitation 3: Can't Track Closure Captures

```rust
fn example() {
    let x = 42;
    let closure = || println!("{}", x);  // Captures x
}
```

**Solution:** Detect closure syntax, track captures (covered in Section 45).

---

## Visualization Strategy

For the UI, show lifetime relationships as a timeline:

```
Time →
|
|---- x (id=1) --------------------------------|
|       |---- r1 (id=2) ------------------|
|       |       |---- r2 (id=3) ------|
|
```

**Legend:**
- Horizontal bars = variable lifetimes
- Nested bars = borrow relationships
- Overlapping bars = concurrent borrows

---

## Export Format

Add to `borrowscope-runtime/src/export.rs`:

```rust
#[derive(Debug, Serialize)]
pub struct LifetimeVisualization {
    pub variables: Vec<VariableLifetime>,
    pub borrows: Vec<BorrowRelation>,
}

#[derive(Debug, Serialize)]
pub struct VariableLifetime {
    pub id: usize,
    pub name: String,
    pub start_time: u64,
    pub end_time: u64,
}

#[derive(Debug, Serialize)]
pub struct BorrowRelation {
    pub borrower_id: usize,
    pub borrowed_id: usize,
    pub start_time: u64,
    pub end_time: u64,
}

impl OwnershipGraph {
    pub fn lifetime_visualization(&self) -> LifetimeVisualization {
        let mut variables = Vec::new();
        let mut borrows = Vec::new();
        
        // Extract variable lifetimes
        for var in self.graph.node_weights() {
            if let Some(end_time) = var.dropped_at {
                variables.push(VariableLifetime {
                    id: var.id,
                    name: var.name.clone(),
                    start_time: var.created_at,
                    end_time,
                });
            }
        }
        
        // Extract borrow relations
        for relation in self.lifetime_relations() {
            if let Some(end_time) = relation.end_time {
                borrows.push(BorrowRelation {
                    borrower_id: relation.borrower_id,
                    borrowed_id: relation.borrowed_id,
                    start_time: relation.start_time,
                    end_time,
                });
            }
        }
        
        LifetimeVisualization { variables, borrows }
    }
}
```

---

## Key Takeaways

✅ **Scope approximates lifetimes** - Use drop order to infer relationships  
✅ **Track borrow relationships** - Record which variables borrow which  
✅ **Accept limitations** - Can't track everything at runtime  
✅ **Visualize timelines** - Show when variables and borrows are active  
✅ **Trust the compiler** - If it compiles, lifetimes are correct  

---

## Further Reading

- [Lifetime variance](https://doc.rust-lang.org/nomicon/subtyping.html)
- [Borrow checker internals](https://rustc-dev-guide.rust-lang.org/borrow_check.html)
- [Polonius](https://github.com/rust-lang/polonius) - Next-gen borrow checker

---

**Previous:** [51-understanding-rust-lifetimes-deeply.md](./51-understanding-rust-lifetimes-deeply.md)  
**Next:** [53-smart-pointers-overview.md](./53-smart-pointers-overview.md)

**Progress:** 2/15 ⬛⬛⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜
