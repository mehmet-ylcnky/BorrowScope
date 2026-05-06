//! Comprehensive tests for Milestone 5: Statistics and Metrics.

use borrowscope_graph::stats::*;
use borrowscope_graph::*;

// ═══════════════════════════════════════════════════════════════════════════
// 5.1 Graph statistics
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_statistics_basic() {
    let mut g = OwnershipGraph::new();
    let x = g.add_variable("x", "i32", 0);
    let r1 = g.add_variable("r1", "&i32", 10);
    let r2 = g.add_variable("r2", "&i32", 20);
    let m = g.add_variable("m", "&mut i32", 30);
    g.add_borrow(r1, x, false, 10);
    g.add_borrow(r2, x, false, 20);
    g.add_borrow(m, x, true, 30);
    g.mark_dropped(r1, 40);

    let stats = statistics(&g);
    assert_eq!(stats.total_nodes, 4);
    assert_eq!(stats.variable_nodes, 4);
    assert_eq!(stats.scope_nodes, 0);
    assert_eq!(stats.dropped_variables, 1);
    assert_eq!(stats.alive_variables, 3);
    assert_eq!(stats.total_edges, 3);
    assert_eq!(stats.shared_borrows, 2);
    assert_eq!(stats.mutable_borrows, 1);
}

#[test]
fn test_statistics_derived_metrics() {
    let mut g = OwnershipGraph::new();
    let x = g.add_variable("x", "i32", 0);
    let y = g.add_variable("y", "i32", 10);
    let r = g.add_variable("r", "&i32", 20);
    g.add_borrow(r, x, false, 20);
    g.add_move(x, y, 30);

    let stats = statistics(&g);
    assert_eq!(stats.moves, 1);
    // avg_borrows = 1 borrow / 3 variables
    assert!((stats.avg_borrows_per_variable - 1.0 / 3.0).abs() < 0.01);
    // move_ratio = 1 move / 3 variables
    assert!((stats.move_ratio - 1.0 / 3.0).abs() < 0.01);
    assert_eq!(stats.max_borrows_on_single_variable, 1);
}

#[test]
fn test_statistics_shared_ownership() {
    let mut g = OwnershipGraph::new();
    let rc1 = g.add_variable("rc1", "Rc<i32>", 0);
    let rc2 = g.add_variable("rc2", "Rc<i32>", 10);
    let arc1 = g.add_variable("arc1", "Arc<i32>", 20);
    let arc2 = g.add_variable("arc2", "Arc<i32>", 30);
    g.add_rc_clone(rc2, rc1, 2, 10);
    g.add_arc_clone(arc2, arc1, 2, 30);

    let stats = statistics(&g);
    assert_eq!(stats.rc_clones, 1);
    assert_eq!(stats.arc_clones, 1);
    // shared_ownership_ratio = 2 clones / 4 variables = 0.5
    assert!((stats.shared_ownership_ratio - 0.5).abs() < 0.01);
}

#[test]
fn test_statistics_empty_graph() {
    let g = OwnershipGraph::new();
    let stats = statistics(&g);
    assert_eq!(stats.total_nodes, 0);
    assert_eq!(stats.total_edges, 0);
    assert_eq!(stats.avg_borrows_per_variable, 0.0);
    assert_eq!(stats.move_ratio, 0.0);
    assert_eq!(stats.shared_ownership_ratio, 0.0);
    assert_eq!(stats.max_borrows_on_single_variable, 0);
}

