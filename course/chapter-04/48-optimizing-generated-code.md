# Section 48: Optimizing Generated Code

## Learning Objectives

By the end of this section, you will:
- Minimize generated code overhead
- Use inline annotations effectively
- Implement conditional compilation
- Optimize for release builds
- Measure transformation impact

## Prerequisites

- Completed Section 47 (Error Reporting)
- Understanding of Rust optimization
- Familiarity with inline and attributes

---

## Optimization Goals

1. **Zero overhead in release** - When tracking is disabled
2. **Minimal overhead when enabled** - <50ns per operation
3. **No code bloat** - Keep binary size reasonable
4. **Preserve optimizations** - Don't prevent compiler optimizations

---

## Inline Annotations

### Always Inline Tracking Functions

```rust
// borrowscope-runtime/src/tracker.rs

#[inline(always)]
pub fn track_new<T>(id: usize, name: &str, type_name: &str, location: &str, value: T) -> T {
    #[cfg(feature = "track")]
    {
        let timestamp = GLOBAL_TIMESTAMP.fetch_add(1, Ordering::SeqCst);
        let mut tracker = TRACKER.lock();
        tracker.events.push(Event::New {
            id,
            name: name.to_string(),
            type_name: type_name.to_string(),
            location: location.to_string(),
            timestamp,
        });
    }
    value
}
```

**Why `#[inline(always)]`?**
- Eliminates function call overhead
- Allows compiler to optimize away when tracking is disabled
- Enables dead code elimination

---

## Conditional Compilation

### Feature Flags

```toml
# Cargo.toml
[features]
default = []
track = []
```

### Conditional Code Generation

```rust
impl OwnershipVisitor {
    fn generate_tracking_call(&self, id: usize, name: &str, expr: &Expr) -> Expr {
        #[cfg(feature = "track")]
        {
            syn::parse_quote! {
                borrowscope_runtime::track_new(
                    #id,
                    #name,
                    "inferred",
                    "location",
                    #expr
                )
            }
        }
        
        #[cfg(not(feature = "track"))]
        {
            // Return expression unchanged
            expr.clone()
        }
    }
}
```

**Better approach:** Always generate tracking calls, but make them no-ops:

```rust
// Runtime function with feature flag
#[inline(always)]
pub fn track_new<T>(id: usize, name: &str, type_name: &str, location: &str, value: T) -> T {
    #[cfg(feature = "track")]
    {
        // Tracking code
    }
    value  // Always return value
}
```

**Benefit:** Macro code is simpler, optimization happens at runtime level.

---

## Minimize String Allocations

### Use &'static str When Possible

```rust
impl OwnershipVisitor {
    fn generate_tracking_call(&self, id: usize, name: &str, location: &str, expr: &Expr) -> Expr {
        // name and location are string literals from macro
        // They become &'static str in generated code
        syn::parse_quote! {
            borrowscope_runtime::track_new(
                #id,
                #name,        // &'static str
                "inferred",   // &'static str
                #location,    // &'static str
                #expr
            )
        }
    }
}
```

### Avoid Unnecessary Clones

```rust
// Bad: Clones expression multiple times
let expr1 = expr.clone();
let expr2 = expr.clone();

// Good: Clone once, use references
let expr_clone = expr.clone();
let expr_ref = &expr_clone;
```

---

## Reduce Generated Code Size

### Don't Duplicate Expressions

**Bad:**
```rust
let x = track_new(1, "x", "i32", "line:1:9", expensive_computation());
let y = track_new(2, "y", "i32", "line:2:9", expensive_computation());
```

**Good:**
```rust
let x = track_new(1, "x", "i32", "line:1:9", expensive_computation());
let y = track_new(2, "y", "i32", "line:2:9", x);  // Reuse x
```

### Reuse Common Subexpressions

```rust
impl OwnershipVisitor {
    fn optimize_tracking_calls(&mut self, block: &mut Block) {
        // Identify common patterns
        let mut seen_locations = HashMap::new();
        
        for stmt in &mut block.stmts {
            // Reuse location strings
            let location = self.get_location(stmt.span());
            let location_id = seen_locations
                .entry(location.clone())
                .or_insert_with(|| self.next_location_id());
            
            // Use location_id instead of string
        }
    }
}
```

---

## Optimize Drop Insertion

### Batch Drop Calls

Instead of:
```rust
track_drop(1, "scope_end");
track_drop(2, "scope_end");
track_drop(3, "scope_end");
```

Generate:
```rust
borrowscope_runtime::track_drop_batch(&[1, 2, 3], "scope_end");
```

