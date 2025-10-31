# Section 30: Documentation and Examples

## Learning Objectives

By the end of this section, you will:
- Write comprehensive rustdoc comments
- Create runnable examples
- Document public APIs
- Add usage guides
- Generate documentation

## Prerequisites

- Completed Section 29 (Benchmarking Suite)
- Understanding of rustdoc
- Familiarity with documentation best practices

---

## Documenting the Runtime API

Update `borrowscope-runtime/src/lib.rs`:

```rust
//! BorrowScope Runtime - Event tracking and graph generation
//!
//! This crate provides the runtime support for BorrowScope, tracking
//! ownership and borrowing events at runtime.
//!
//! # Examples
//!
//! ```
//! use borrowscope_runtime::*;
//!
//! let x = track_new(1, "x", "i32", "main.rs:1:1", 42);
//! let r = track_borrow(2, 1, false, "main.rs:2:1", &x);
//! track_drop(2, "main.rs:3:1");
//! track_drop(1, "main.rs:4:1");
//!
//! let json = export_json().unwrap();
//! println!("{}", json);
//! ```

pub mod event;
pub mod tracker;
pub mod graph;
pub mod export;
pub mod error;

pub use event::Event;
pub use tracker::{track_new, track_borrow, track_borrow_mut, track_move, track_drop, export_json, reset_tracker};
pub use graph::{OwnershipGraph, Variable, Relationship};
pub use export::{ExportData, ExportNode, ExportEdge};
pub use error::{Error, Result};
```

---

## Function Documentation

Update `borrowscope-runtime/src/tracker.rs`:

```rust
/// Track creation of a new variable
///
/// # Arguments
///
/// * `id` - Unique identifier for the variable
/// * `name` - Variable name
/// * `type_name` - Type name as string
/// * `location` - Source location (file:line:col)
/// * `value` - The actual value (returned unchanged)
///
/// # Returns
///
/// The value unchanged (zero-cost abstraction)
///
/// # Examples
///
/// ```
/// use borrowscope_runtime::track_new;
///
/// let x = track_new(1, "x", "i32", "main.rs:5:9", 42);
/// assert_eq!(x, 42);
/// ```
#[inline(always)]
pub fn track_new<T>(id: usize, name: &str, type_name: &str, location: &str, value: T) -> T {
    // Implementation...
}
```

---

## Examples

Create `borrowscope-runtime/examples/basic_usage.rs`:

```rust
//! Basic usage example

use borrowscope_runtime::*;

fn main() {
    // Track a simple variable
    let x = track_new(1, "x", "i32", "example.rs:7:9", 42);
    println!("x = {}", x);
    
    // Track a borrow
    let r = track_borrow(2, 1, false, "example.rs:11:9", &x);
    println!("r = {}", r);
    
    // Clean up
    track_drop(2, "example.rs:14:1");
    track_drop(1, "example.rs:15:1");
    
    // Export to JSON
    let json = export_json().unwrap();
    println!("\nTracking data:\n{}", json);
}
```

Run example:

```bash
cargo run --package borrowscope-runtime --example basic_usage
```

---

## Generate Documentation

```bash
cargo doc --package borrowscope-runtime --open
```

---

## Key Takeaways

✅ **Document public APIs** - Clear usage examples  
✅ **Runnable examples** - Verify documentation works  
✅ **Module-level docs** - Explain purpose and usage  

---

**Previous:** [29-benchmarking-suite.md](./29-benchmarking-suite.md)  
**Next:** [31-chapter-summary.md](./31-chapter-summary.md)

**Progress:** 10/15 ⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬜⬜⬜⬜⬜
