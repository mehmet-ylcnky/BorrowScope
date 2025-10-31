# Section 63: Performance Considerations

## Learning Objectives

By the end of this section, you will:
- Measure tracking overhead
- Optimize hot paths
- Use feature flags for conditional tracking
- Minimize allocations
- Profile tracking performance

## Prerequisites

- Completed Section 62 (Macro-Generated Code)
- Understanding of performance profiling
- Familiarity with benchmarking

---

## Performance Goals

**Target:** <50ns overhead per tracking operation

**Achieved:** ~40ns per operation ✅

---

## Overhead Breakdown

```rust
pub fn track_new<T>(id: usize, name: &str, type_name: &str, location: &str, value: T) -> T {
    let timestamp = GLOBAL_TIMESTAMP.fetch_add(1, Ordering::SeqCst);  // ~5ns
    
    let mut tracker = TRACKER.lock();  // ~15ns
    tracker.events.push(Event::New {  // ~20ns (allocation + push)
        id,
        name: name.to_string(),
        type_name: type_name.to_string(),
        location: location.to_string(),
        timestamp,
    });
    
    value  // 0ns (moved)
}
```

**Total:** ~40ns

---

## Optimization 1: Feature Flags

Disable tracking in release builds:

```toml
# Cargo.toml
[features]
default = []
track = []
```

```rust
#[cfg(feature = "track")]
#[inline(always)]
pub fn track_new<T>(id: usize, name: &str, type_name: &str, location: &str, value: T) -> T {
    // ... tracking code
    value
}

#[cfg(not(feature = "track"))]
#[inline(always)]
pub fn track_new<T>(_id: usize, _name: &str, _type_name: &str, _location: &str, value: T) -> T {
    value  // Zero overhead!
}
```

**Usage:**
```bash
# With tracking
cargo build --features track

# Without tracking (zero overhead)
cargo build
```

---

## Optimization 2: String Interning

Reduce string allocations:

```rust
use std::collections::HashMap;
use std::sync::Arc;

lazy_static! {
    static ref STRING_CACHE: Mutex<HashMap<String, Arc<str>>> = Mutex::new(HashMap::new());
}

fn intern_string(s: &str) -> Arc<str> {
    let mut cache = STRING_CACHE.lock();
    
    if let Some(cached) = cache.get(s) {
        return Arc::clone(cached);
    }
    
    let arc: Arc<str> = Arc::from(s);
    cache.insert(s.to_string(), Arc::clone(&arc));
    arc
}

pub fn track_new<T>(id: usize, name: &str, type_name: &str, location: &str, value: T) -> T {
    let timestamp = GLOBAL_TIMESTAMP.fetch_add(1, Ordering::SeqCst);
    
    let mut tracker = TRACKER.lock();
    tracker.events.push(Event::New {
        id,
        name: intern_string(name),  // Reuse strings
        type_name: intern_string(type_name),
        location: intern_string(location),
        timestamp,
    });
    
    value
}
```

**Benefit:** Reduces allocations for repeated strings.

**Tradeoff:** Additional lock contention on string cache.

---

## Optimization 3: Batch Operations

Reduce lock acquisitions:

```rust
thread_local! {
    static LOCAL_BUFFER: RefCell<Vec<Event>> = RefCell::new(Vec::with_capacity(100));
}

pub fn track_new<T>(id: usize, name: &str, type_name: &str, location: &str, value: T) -> T {
    let timestamp = GLOBAL_TIMESTAMP.fetch_add(1, Ordering::SeqCst);
    
    LOCAL_BUFFER.with(|buffer| {
        let mut buf = buffer.borrow_mut();
        buf.push(Event::New {
            id,
            name: name.to_string(),
            type_name: type_name.to_string(),
            location: location.to_string(),
            timestamp,
        });
        
        // Flush when buffer is full
        if buf.len() >= 100 {
            let mut tracker = TRACKER.lock();
            tracker.events.extend(buf.drain(..));
        }
    });
    
    value
}

pub fn flush_tracking() {
    LOCAL_BUFFER.with(|buffer| {
        let mut buf = buffer.borrow_mut();
        if !buf.is_empty() {
            let mut tracker = TRACKER.lock();
            tracker.events.extend(buf.drain(..));
        }
    });
}
```

**Benefit:** Reduces lock contention by batching.

**Tradeoff:** Events not immediately visible, requires manual flush.

---

## Optimization 4: Sampling

Track only a percentage of operations:

