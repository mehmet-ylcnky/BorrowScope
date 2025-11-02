use borrowscope_runtime::*;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::time::Duration;

fn bench_tracking_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("tracking_overhead");

    group.bench_function("baseline_no_op", |b| {
        b.iter(|| {
            black_box(42);
        });
    });

    group.bench_function("track_new_overhead", |b| {
        b.iter(|| {
            reset();
            let x = track_new("x", black_box(42));
            black_box(x);
        });
    });

    group.bench_function("track_borrow_overhead", |b| {
        b.iter(|| {
            reset();
            let x = 42;
            let r = track_borrow("r", black_box(&x));
            black_box(r);
        });
    });

    group.bench_function("track_move_overhead", |b| {
        b.iter(|| {
            reset();
            let x = track_move("x", "y", black_box(42));
            black_box(x);
        });
    });

    group.bench_function("track_drop_overhead", |b| {
        b.iter(|| {
            reset();
            track_drop(black_box("x"));
        });
    });

    group.finish();
}

fn bench_smart_pointer_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("smart_pointer_overhead");

    group.bench_function("rc_new_baseline", |b| {
        b.iter(|| {
            let rc = std::rc::Rc::new(black_box(42));
            black_box(rc);
        });
    });

    group.bench_function("rc_new_tracked", |b| {
        b.iter(|| {
            reset();
            let rc = std::rc::Rc::new(black_box(42));
            let tracked = track_rc_new("rc", rc);
            black_box(tracked);
        });
    });

    group.bench_function("rc_clone_baseline", |b| {
        let rc = std::rc::Rc::new(42);
        b.iter(|| {
            let cloned = std::rc::Rc::clone(black_box(&rc));
            black_box(cloned);
        });
    });

    group.bench_function("rc_clone_tracked", |b| {
        reset();
        let rc = std::rc::Rc::new(42);
        b.iter(|| {
            let cloned = std::rc::Rc::clone(black_box(&rc));
            let tracked = track_rc_clone("rc2", "rc1", cloned);
            black_box(tracked);
        });
    });

    group.bench_function("arc_new_baseline", |b| {
        b.iter(|| {
            let arc = std::sync::Arc::new(black_box(42));
            black_box(arc);
        });
    });

    group.bench_function("arc_new_tracked", |b| {
        b.iter(|| {
            reset();
            let arc = std::sync::Arc::new(black_box(42));
            let tracked = track_arc_new("arc", arc);
            black_box(tracked);
        });
    });

    group.finish();
}

fn bench_interior_mutability_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("interior_mutability_overhead");

    group.bench_function("refcell_new_baseline", |b| {
        b.iter(|| {
            let cell = std::cell::RefCell::new(black_box(42));
            black_box(cell);
        });
    });

    group.bench_function("refcell_new_tracked", |b| {
        b.iter(|| {
            reset();
            let cell = std::cell::RefCell::new(black_box(42));
            let tracked = track_refcell_new("cell", cell);
            black_box(tracked);
        });
    });

    group.bench_function("refcell_borrow_baseline", |b| {
        let cell = std::cell::RefCell::new(42);
        b.iter(|| {
            let borrowed = cell.borrow();
            black_box(&*borrowed);
        });
    });

    group.bench_function("refcell_borrow_tracked", |b| {
        reset();
        let cell = std::cell::RefCell::new(42);
        b.iter(|| {
            let borrowed = cell.borrow();
            let tracked = track_refcell_borrow("borrow", "cell", "test.rs:1:1", borrowed);
            black_box(&*tracked);
        });
    });

    group.bench_function("cell_new_baseline", |b| {
        b.iter(|| {
            let cell = std::cell::Cell::new(black_box(42));
            black_box(cell);
        });
    });

    group.bench_function("cell_new_tracked", |b| {
        b.iter(|| {
            reset();
            let cell = std::cell::Cell::new(black_box(42));
            let tracked = track_cell_new("cell", cell);
            black_box(tracked);
        });
    });

    group.finish();
}

fn bench_unsafe_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("unsafe_overhead");

    group.bench_function("raw_ptr_baseline", |b| {
        let x = 42;
        b.iter(|| {
            let ptr = &x as *const i32;
            black_box(ptr);
        });
    });

    group.bench_function("raw_ptr_tracked", |b| {
        reset();
        let x = 42;
        b.iter(|| {
            let ptr = &x as *const i32;
            let tracked = track_raw_ptr("ptr", 1, "i32", "test.rs:1:1", ptr);
            black_box(tracked);
        });
    });

    group.bench_function("unsafe_block_baseline", |b| {
        b.iter(|| {
            black_box(42);
        });
    });

    group.bench_function("unsafe_block_tracked", |b| {
        b.iter(|| {
            reset();
            track_unsafe_block_enter(1, "test.rs:1:1");
            black_box(42);
            track_unsafe_block_exit(1, "test.rs:1:1");
        });
    });

    group.finish();
}

