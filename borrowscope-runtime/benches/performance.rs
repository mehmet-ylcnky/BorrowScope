use borrowscope_runtime::*;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn bench_single_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_operations");

    group.bench_function("track_new", |b| {
        b.iter(|| {
            reset();
            black_box(track_new("x", 42));
        });
    });

    group.bench_function("track_borrow", |b| {
        b.iter(|| {
            reset();
            let x = 42;
            black_box(track_borrow("r", &x));
        });
    });

    group.bench_function("track_borrow_mut", |b| {
        b.iter(|| {
            reset();
            let mut x = 42;
            black_box(track_borrow_mut("r", &mut x));
        });
    });

    group.bench_function("track_move", |b| {
        b.iter(|| {
            reset();
            black_box(track_move("x", "y", 42));
        });
    });

    group.bench_function("track_drop", |b| {
        b.iter(|| {
            reset();
            track_drop("x");
            black_box(());
        });
    });

    group.finish();
}

fn bench_bulk_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("bulk_operations");

    for size in [100, 1000, 10000] {
        group.bench_with_input(BenchmarkId::new("track_new", size), &size, |b, &size| {
            b.iter(|| {
                reset();
                for i in 0..size {
                    black_box(track_new(&format!("var_{}", i), i));
                }
            });
        });

        group.bench_with_input(
            BenchmarkId::new("mixed_operations", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    reset();
                    for i in 0..size {
                        let name = format!("var_{}", i);
                        let x = track_new(&name, i);
                        let _r = track_borrow("r", &x);
                        track_drop(&name);
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_graph_building(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph_building");

    for size in [100, 1000, 5000] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            reset();
            for i in 0..size {
                let name = format!("var_{}", i);
                let x = track_new(&name, i);
                let _r = track_borrow("r", &x);
                track_drop(&name);
            }

            b.iter(|| {
                let events = get_events();
                black_box(build_graph(&events));
            });
        });
    }

    group.finish();
}

fn bench_json_export(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_export");

    for size in [100, 1000, 5000] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            reset();
            for i in 0..size {
                let name = format!("var_{}", i);
                let x = track_new(&name, i);
                let _r = track_borrow("r", &x);
                track_drop(&name);
            }

            b.iter(|| {
                let events = get_events();
                let graph = build_graph(&events);
                let export = ExportData::new(graph, events.clone());
                black_box(export.to_json().unwrap());
            });
        });
    }

    group.finish();
}

fn bench_concurrent_tracking(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent");

    group.bench_function("4_threads_100_ops", |b| {
        b.iter(|| {
            reset();
            let handles: Vec<_> = (0..4)
                .map(|thread_id| {
                    std::thread::spawn(move || {
                        for i in 0..100 {
                            let name = format!("var_{}_{}", thread_id, i);
                            black_box(track_new(&name, i));
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

fn bench_memory_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory");

    group.bench_function("event_size", |b| {
        b.iter(|| {
            black_box(std::mem::size_of::<Event>());
        });
    });

    group.bench_function("get_events_clone", |b| {
        reset();
        for i in 0..1000 {
            track_new(&format!("var_{}", i), i);
        }

        b.iter(|| {
            black_box(get_events());
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_single_operations,
    bench_bulk_operations,
    bench_graph_building,
    bench_json_export,
    bench_concurrent_tracking,
    bench_memory_overhead
);
criterion_main!(benches);