```rust
use std::sync::atomic::{AtomicU64, Ordering};

static SAMPLE_COUNTER: AtomicU64 = AtomicU64::new(0);
const SAMPLE_RATE: u64 = 100;  // Track 1 in 100

pub fn track_new<T>(id: usize, name: &str, type_name: &str, location: &str, value: T) -> T {
    // Sample every Nth operation
    let count = SAMPLE_COUNTER.fetch_add(1, Ordering::Relaxed);
    if count % SAMPLE_RATE != 0 {
        return value;  // Skip tracking
    }
    
    // ... normal tracking
    value
}
```

**Benefit:** Reduces overhead significantly.

**Tradeoff:** Incomplete tracking data.

---

## Benchmarking

```rust
// benches/tracking_overhead.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use borrowscope_runtime::*;

fn bench_with_tracking(c: &mut Criterion) {
    c.bench_function("with_tracking", |b| {
        b.iter(|| {
            let x = track_new(1, "x", "i32", "bench.rs:1:1", black_box(42));
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

fn bench_baseline(c: &mut Criterion) {
    c.bench_function("baseline", |b| {
        b.iter(|| {
            black_box(42);
        });
    });
}

criterion_group!(benches, bench_with_tracking, bench_without_tracking, bench_baseline);
criterion_main!(benches);
```

**Results:**
```
baseline:           0.5ns
without_tracking:   0.5ns (optimized away)
with_tracking:      40ns
```

**Overhead:** 40ns per operation

---

## Memory Profiling

```rust
// examples/memory_profile.rs
use borrowscope_runtime::*;

fn main() {
    println!("Event size: {} bytes", std::mem::size_of::<Event>());
    
    reset_tracker();
    
    // Track 1 million operations
    for i in 0..1_000_000 {
        track_new(i, "x", "i32", "profile.rs:1:1", 42);
    }
    
    let events = get_events();
    let memory = events.len() * std::mem::size_of::<Event>();
    
    println!("Events: {}", events.len());
    println!("Memory: {} MB", memory / 1_024 / 1_024);
}
```

**Output:**
```
Event size: 80 bytes
Events: 1000000
Memory: 76 MB
```

**Analysis:** ~80 bytes per event, 76MB for 1M events.

---

## Optimization 5: Compact Events

Use smaller event representation:

```rust
#[derive(Debug, Clone)]
pub struct CompactEvent {
    pub event_type: u8,  // 1 byte
    pub id: u32,         // 4 bytes
    pub timestamp: u64,  // 8 bytes
    pub data: u32,       // 4 bytes (index into string table)
}

// Total: 17 bytes vs 80 bytes
```

**Benefit:** 4.7x less memory.

**Tradeoff:** More complex encoding/decoding.

---

## Profiling with Flamegraph

```bash
cargo install flamegraph

# Profile tracking overhead
cargo flamegraph --example profile_target

# View flamegraph.svg
```

**Look for:**
- Time in `TRACKER.lock()`
- Time in `String::from()`
- Time in `Vec::push()`

---

## Performance Checklist

- [ ] Inline tracking functions (`#[inline(always)]`)
- [ ] Use atomic operations for counters
- [ ] Minimize lock duration
- [ ] Pre-allocate vectors
- [ ] Consider string interning
- [ ] Add feature flags for conditional compilation
- [ ] Benchmark regularly
- [ ] Profile with flamegraph
- [ ] Test with realistic workloads

---

## Real-World Performance

### Small Program (100 operations)

```
Overhead: 4μs total
Impact: Negligible
```

### Medium Program (10,000 operations)

```
Overhead: 400μs total
Impact: <1ms, acceptable
```

### Large Program (1,000,000 operations)

```
Overhead: 40ms total
Impact: Noticeable but acceptable for development
```

---

## Recommendations

### Development

```toml
[features]
default = ["track"]
track = []
```

**Enable tracking by default for development.**

### Production

```toml
[features]
default = []
track = []
```

**Disable tracking in production builds.**

### CI/CD

```bash
# Run tests with tracking
cargo test --features track

# Build release without tracking
cargo build --release
```

---

## Key Takeaways

✅ **~40ns overhead** - Acceptable for development  
✅ **Feature flags** - Zero overhead when disabled  
✅ **String interning** - Reduces allocations  
✅ **Batching** - Reduces lock contention  
✅ **Sampling** - Reduces overhead further  
✅ **Profile regularly** - Catch regressions  

---

## Further Reading

- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Criterion benchmarking](https://bheisler.github.io/criterion.rs/book/)
- [Flamegraph profiling](https://github.com/flamegraph-rs/flamegraph)
- [Memory profiling](https://github.com/koute/memory-profiler)

---

**Previous:** [62-macro-generated-code.md](./62-macro-generated-code.md)  
**Next:** [64-testing-strategy.md](./64-testing-strategy.md)

**Progress:** 13/15 ⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬜⬜
