# Section 29: Comprehensive Benchmarking Suite

## Learning Objectives

By the end of this section, you will:
- Create a complete benchmark suite
- Measure performance across scenarios
- Track performance regressions
- Generate benchmark reports
- Optimize based on data

## Prerequisites

- Completed Section 28 (Error Handling)
- Understanding of criterion benchmarks
- Familiarity with performance analysis

---

## Benchmark Organization

Create `borrowscope-runtime/benches/suite.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use borrowscope_runtime::*;

fn bench_tracking(c: &mut Criterion) {
    c.bench_function("track_new", |b| {
        b.iter(|| black_box(track_new(1, "x", "i32", "b.rs:1:1", 42)));
    });
    
    c.bench_function("track_borrow", |b| {
        let x = 42;
        b.iter(|| black_box(track_borrow(2, 1, false, "b.rs:2:1", &x)));
    });
    
    c.bench_function("track_drop", |b| {
        b.iter(|| black_box(track_drop(1, "b.rs:3:1")));
    });
}

fn bench_bulk(c: &mut Criterion) {
    let mut group = c.benchmark_group("bulk");
    
    for size in [100, 1000, 10000] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &s| {
            b.iter(|| {
                reset_tracker();
                for i in 0..s {
                    black_box(track_new(i, "x", "i32", "b.rs:1:1", 42));
                }
            });
        });
    }
    
    group.finish();
}

fn bench_export(c: &mut Criterion) {
    let mut group = c.benchmark_group("export");
    
    for size in [100, 1000, 10000] {
        reset_tracker();
        for i in 0..size {
            track_new(i, "x", "i32", "b.rs:1:1", 42);
        }
        
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| black_box(export_json().unwrap()));
        });
    }
    
    group.finish();
}

criterion_group!(benches, bench_tracking, bench_bulk, bench_export);
criterion_main!(benches);
```

Run benchmarks:

```bash
cargo bench --package borrowscope-runtime
```

---

## Performance Baselines

Create `borrowscope-runtime/benches/baseline.txt`:

```
track_new:      40ns
track_borrow:   40ns
track_drop:     35ns
bulk_100:       4μs
bulk_1000:      40μs
bulk_10000:     400μs
export_100:     50μs
export_1000:    500μs
export_10000:   5ms
```

Compare against baseline:

```bash
cargo bench --bench suite -- --save-baseline main
cargo bench --bench suite -- --baseline main
```

---

## Key Takeaways

✅ **Comprehensive benchmarks** - Cover all operations  
✅ **Baseline tracking** - Detect regressions  
✅ **Parameterized tests** - Test at scale  

---

**Previous:** [28-error-handling.md](./28-error-handling.md)  
**Next:** [30-documentation.md](./30-documentation.md)

**Progress:** 9/15 ⬛⬛⬛⬛⬛⬛⬛⬛⬛⬜⬜⬜⬜⬜⬜
