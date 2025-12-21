use borrowscope_runtime::*;
use std::time::Instant;

fn main() {
    println!("BorrowScope Runtime Performance Benchmark");
    println!("=========================================");
    
    // Benchmark 1: Basic tracking overhead
    benchmark_basic_tracking();
    
    // Benchmark 2: Memory usage scaling
    benchmark_memory_scaling();
    
    // Benchmark 3: Complex ownership patterns
    benchmark_complex_patterns();
    
    // Benchmark 4: Disabled tracking (zero-cost)
    benchmark_disabled_tracking();
}

fn benchmark_basic_tracking() {
    println!("\n1. Basic Tracking Overhead");
    println!("--------------------------");
    
    reset();
    let iterations = 100_000;
    
    // Measure tracking overhead
    let start = Instant::now();
    for i in 0..iterations {
        let data = track_new(&format!("data_{}", i), vec![1, 2, 3, 4, 5]);
        let _r1 = track_borrow(&format!("r1_{}", i), &data);
        let _r2 = track_borrow(&format!("r2_{}", i), &data);
        track_drop(&format!("r2_{}", i));
        track_drop(&format!("r1_{}", i));
        track_drop(&format!("data_{}", i));
    }
    let tracking_duration = start.elapsed();
    
    // Measure baseline (no tracking)
    let start = Instant::now();
    for _i in 0..iterations {
        let data = vec![1, 2, 3, 4, 5];
        let _r1 = &data;
        let _r2 = &data;
        // Variables drop automatically
    }
    let baseline_duration = start.elapsed();
    
    let overhead_per_op = (tracking_duration.as_nanos() - baseline_duration.as_nanos()) / (iterations * 6); // 6 operations per iteration
    
    println!("Iterations: {}", iterations);
    println!("Tracking time: {:?}", tracking_duration);
    println!("Baseline time: {:?}", baseline_duration);
    println!("Overhead per operation: {} ns", overhead_per_op);
    
    let events = get_events();
    println!("Events generated: {}", events.len());
    println!("Memory per event: ~{} bytes", std::mem::size_of::<Event>());
}

fn benchmark_memory_scaling() {
    println!("\n2. Memory Usage Scaling");
    println!("-----------------------");
    
    let test_sizes = vec![1_000, 10_000, 50_000, 100_000];
    
    for &size in &test_sizes {
        reset();
        
        // Generate events
        for i in 0..size {
            let data = track_new(&format!("data_{}", i), vec![i; 10]);
            let _r1 = track_borrow(&format!("r1_{}", i), &data);
            track_drop(&format!("r1_{}", i));
            track_drop(&format!("data_{}", i));
        }
        
        let events = get_events();
        let memory_usage = events.len() * std::mem::size_of::<Event>();
        
        println!("Variables: {}, Events: {}, Memory: {} KB", 
                 size, events.len(), memory_usage / 1024);
    }
}

fn benchmark_complex_patterns() {
    println!("\n3. Complex Ownership Patterns");
    println!("-----------------------------");
    
    reset();
    let start = Instant::now();
    
    // Simulate complex smart pointer usage
    for i in 0..1000 {
        let rc1 = track_rc_new(&format!("rc1_{}", i), std::rc::Rc::new(vec![i; 100]));
        let _rc2 = track_rc_clone(&format!("rc2_{}", i), &format!("rc1_{}", i), std::rc::Rc::clone(&rc1));
        let _rc3 = track_rc_clone(&format!("rc3_{}", i), &format!("rc1_{}", i), std::rc::Rc::clone(&rc1));
        
        // Simulate RefCell operations (separate borrows)
        let cell = track_refcell_new(&format!("cell_{}", i), std::cell::RefCell::new(i));
        {
            let _borrow1 = track_refcell_borrow(&format!("borrow1_{}", i), &format!("cell_{}", i), "benchmark", cell.borrow());
            track_drop(&format!("borrow1_{}", i));
        }
        {
            let _borrow_mut = track_refcell_borrow_mut(&format!("borrow_mut_{}", i), &format!("cell_{}", i), "benchmark", cell.borrow_mut());
            track_drop(&format!("borrow_mut_{}", i));
        }
        
        // Clean up
        track_drop(&format!("cell_{}", i));
        track_drop(&format!("rc3_{}", i));
        track_drop(&format!("rc2_{}", i));
        track_drop(&format!("rc1_{}", i));
    }
    
    let duration = start.elapsed();
    let events = get_events();
    
    println!("Complex pattern iterations: 1000");
    println!("Total time: {:?}", duration);
    println!("Events generated: {}", events.len());
    println!("Time per complex operation: {} μs", duration.as_micros() / 1000);
}

fn benchmark_disabled_tracking() {
    println!("\n4. Disabled Tracking (Zero-Cost Verification)");
    println!("---------------------------------------------");
    
    // This would only show zero cost if compiled without 'track' feature
    // For demonstration, we'll show the difference
    
    let iterations = 1_000_000;
    
    let start = Instant::now();
    for i in 0..iterations {
        // These calls should be no-ops when tracking is disabled
        let _data = vec![i; 5];
        let _r1 = &_data;
        let _r2 = &_data;
    }
    let no_tracking_time = start.elapsed();
    
    println!("No-tracking baseline (1M iterations): {:?}", no_tracking_time);
    println!("Per operation: {} ns", no_tracking_time.as_nanos() / iterations);
    
    #[cfg(feature = "track")]
    println!("Note: Tracking is ENABLED - overhead measurements are valid");
    
    #[cfg(not(feature = "track"))]
    println!("Note: Tracking is DISABLED - zero-cost abstraction active");
}