#[test]
fn test_statistics_all_edge_types() {
    let mut g = OwnershipGraph::new();
    let a = g.add_variable("a", "i32", 0);
    let b = g.add_variable("b", "i32", 10);
    let c = g.add_variable("c", "i32", 20);
    let d = g.add_variable("d", "i32", 30);
    let e = g.add_variable("e", "i32", 40);
    let f = g.add_variable("f", "i32", 50);

    g.add_weak_downgrade(b, a, 10);
    g.add_refcell_borrow(c, a, false, 20);
    g.add_lock_acquire(d, a, "mutex", 30);
    g.add_capture(e, a, CaptureMode::ByRef, 40);
    g.add_channel_send(f, a, 50);

    let stats = statistics(&g);
    assert_eq!(stats.weak_downgrades, 1);
    assert_eq!(stats.refcell_borrows, 1);
    assert_eq!(stats.lock_acquires, 1);
    assert_eq!(stats.closure_captures, 1);
    assert_eq!(stats.channel_sends, 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// 5.2 Hotspot detection
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_hotspots_most_borrowed() {
    let mut g = OwnershipGraph::new();
    let data = g.add_variable("data", "Vec<i32>", 0);
    let other = g.add_variable("other", "i32", 0);

    // data gets 5 borrows, other gets 1
    for i in 0..5 {
        let r = g.add_variable(&format!("r{}", i), "&Vec<i32>", (i + 1) * 10);
        g.add_borrow(r, data, false, (i + 1) * 10);
    }
    let r = g.add_variable("ro", "&i32", 60);
    g.add_borrow(r, other, false, 60);

    let spots = hotspots(&g, 1);
    assert_eq!(spots.len(), 1);
    assert_eq!(spots[0].node, data);
    assert_eq!(spots[0].incoming_borrows, 5);
}

#[test]
fn test_hotspots_top_n() {
    let mut g = OwnershipGraph::new();
    let a = g.add_variable("a", "i32", 0);
    let b = g.add_variable("b", "i32", 0);
    let c = g.add_variable("c", "i32", 0);

    // a: 3 borrows, b: 2 borrows, c: 1 borrow
    for i in 0..3 {
        let r = g.add_variable(&format!("ra{}", i), "&i32", 10);
        g.add_borrow(r, a, false, 10);
    }
    for i in 0..2 {
        let r = g.add_variable(&format!("rb{}", i), "&i32", 10);
        g.add_borrow(r, b, false, 10);
    }
    let r = g.add_variable("rc0", "&i32", 10);
    g.add_borrow(r, c, false, 10);

    let spots = hotspots(&g, 2);
    assert_eq!(spots.len(), 2);
    assert_eq!(spots[0].node, a); // most edges
    assert_eq!(spots[1].node, b); // second most
}

#[test]
fn test_hotspots_score_normalized() {
    let mut g = OwnershipGraph::new();
    let a = g.add_variable("a", "i32", 0);
    let b = g.add_variable("b", "i32", 0);
    for i in 0..10 {
        let r = g.add_variable(&format!("r{}", i), "&i32", 10);
        g.add_borrow(r, a, false, 10);
    }
    let r = g.add_variable("rb", "&i32", 10);
    g.add_borrow(r, b, false, 10);

    let spots = hotspots(&g, 10);
    assert!((spots[0].score - 1.0).abs() < 0.01); // top has score 1.0
    assert!(spots.last().unwrap().score >= 0.0);
    assert!(spots.last().unwrap().score <= 1.0);
}

#[test]
fn test_heavily_borrowed() {
    let mut g = OwnershipGraph::new();
    let a = g.add_variable("a", "i32", 0);
    let b = g.add_variable("b", "i32", 0);
    for i in 0..5 {
        let r = g.add_variable(&format!("ra{}", i), "&i32", 10);
        g.add_borrow(r, a, false, 10);
    }
    let r = g.add_variable("rb", "&i32", 10);
    g.add_borrow(r, b, false, 10);

    let heavy = heavily_borrowed(&g, 3);
    assert_eq!(heavy.len(), 1);
    assert_eq!(heavy[0].node, a);
}

#[test]
fn test_most_transferred() {
    let mut g = OwnershipGraph::new();
    let a = g.add_variable("a", "String", 0);
    let b = g.add_variable("b", "String", 10);
    let c = g.add_variable("c", "String", 20);
    let d = g.add_variable("d", "i32", 0); // no moves
    g.add_move(a, b, 10);
    g.add_move(b, c, 20);

    let transferred = most_transferred(&g, 5);
    assert!(!transferred.is_empty());
    // a, b, c all have moves; d does not
    assert!(transferred.iter().all(|h| h.node != d));
}

// ═══════════════════════════════════════════════════════════════════════════
// 5.3 Borrow frequency analysis
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_borrow_frequency_basic() {
    let mut g = OwnershipGraph::new();
    let x = g.add_variable("x", "i32", 0);
    for i in 0..10u64 {
        let r = g.add_variable(&format!("r{}", i), "&i32", i * 100);
        let eid = g.add_borrow(r, x, false, i * 100);
        g.end_edge(eid, i * 100 + 50);
    }

    let freq = borrow_frequency(&g);
    assert_eq!(freq.total_borrows, 10);
    assert_eq!(freq.shared_borrows, 10);
    assert_eq!(freq.mutable_borrows, 0);
    assert_eq!(freq.min_duration, 50);
    assert_eq!(freq.max_duration, 50);
    assert_eq!(freq.median_duration, 50);
    assert!((freq.avg_duration - 50.0).abs() < 0.01);
}

#[test]
fn test_borrow_frequency_max_concurrent() {
    let mut g = OwnershipGraph::new();
    let x = g.add_variable("x", "i32", 0);
    // 3 overlapping borrows
    let r1 = g.add_variable("r1", "&i32", 10);
    let r2 = g.add_variable("r2", "&i32", 20);
    let r3 = g.add_variable("r3", "&i32", 30);
    let e1 = g.add_borrow(r1, x, false, 10);
    let e2 = g.add_borrow(r2, x, false, 20);
    let e3 = g.add_borrow(r3, x, false, 30);
    g.end_edge(e1, 100);
    g.end_edge(e2, 100);
    g.end_edge(e3, 100);

    let freq = borrow_frequency(&g);
    assert_eq!(freq.max_concurrent, 3);
}

#[test]
fn test_borrow_frequency_burst_detection() {
    let mut g = OwnershipGraph::new();
    let x = g.add_variable("x", "i32", 0);
    // Cluster of 5 borrows at t=10..14, then nothing until t=1000
    for i in 0..5u64 {
        let r = g.add_variable(&format!("r{}", i), "&i32", 10 + i);
        let eid = g.add_borrow(r, x, false, 10 + i);
        g.end_edge(eid, 10 + i + 1);
    }
    // One isolated borrow far away
    let r = g.add_variable("rlate", "&i32", 1000);
    let eid = g.add_borrow(r, x, false, 1000);
    g.end_edge(eid, 1001);

    let freq = borrow_frequency(&g);
    assert_eq!(freq.total_borrows, 6);
    // Should detect the burst at t=10..14
    assert!(!freq.bursts.is_empty());
    assert!(freq.bursts[0].borrow_count >= 3);
}

#[test]
fn test_borrow_frequency_empty() {
    let g = OwnershipGraph::new();
    let freq = borrow_frequency(&g);
    assert_eq!(freq.total_borrows, 0);
    assert_eq!(freq.frequency, 0.0);
    assert_eq!(freq.max_concurrent, 0);
}

#[test]
fn test_borrow_frequency_of_specific_var() {
    let mut g = OwnershipGraph::new();
    let x = g.add_variable("x", "i32", 0);
    let y = g.add_variable("y", "i32", 0);
    // x gets 3 borrows, y gets 1
    for i in 0..3 {
        let r = g.add_variable(&format!("rx{}", i), "&i32", 10);
        g.add_borrow(r, x, false, 10);
    }
    let r = g.add_variable("ry", "&i32", 10);
    g.add_borrow(r, y, false, 10);

    let freq_x = borrow_frequency_of(&g, x);
    assert_eq!(freq_x.total_borrows, 3);

    let freq_y = borrow_frequency_of(&g, y);
    assert_eq!(freq_y.total_borrows, 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// 5.4 Scope depth distribution
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_depth_distribution_flat() {
    let mut g = OwnershipGraph::new();
    // All at default depth 0
    g.add_variable("a", "i32", 0);
    g.add_variable("b", "i32", 10);
    g.add_variable("c", "i32", 20);

    let dist = depth_distribution(&g);
    assert_eq!(dist.max_depth, 0);
    assert_eq!(dist.histogram, vec![(0, 3)]);
    assert!((dist.avg_depth - 0.0).abs() < 0.01);
}

#[test]
fn test_depth_distribution_varied() {
    let mut g = OwnershipGraph::new();
    // Manually set scope_depth by using the internal structure
    // Since add_variable sets scope_depth=0, we test with that limitation
    // The depth_distribution function reads scope_depth from VariableNode
    g.add_variable("a", "i32", 0); // depth 0
    g.add_variable("b", "i32", 10); // depth 0

    let dist = depth_distribution(&g);
    assert_eq!(dist.max_depth, 0);
    assert_eq!(dist.deepest_variables.len(), 2);
}

#[test]
fn test_scope_depth_query() {
    let mut g = OwnershipGraph::new();
    let x = g.add_variable("x", "i32", 0);
    assert_eq!(scope_depth(&g, x), 0);
}

#[test]
fn test_depth_distribution_empty() {
    let g = OwnershipGraph::new();
    let dist = depth_distribution(&g);
    assert_eq!(dist.max_depth, 0);
    assert!(dist.histogram.is_empty());
    assert_eq!(dist.avg_depth, 0.0);
}

// ═══════════════════════════════════════════════════════════════════════════
// 5.5 Smart pointer usage patterns
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_smart_pointer_report_rc() {
    let mut g = OwnershipGraph::new();
    let rc1 = g.add_variable("rc1", "Rc<i32>", 0);
    let rc2 = g.add_variable("rc2", "Rc<i32>", 10);
    let rc3 = g.add_variable("rc3", "Rc<i32>", 20);
    g.add_rc_clone(rc2, rc1, 2, 10);
    g.add_rc_clone(rc3, rc1, 3, 20);
    g.mark_dropped(rc2, 50);
    g.mark_dropped(rc3, 60);
    g.mark_dropped(rc1, 70);

    let report = smart_pointer_report(&g);
    assert_eq!(report.rc_families.len(), 1);
    assert_eq!(report.rc_families[0].origin, rc1);
    assert_eq!(report.rc_families[0].clone_count, 2);
    assert!(!report.rc_families[0].is_leaked);
}

#[test]
fn test_smart_pointer_report_leaked_rc() {
    let mut g = OwnershipGraph::new();
    let rc1 = g.add_variable("rc1", "Rc<i32>", 0);
    let rc2 = g.add_variable("rc2", "Rc<i32>", 10);
    g.add_rc_clone(rc2, rc1, 2, 10);
    // Not dropped

    let report = smart_pointer_report(&g);
    assert_eq!(report.rc_families.len(), 1);
    assert!(report.rc_families[0].is_leaked);
}

#[test]
fn test_smart_pointer_report_refcell() {
    let mut g = OwnershipGraph::new();
    let cell = g.add_variable("cell", "RefCell<i32>", 0);
    let g1 = g.add_variable("g1", "Ref<i32>", 10);
    let g2 = g.add_variable("g2", "Ref<i32>", 20);
    let g3 = g.add_variable("g3", "RefMut<i32>", 30);
    g.add_refcell_borrow(g1, cell, false, 10);
    g.add_refcell_borrow(g2, cell, false, 20);
    g.add_refcell_borrow(g3, cell, true, 30);

    let report = smart_pointer_report(&g);
    assert_eq!(report.refcell_usage.len(), 1);
    assert_eq!(report.refcell_usage[0].immutable_borrows, 2);
    assert_eq!(report.refcell_usage[0].mutable_borrows, 1);
}

#[test]
fn test_smart_pointer_report_mutex() {
    let mut g = OwnershipGraph::new();
    let mtx = g.add_variable("mtx", "Mutex<i32>", 0);
    let g1 = g.add_variable("g1", "MutexGuard<i32>", 10);
    let g2 = g.add_variable("g2", "MutexGuard<i32>", 50);
    let e1 = g.add_lock_acquire(g1, mtx, "mutex", 10);
    g.end_edge(e1, 30); // hold time = 20
    let e2 = g.add_lock_acquire(g2, mtx, "mutex", 50);
    g.end_edge(e2, 90); // hold time = 40

    let report = smart_pointer_report(&g);
    assert_eq!(report.mutex_usage.len(), 1);
    assert_eq!(report.mutex_usage[0].lock_count, 2);
    assert!((report.mutex_usage[0].avg_hold_time - 30.0).abs() < 0.01);
    assert_eq!(report.mutex_usage[0].max_hold_time, 40);
}

#[test]
fn test_smart_pointer_report_empty() {
    let g = OwnershipGraph::new();
    let report = smart_pointer_report(&g);
    assert!(report.rc_families.is_empty());
    assert!(report.arc_families.is_empty());
    assert!(report.refcell_usage.is_empty());
    assert!(report.mutex_usage.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// Performance
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_performance_statistics() {
    let mut g = OwnershipGraph::new();
    let owner = g.add_variable("data", "Vec<i32>", 0);
    for i in 0..10_000u64 {
        let r = g.add_variable(&format!("r{}", i), "&Vec<i32>", i);
        g.add_borrow(r, owner, false, i);
    }

    let start = std::time::Instant::now();
    let stats = statistics(&g);
    let elapsed = start.elapsed();

    assert_eq!(stats.shared_borrows, 10_000);
    assert!(
        elapsed.as_millis() < 50,
        "Statistics took too long: {:?}",
        elapsed
    );
}
