# Section 21: Designing the Runtime API

## Learning Objectives

By the end of this section, you will:
- Design a clean, efficient runtime API
- Understand zero-cost abstraction principles
- Plan the event tracking system
- Design the public interface
- Prepare for implementation

## Prerequisites

- Completed Chapter 2
- Understanding of Rust ownership
- Familiarity with API design

---

## Runtime Requirements

### What the Runtime Must Do

1. **Track Events** - Record ownership operations
2. **Build Graph** - Construct ownership relationships
3. **Export Data** - Serialize to JSON
4. **Thread Safety** - Work in concurrent code
5. **Zero Cost** - Minimal performance impact

### What the Macro Generates

```rust
// User writes:
let s = String::from("hello");

// Macro generates:
let s = borrowscope_runtime::track_new("s", String::from("hello"));
```

The runtime must make this **fast** and **safe**.

---

## API Design

### Core Functions

```rust
/// Track a new variable
pub fn track_new<T>(name: &str, value: T) -> T;

/// Track an immutable borrow
pub fn track_borrow<T>(name: &str, value: &T) -> &T;

/// Track a mutable borrow
pub fn track_borrow_mut<T>(name: &str, value: &mut T) -> &mut T;

/// Track a move
pub fn track_move<T>(from: &str, to: &str, value: T) -> T;

/// Track a drop
pub fn track_drop(name: &str);
```

### Export Functions

```rust
/// Export events to JSON file
pub fn export_json(path: &str) -> Result<(), std::io::Error>;

/// Get all events
pub fn get_events() -> Vec<Event>;

/// Get ownership graph
pub fn get_graph() -> OwnershipGraph;

/// Reset tracking state
pub fn reset();
```

---

## Zero-Cost Abstraction

### The Goal

```rust
// Without tracking:
let x = 5;

// With tracking:
let x = track_new("x", 5);

// Should have ZERO runtime cost in release builds
```

### Implementation Strategy

```rust
#[inline(always)]
pub fn track_new<T>(name: &str, value: T) -> T {
    #[cfg(feature = "tracking")]
    {
        // Record event
        TRACKER.lock().record_new(name);
    }
    
    // Always return value unchanged
    value
}
```

**Key points:**
- `#[inline(always)]` - Compiler inlines the function
- Return value unchanged - No wrapper, no overhead
- Optional tracking - Can be disabled

---

## Event Model

### Event Enum

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    New {
        timestamp: u64,
        var_name: String,
        var_id: String,
        type_name: String,
    },
    Borrow {
        timestamp: u64,
        borrower_name: String,
        borrower_id: String,
        owner_id: String,
        mutable: bool,
    },
    Move {
        timestamp: u64,
        from_id: String,
        to_name: String,
        to_id: String,
    },
    Drop {
        timestamp: u64,
        var_id: String,
    },
}
```

### Why This Design?

- **Tagged enum** - Easy to serialize
- **Timestamps** - Track order of operations
- **IDs** - Unique identification
- **Names** - Human-readable
- **Type info** - For visualization

---

## Graph Model

### Ownership Graph

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct OwnershipGraph {
    pub nodes: Vec<Variable>,
    pub edges: Vec<Relationship>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Variable {
    pub id: String,
    pub name: String,
    pub type_name: String,
    pub created_at: u64,
    pub dropped_at: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Relationship {
    Owns { from: String, to: String },
    BorrowsImmut { from: String, to: String, start: u64, end: u64 },
    BorrowsMut { from: String, to: String, start: u64, end: u64 },
}
```

### Graph Example

```rust
// Code:
let s = String::from("hello");
let r = &s;

// Graph:
nodes: [
    Variable { id: "s_0", name: "s", type_name: "String", ... },
    Variable { id: "r_1", name: "r", type_name: "&String", ... },
]
edges: [
    BorrowsImmut { from: "r_1", to: "s_0", start: 2, end: 3 },
]
```

---

## Thread Safety

### Global Tracker

```rust
use lazy_static::lazy_static;
use parking_lot::Mutex;

lazy_static! {
    static ref TRACKER: Mutex<Tracker> = Mutex::new(Tracker::new());
}

struct Tracker {
    events: Vec<Event>,
    timestamp: AtomicU64,
}
```

### Why This Design?

- **lazy_static** - Initialize on first use
- **Mutex** - Thread-safe access
- **parking_lot** - Faster than std::sync::Mutex
- **AtomicU64** - Lock-free timestamp generation

---

## Performance Considerations

### Minimize Lock Contention

```rust
#[inline(always)]
pub fn track_new<T>(name: &str, value: T) -> T {
    // Quick timestamp generation (lock-free)
    let timestamp = TIMESTAMP.fetch_add(1, Ordering::Relaxed);
    
    // Short critical section
    {
        let mut tracker = TRACKER.lock();
        tracker.record_new(name, timestamp);
    } // Lock released immediately
    
    value
}
```

### Lazy Graph Construction

```rust
// Don't build graph on every event
pub fn get_graph() -> OwnershipGraph {
    let tracker = TRACKER.lock();
    
    // Build graph from events only when requested
    build_graph_from_events(&tracker.events)
}
```

---

## File Structure

### File: `borrowscope-runtime/src/lib.rs`