**Implementation:**
```rust
// Runtime function
#[inline(always)]
pub fn track_drop_batch(ids: &[usize], location: &str) {
    #[cfg(feature = "track")]
    {
        let timestamp = GLOBAL_TIMESTAMP.fetch_add(1, Ordering::SeqCst);
        let mut tracker = TRACKER.lock();
        
        for &id in ids {
            tracker.events.push(Event::Drop {
                id,
                location: location.to_string(),
                timestamp,
            });
        }
    }
}
```

**Benefit:** One lock acquisition instead of N.

---

## Benchmarking Generated Code

```rust
// benches/generated_code.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_with_tracking(c: &mut Criterion) {
    c.bench_function("with_tracking", |b| {
        b.iter(|| {
            let x = borrowscope_runtime::track_new(1, "x", "i32", "bench.rs:1:1", black_box(42));
            black_box(x);
        });
    });
}

fn bench_without_tracking(c: &mut Criterion) {
    c.bench_function("without_tracking", |b| {
        b.iter(|| {
            let x = black_box(42);
            black_box(x);
        });
    });
}

criterion_group!(benches, bench_with_tracking, bench_without_tracking);
criterion_main!(benches);
```

**Run:**
```bash
# With tracking
cargo bench --features track

# Without tracking
cargo bench
```

---

## Code Size Analysis

```bash
# Check binary size
cargo build --release
ls -lh target/release/your_binary

# With tracking
cargo build --release --features track
ls -lh target/release/your_binary

# Analyze code size
cargo bloat --release
cargo bloat --release --features track
```

---

## Optimization Checklist

- [ ] All tracking functions marked `#[inline(always)]`
- [ ] Feature flags for conditional compilation
- [ ] String literals used for static strings
- [ ] Minimal expression cloning
- [ ] Batch operations where possible
- [ ] No unnecessary allocations
- [ ] Benchmarks show acceptable overhead
- [ ] Binary size increase is reasonable

---

## Advanced Optimizations

### 1. Const Evaluation

```rust
// Use const for compile-time computation
const fn compute_id(line: u32, col: u32) -> usize {
    (line as usize) << 16 | (col as usize)
}

// In generated code
track_new(compute_id(5, 9), "x", "i32", "line:5:9", value)
```

### 2. Type-Based Optimization

```rust
impl OwnershipVisitor {
    fn should_track_type(&self, type_name: &str) -> bool {
        // Don't track primitive types in release mode
        #[cfg(not(debug_assertions))]
        {
            if matches!(type_name, "i32" | "u32" | "bool" | "char") {
                return false;
            }
        }
        true
    }
}
```

### 3. Lazy Initialization

```rust
// Instead of always creating strings
track_new(1, "x", "i32", "line:5:9", value)

// Use lazy string creation
track_new_lazy(1, || "x", || "i32", || "line:5:9", value)
```

---

## Measuring Impact

### Compile Time

```bash
# Measure compile time
cargo clean
time cargo build

cargo clean
time cargo build --features track
```

### Runtime Performance

```rust
#[test]
fn measure_overhead() {
    use std::time::Instant;
    
    let start = Instant::now();
    for i in 0..1_000_000 {
        let x = track_new(i, "x", "i32", "test.rs:1:1", 42);
        std::hint::black_box(x);
    }
    let duration = start.elapsed();
    
    println!("1M operations: {:?}", duration);
    println!("Per operation: {:?}", duration / 1_000_000);
}
```

### Memory Usage

```rust
#[test]
fn measure_memory() {
    reset_tracker();
    
    for i in 0..100_000 {
        track_new(i, "x", "i32", "test.rs:1:1", 42);
    }
    
    let events = get_events();
    let memory = events.len() * std::mem::size_of::<Event>();
    
    println!("100K events: {} MB", memory / 1_024 / 1_024);
}
```

---

## Release Profile Optimization

```toml
# Cargo.toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

**Impact on tracking:**
- Inlined functions are optimized away when disabled
- LTO enables cross-crate optimization
- Single codegen unit allows better optimization

---

## Key Takeaways

✅ **Inline always** - Eliminate function call overhead  
✅ **Feature flags** - Zero overhead when disabled  
✅ **Minimize allocations** - Use &'static str  
✅ **Batch operations** - Reduce lock contention  
✅ **Benchmark regularly** - Measure actual impact  
✅ **Optimize for release** - Use LTO and opt-level 3  

---

## Further Reading

- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Inline optimization](https://doc.rust-lang.org/reference/attributes/codegen.html#the-inline-attribute)
- [LTO](https://doc.rust-lang.org/cargo/reference/profiles.html#lto)
- [cargo-bloat](https://github.com/RazrFalcon/cargo-bloat)

---

**Previous:** [47-error-reporting-in-macros.md](./47-error-reporting-in-macros.md)  
**Next:** [49-handling-generic-functions.md](./49-handling-generic-functions.md)

**Progress:** 13/15 ⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬜⬜
