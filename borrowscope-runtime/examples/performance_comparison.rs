use borrowscope_runtime::*;
use std::time::Instant;

fn main() {
    println!("=== BorrowScope Performance Comparison ===\n");

    compare_baseline_vs_tracked();
    compare_batch_vs_individual();
    compare_smart_pointers();
    compare_concurrent_tracking();
}

fn compare_baseline_vs_tracked() {
    println!("=== Baseline vs Tracked Operations ===\n");

    const ITERATIONS: usize = 100_000;

    // Baseline: no tracking
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let _x = i;
    }
    let baseline_duration = start.elapsed();

    // With tracking
    reset();
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let _x = track_new(&format!("var_{}", i), i);
    }
    let tracked_duration = start.elapsed();

    let overhead = tracked_duration.saturating_sub(baseline_duration);
    let overhead_per_op = overhead / ITERATIONS as u32;
    let overhead_pct = if baseline_duration.as_nanos() > 0 {
        (overhead.as_nanos() as f64 / baseline_duration.as_nanos() as f64) * 100.0
    } else {
        0.0
    };

    println!("  Iterations: {}", ITERATIONS);
    println!("  Baseline time: {:?}", baseline_duration);
    println!("  Tracked time: {:?}", tracked_duration);
    println!("  Overhead: {:?}", overhead);
    println!("  Overhead per op: {:?}", overhead_per_op);
    println!("  Overhead %: {:.2}%", overhead_pct);
    println!();
}

fn compare_batch_vs_individual() {
    println!("=== Batch vs Individual Drop Operations ===\n");

    const COUNT: usize = 1000;

    // Individual drops
    reset();
    let start = Instant::now();
    for i in 0..COUNT {
        track_drop(&format!("var_{}", i));
    }
    let individual_duration = start.elapsed();

    // Batch drops
    reset();
    let names: Vec<String> = (0..COUNT).map(|i| format!("var_{}", i)).collect();
    let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();

    let start = Instant::now();
    track_drop_batch(&name_refs);
    let batch_duration = start.elapsed();

    let speedup = individual_duration.as_nanos() as f64 / batch_duration.as_nanos() as f64;

    println!("  Operations: {}", COUNT);
    println!("  Individual drops: {:?}", individual_duration);
    println!("  Batch drop: {:?}", batch_duration);
    println!("  Speedup: {:.2}x", speedup);
    println!();
}

fn compare_smart_pointers() {
    println!("=== Smart Pointer Tracking Overhead ===\n");

    const ITERATIONS: usize = 10_000;

    // Rc baseline
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let _rc = std::rc::Rc::new(i);
    }
    let rc_baseline = start.elapsed();

    // Rc tracked
    reset();
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let rc = std::rc::Rc::new(i);
        let _tracked = track_rc_new(&format!("rc_{}", i), rc);
    }
    let rc_tracked = start.elapsed();

    // Arc baseline
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let _arc = std::sync::Arc::new(i);
    }
    let arc_baseline = start.elapsed();

    // Arc tracked
    reset();
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let arc = std::sync::Arc::new(i);
        let _tracked = track_arc_new(&format!("arc_{}", i), arc);
    }
    let arc_tracked = start.elapsed();

    // RefCell baseline
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let _cell = std::cell::RefCell::new(i);
    }
    let refcell_baseline = start.elapsed();

    // RefCell tracked
    reset();
    let start = Instant::now();
    for i in 0..ITERATIONS {
        let cell = std::cell::RefCell::new(i);
        let _tracked = track_refcell_new(&format!("cell_{}", i), cell);
    }
    let refcell_tracked = start.elapsed();

    println!("  Iterations: {}", ITERATIONS);
    println!();
    println!("  Rc::new:");
    println!("    Baseline: {:?}", rc_baseline);
    println!("    Tracked: {:?}", rc_tracked);
    println!("    Overhead: {:?}", rc_tracked.saturating_sub(rc_baseline));
    println!();
    println!("  Arc::new:");
    println!("    Baseline: {:?}", arc_baseline);
    println!("    Tracked: {:?}", arc_tracked);
    println!(
        "    Overhead: {:?}",
        arc_tracked.saturating_sub(arc_baseline)
    );
    println!();
    println!("  RefCell::new:");
    println!("    Baseline: {:?}", refcell_baseline);
    println!("    Tracked: {:?}", refcell_tracked);
    println!(
        "    Overhead: {:?}",
        refcell_tracked.saturating_sub(refcell_baseline)
    );
    println!();
}

fn compare_concurrent_tracking() {
    println!("=== Concurrent Tracking Performance ===\n");

    const OPS_PER_THREAD: usize = 1000;

    // Single thread
    reset();
    let start = Instant::now();
    for i in 0..OPS_PER_THREAD {
        let _x = track_new(&format!("var_{}", i), i);
    }
    let single_thread = start.elapsed();

    // 2 threads
    reset();
    let start = Instant::now();
    let handles: Vec<_> = (0..2)
        .map(|thread_id| {
            std::thread::spawn(move || {
                for i in 0..OPS_PER_THREAD {
                    let _x = track_new(&format!("var_{}_{}", thread_id, i), i);
                }
            })
        })
        .collect();
    for handle in handles {
        handle.join().unwrap();
    }
    let two_threads = start.elapsed();

    // 4 threads
    reset();
    let start = Instant::now();
    let handles: Vec<_> = (0..4)
        .map(|thread_id| {
            std::thread::spawn(move || {
                for i in 0..OPS_PER_THREAD {
                    let _x = track_new(&format!("var_{}_{}", thread_id, i), i);
                }
            })
        })
        .collect();
    for handle in handles {
        handle.join().unwrap();
    }
    let four_threads = start.elapsed();

    println!("  Operations per thread: {}", OPS_PER_THREAD);
    println!();
    println!("  1 thread ({} ops): {:?}", OPS_PER_THREAD, single_thread);
    println!(
        "  2 threads ({} ops): {:?}",
        OPS_PER_THREAD * 2,
        two_threads
    );
    println!(
        "  4 threads ({} ops): {:?}",
        OPS_PER_THREAD * 4,
        four_threads
    );
    println!();
    println!("  Scaling efficiency:");
    println!(
        "    2 threads: {:.2}x (ideal: 2.0x)",
        single_thread.as_nanos() as f64 / two_threads.as_nanos() as f64
    );
    println!(
        "    4 threads: {:.2}x (ideal: 4.0x)",
        single_thread.as_nanos() as f64 / four_threads.as_nanos() as f64
    );
    println!();
}