fn bench_static_const_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("static_const_overhead");

    group.bench_function("static_init_baseline", |b| {
        b.iter(|| {
            black_box(42);
        });
    });

    group.bench_function("static_init_tracked", |b| {
        b.iter(|| {
            reset();
            let x = track_static_init("STATIC", 1, "i32", false, black_box(42));
            black_box(x);
        });
    });

    group.bench_function("static_access_baseline", |b| {
        b.iter(|| {
            black_box(42);
        });
    });

    group.bench_function("static_access_tracked", |b| {
        b.iter(|| {
            reset();
            track_static_access(1, "STATIC", false, "test.rs:1:1");
            black_box(42);
        });
    });

    group.bench_function("const_eval_baseline", |b| {
        b.iter(|| {
            black_box(42);
        });
    });

    group.bench_function("const_eval_tracked", |b| {
        b.iter(|| {
            reset();
            let x = track_const_eval("CONST", 1, "i32", "test.rs:1:1", black_box(42));
            black_box(x);
        });
    });

    group.finish();
}

fn bench_batch_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_operations");
    group.measurement_time(Duration::from_secs(10));

    for size in [10, 100, 1000, 10000] {
        group.bench_with_input(
            BenchmarkId::new("sequential_new", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    reset();
                    for i in 0..size {
                        let x = track_new(&format!("var_{}", i), i);
                        black_box(x);
                    }
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("sequential_borrow", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    reset();
                    let values: Vec<_> = (0..size).collect();
                    for (i, val) in values.iter().enumerate() {
                        let r = track_borrow(&format!("ref_{}", i), val);
                        black_box(r);
                    }
                });
            },
        );

        group.bench_with_input(BenchmarkId::new("drop_batch", size), &size, |b, &size| {
            b.iter(|| {
                reset();
                let names: Vec<String> = (0..size).map(|i| format!("var_{}", i)).collect();
                let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
                track_drop_batch(&name_refs);
            });
        });

        group.bench_with_input(
            BenchmarkId::new("drop_individual", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    reset();
                    for i in 0..size {
                        track_drop(&format!("var_{}", i));
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_lock_contention(c: &mut Criterion) {
    let mut group = c.benchmark_group("lock_contention");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("single_thread_1000_ops", |b| {
        b.iter(|| {
            reset();
            for i in 0..1000 {
                let x = track_new(&format!("var_{}", i), i);
                black_box(x);
            }
        });
    });

    group.bench_function("2_threads_500_ops_each", |b| {
        b.iter(|| {
            reset();
            let handles: Vec<_> = (0..2)
                .map(|thread_id| {
                    std::thread::spawn(move || {
                        for i in 0..500 {
                            let x = track_new(&format!("var_{}_{}", thread_id, i), i);
                            black_box(x);
                        }
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }
        });
    });

    group.bench_function("4_threads_250_ops_each", |b| {
        b.iter(|| {
            reset();
            let handles: Vec<_> = (0..4)
                .map(|thread_id| {
                    std::thread::spawn(move || {
                        for i in 0..250 {
                            let x = track_new(&format!("var_{}_{}", thread_id, i), i);
                            black_box(x);
                        }
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }
        });
    });

    group.finish();
}

fn bench_memory_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_operations");

    group.bench_function("event_size", |b| {
        b.iter(|| {
            black_box(std::mem::size_of::<Event>());
        });
    });

    group.bench_function("get_events_1000", |b| {
        reset();
        for i in 0..1000 {
            track_new(&format!("var_{}", i), i);
        }

        b.iter(|| {
            let events = get_events();
            black_box(events.len());
        });
    });

    group.bench_function("build_graph_1000", |b| {
        reset();
        for i in 0..1000 {
            let name = format!("var_{}", i);
            let x = track_new(&name, i);
            let _r = track_borrow("r", &x);
            track_drop(&name);
        }
        let events = get_events();

        b.iter(|| {
            let graph = build_graph(&events);
            black_box(graph.nodes.len());
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_tracking_overhead,
    bench_smart_pointer_overhead,
    bench_interior_mutability_overhead,
    bench_unsafe_overhead,
    bench_static_const_overhead,
    bench_batch_operations,
    bench_lock_contention,
    bench_memory_operations
);
criterion_main!(benches);