```rust
//! Runtime tracking for BorrowScope

mod event;
mod tracker;
mod graph;
mod export;

pub use event::Event;
pub use graph::OwnershipGraph;

// Public API
pub use tracker::{
    track_new,
    track_borrow,
    track_borrow_mut,
    track_move,
    track_drop,
    reset,
};

pub use export::{
    export_json,
    get_events,
    get_graph,
};
```

---

## API Usage Examples

### Example 1: Simple Tracking

```rust
use borrowscope_runtime::*;

fn main() {
    reset();
    
    let x = track_new("x", 5);
    let y = track_new("y", 10);
    let z = x + y;
    
    track_drop("y");
    track_drop("x");
    
    export_json("output.json").unwrap();
}
```

### Example 2: Borrow Tracking

```rust
use borrowscope_runtime::*;

fn main() {
    reset();
    
    let s = track_new("s", String::from("hello"));
    let r = track_borrow("r", &s);
    
    println!("{}", r);
    
    track_drop("r");
    track_drop("s");
    
    let events = get_events();
    println!("Tracked {} events", events.len());
}
```

---

## Design Principles

### 1. Simplicity

```rust
// ✅ Simple, clear API
track_new("x", value)

// ❌ Complex, confusing API
track_variable(TrackingOptions {
    name: "x",
    value: value,
    mode: TrackingMode::New,
})
```

### 2. Type Safety

```rust
// ✅ Generic, works with any type
pub fn track_new<T>(name: &str, value: T) -> T

// ❌ Requires boxing, loses type info
pub fn track_new(name: &str, value: Box<dyn Any>) -> Box<dyn Any>
```

### 3. Zero Cost

```rust
// ✅ Returns value directly
pub fn track_new<T>(name: &str, value: T) -> T {
    // ... tracking ...
    value
}

// ❌ Wraps value, adds overhead
pub fn track_new<T>(name: &str, value: T) -> Tracked<T> {
    Tracked { value, metadata: ... }
}
```

---

## Configuration

### Feature Flags

```toml
[features]
default = ["tracking"]
tracking = []
websocket = ["tokio", "tokio-tungstenite"]
```

### Conditional Compilation

```rust
#[cfg(feature = "tracking")]
pub fn track_new<T>(name: &str, value: T) -> T {
    // Full tracking
    TRACKER.lock().record_new(name);
    value
}

#[cfg(not(feature = "tracking"))]
#[inline(always)]
pub fn track_new<T>(_name: &str, value: T) -> T {
    // No-op, zero cost
    value
}
```

---

## Error Handling

### Design Decisions

```rust
// ✅ Tracking never panics
pub fn track_new<T>(name: &str, value: T) -> T {
    if let Ok(mut tracker) = TRACKER.try_lock() {
        tracker.record_new(name);
    }
    // If lock fails, just skip tracking
    value
}

// ✅ Export can fail
pub fn export_json(path: &str) -> Result<(), std::io::Error> {
    // File I/O can fail, return Result
}
```

**Principle:** Tracking should never break user code.

---

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_track_new_returns_value() {
    let value = track_new("x", 42);
    assert_eq!(value, 42);
}

#[test]
fn test_track_borrow_returns_reference() {
    let s = String::from("hello");
    let r = track_borrow("r", &s);
    assert_eq!(r, "hello");
}
```

### Integration Tests

```rust
#[test]
fn test_complete_workflow() {
    reset();
    
    let x = track_new("x", 5);
    let r = track_borrow("r", &x);
    track_drop("r");
    track_drop("x");
    
    let events = get_events();
    assert_eq!(events.len(), 4);
}
```

---

## Key Takeaways

### API Design

✅ **Simple** - Easy to use, hard to misuse  
✅ **Generic** - Works with any type  
✅ **Zero-cost** - No runtime overhead  
✅ **Thread-safe** - Works in concurrent code  
✅ **Flexible** - Can be extended  

### Implementation Strategy

✅ **Events** - Record all operations  
✅ **Graph** - Build from events  
✅ **Export** - Serialize to JSON  
✅ **Performance** - Minimize overhead  
✅ **Safety** - Never panic  

### Next Steps

In the following sections, we'll implement:
- Event tracking system
- Graph data structures
- JSON serialization
- Thread safety
- Performance optimization

---

## Exercises

### Exercise 1: API Design

Design an API for tracking function calls:
```rust
pub fn track_call(name: &str, args: ???) -> ???;
```

### Exercise 2: Event Design

Add a new event type for tracking closures:
```rust
ClosureCapture {
    closure_id: String,
    captured_vars: Vec<String>,
}
```

### Exercise 3: Performance Analysis

Calculate the theoretical overhead of tracking:
- Lock acquisition: ~20ns
- Event creation: ~10ns
- Vector push: ~5ns
- Total: ~35ns per operation

Is this acceptable?

---

## What's Next?

In **Section 22: Event Tracking System**, we'll:
- Implement the Event enum
- Create the Tracker struct
- Implement tracking functions
- Add timestamp generation
- Test the event system

---

**Previous Chapter:** [Chapter 2 Complete](../chapter-02/20-testing-procedural-macros.md)  
**Next Section:** [22-event-tracking-system.md](./22-event-tracking-system.md)

**Chapter Progress:** 1/15 sections complete ⬛⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜

---

*"Good API design is the foundation of good software." 🎯*
