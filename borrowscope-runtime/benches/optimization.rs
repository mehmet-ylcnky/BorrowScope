use borrowscope_runtime::{reset, track_borrow, track_drop, track_drop_batch, track_new};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_track_new(c: &mut Criterion) {
    c.bench_function("track_new", |b| {
        b.iter(|| {
            reset();
            let x = track_new("x", black_box(42));
            black_box(x);
        });
    });
}

fn bench_track_borrow(c: &mut Criterion) {
    c.bench_function("track_borrow", |b| {
        b.iter(|| {
            reset();
            let x = 42;
            let r = track_borrow("r", black_box(&x));
            black_box(r);
        });
    });
}

fn bench_track_drop(c: &mut Criterion) {
    c.bench_function("track_drop", |b| {
        b.iter(|| {
            reset();
            track_drop(black_box("x"));
        });
    });
}

fn bench_track_drop_batch(c: &mut Criterion) {
    c.bench_function("track_drop_batch_3", |b| {
        b.iter(|| {
            reset();
            track_drop_batch(black_box(&["x", "y", "z"]));
        });
    });
}

fn bench_track_drop_individual(c: &mut Criterion) {
    c.bench_function("track_drop_individual_3", |b| {
        b.iter(|| {
            reset();
            track_drop(black_box("x"));
            track_drop(black_box("y"));
            track_drop(black_box("z"));
        });
    });
}

fn bench_baseline_no_tracking(c: &mut Criterion) {
    c.bench_function("baseline_no_tracking", |b| {
        b.iter(|| {
            let x = black_box(42);
            black_box(x);
        });
    });
}

fn bench_with_tracking_full(c: &mut Criterion) {
    c.bench_function("full_tracking_scenario", |b| {
        b.iter(|| {
            reset();
            let x = track_new("x", black_box(42));
            let r = track_borrow("r", black_box(&x));
            black_box(r);
            track_drop("r");
            track_drop("x");
        });
    });
}

criterion_group!(
    benches,
    bench_track_new,
    bench_track_borrow,
    bench_track_drop,
    bench_track_drop_batch,
    bench_track_drop_individual,
    bench_baseline_no_tracking,
    bench_with_tracking_full
);
criterion_main!(benches);
