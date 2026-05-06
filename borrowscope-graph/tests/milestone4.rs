//! Comprehensive tests for Milestone 4: Temporal Queries and Lifetime Analysis.

use borrowscope_graph::*;
use borrowscope_graph::temporal::*;

// ═══════════════════════════════════════════════════════════════════════════
// 4.1 Lifetime spans
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_lifetime_span_duration() {
    let mut g = OwnershipGraph::new();
    let x = g.add_variable("x", "i32", 10);
    g.mark_dropped(x, 50);

    let span = lifetime_of(&g, x).unwrap();
    assert_eq!(span.duration(), Some(40));
}

#[test]
fn test_lifetime_span_never_dropped() {
    let mut g = OwnershipGraph::new();
    let x = g.add_variable("x", "i32", 10);

    let span = lifetime_of(&g, x).unwrap();
    assert_eq!(span.end, None);
    assert_eq!(span.duration(), None);
}

#[test]
fn test_lifetime_is_alive_at_boundaries() {
    let mut g = OwnershipGraph::new();
    let x = g.add_variable("x", "i32", 10);
    g.mark_dropped(x, 50);

    let span = lifetime_of(&g, x).unwrap();
    assert!(!span.is_alive_at(9));   // before start
    assert!(span.is_alive_at(10));   // at start
    assert!(span.is_alive_at(49));   // just before end
    assert!(!span.is_alive_at(50));  // at end (half-open)
}

#[test]
fn test_lifetime_open_ended_always_alive() {
    let mut g = OwnershipGraph::new();
    let x = g.add_variable("x", "i32", 10);

    let span = lifetime_of(&g, x).unwrap();
    assert!(span.is_alive_at(10));
    assert!(span.is_alive_at(u64::MAX - 1));
}

#[test]
fn test_all_lifetimes() {
    let mut g = OwnershipGraph::new();
    g.add_variable("a", "i32", 0);
    g.add_variable("b", "i32", 10);
    g.add_variable("c", "i32", 20);

    let spans = all_lifetimes(&g);
    assert_eq!(spans.len(), 3);
}

#[test]
fn test_lifetime_zero_duration() {
    let mut g = OwnershipGraph::new();
    let x = g.add_variable("x", "i32", 10);
    g.mark_dropped(x, 10); // created and dropped at same timestamp

    let span = lifetime_of(&g, x).unwrap();
    assert_eq!(span.duration(), Some(0));
    assert!(!span.is_alive_at(10)); // not alive even at creation (half-open: [10, 10) is empty)
}

// ═══════════════════════════════════════════════════════════════════════════
// 4.2 Overlapping lifetimes
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_overlapping_lifetimes_detected() {
    let mut g = OwnershipGraph::new();
    let a = g.add_variable("a", "i32", 0);
    g.mark_dropped(a, 100);
    let b = g.add_variable("b", "i32", 50);
    g.mark_dropped(b, 150);

    assert!(lifetimes_overlap(&g, a, b));
    let pairs = overlapping_lifetimes(&g);
    assert_eq!(pairs.len(), 1);
}

#[test]
fn test_non_overlapping_lifetimes() {
    let mut g = OwnershipGraph::new();
    let a = g.add_variable("a", "i32", 0);
    g.mark_dropped(a, 50);
    let b = g.add_variable("b", "i32", 60);
    g.mark_dropped(b, 100);

    assert!(!lifetimes_overlap(&g, a, b));
    let pairs = overlapping_lifetimes(&g);
    assert!(pairs.is_empty());
}

#[test]
fn test_open_ended_overlaps_with_everything_after() {
    let mut g = OwnershipGraph::new();
    let a = g.add_variable("a", "i32", 0); // never dropped
    let b = g.add_variable("b", "i32", 50);
    g.mark_dropped(b, 100);

    assert!(lifetimes_overlap(&g, a, b));
}

#[test]
fn test_contemporaries() {
    let mut g = OwnershipGraph::new();
    let a = g.add_variable("a", "i32", 0);
    g.mark_dropped(a, 100);
    let b = g.add_variable("b", "i32", 50);
    g.mark_dropped(b, 150);
    let c = g.add_variable("c", "i32", 200);
    g.mark_dropped(c, 300);

    let contemps = contemporaries(&g, a);
    assert_eq!(contemps.len(), 1); // only b overlaps with a
    assert!(contemps.contains(&b));
}

// ═══════════════════════════════════════════════════════════════════════════
// 4.3 Active variables at timestamp
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_alive_at_empty_before_events() {
    let mut g = OwnershipGraph::new();
    g.add_variable("x", "i32", 10);

    assert!(alive_at(&g, 0).is_empty());
}

#[test]
fn test_alive_at_includes_created() {
    let mut g = OwnershipGraph::new();
    let x = g.add_variable("x", "i32", 10);

    let alive = alive_at(&g, 15);
    assert_eq!(alive.len(), 1);
    assert_eq!(alive[0], x);
}

#[test]
fn test_alive_at_excludes_dropped() {
    let mut g = OwnershipGraph::new();
    let x = g.add_variable("x", "i32", 10);
    g.mark_dropped(x, 50);

    assert!(alive_at(&g, 55).is_empty());
}

#[test]
fn test_snapshot_at() {
    let mut g = OwnershipGraph::new();
    let x = g.add_variable("x", "i32", 0);
    let r = g.add_variable("r", "&i32", 10);
    g.add_borrow(r, x, false, 10);

    let snap = snapshot_at(&g, 20);
    assert_eq!(snap.alive_variables.len(), 2);
    assert_eq!(snap.active_borrows.len(), 1);
    assert_eq!(snap.active_locks.len(), 0);
}

#[test]
fn test_alive_during_entire_interval() {
    let mut g = OwnershipGraph::new();
    let a = g.add_variable("a", "i32", 0);
    g.mark_dropped(a, 100);
    let b = g.add_variable("b", "i32", 30);
    g.mark_dropped(b, 60);

    // alive_during [20, 80): only a is alive for the entire interval
    let alive = alive_during(&g, 20, 80);
    assert_eq!(alive.len(), 1);
    assert_eq!(alive[0], a);
}

#[test]
fn test_alive_during_none_qualify() {
    let mut g = OwnershipGraph::new();
    let a = g.add_variable("a", "i32", 0);
    g.mark_dropped(a, 50);
    let b = g.add_variable("b", "i32", 60);
    g.mark_dropped(b, 100);

    // No variable is alive for the entire [40, 70) interval
    let alive = alive_during(&g, 40, 70);
    assert!(alive.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// 4.4 Borrow scope computation
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_borrow_scope_with_end() {
    let mut g = OwnershipGraph::new();
    let x = g.add_variable("x", "i32", 0);
    let r = g.add_variable("r", "&i32", 10);
    g.mark_dropped(r, 40);
    let eid = g.add_borrow(r, x, false, 10);
    g.end_edge(eid, 30);

    let scope = borrow_scope_of(&g, eid).unwrap();
    assert_eq!(scope.start, 10);
    assert_eq!(scope.effective_end, 30); // edge ended at 30
    assert_eq!(scope.drop_time, Some(40));
    assert!(!scope.mutable);
}

#[test]
fn test_borrow_scope_falls_back_to_drop() {
    let mut g = OwnershipGraph::new();
    let x = g.add_variable("x", "i32", 0);
    let r = g.add_variable("r", "&i32", 10);
    g.mark_dropped(r, 50);
    let eid = g.add_borrow(r, x, false, 10);
    // Edge never explicitly ended

    let scope = borrow_scope_of(&g, eid).unwrap();
    assert_eq!(scope.start, 10);
    assert_eq!(scope.effective_end, 50); // falls back to drop time
}

#[test]
fn test_borrow_scope_open_ended() {
    let mut g = OwnershipGraph::new();
    let x = g.add_variable("x", "i32", 0);
    let r = g.add_variable("r", "&i32", 10);
    let eid = g.add_borrow(r, x, false, 10);
    // Neither edge ended nor borrower dropped

    let scope = borrow_scope_of(&g, eid).unwrap();
    assert_eq!(scope.effective_end, u64::MAX);
}

#[test]
fn test_borrow_scopes_all() {
    let mut g = OwnershipGraph::new();
    let x = g.add_variable("x", "i32", 0);
    let r1 = g.add_variable("r1", "&i32", 10);
    let r2 = g.add_variable("r2", "&mut i32", 20);
    g.add_borrow(r1, x, false, 10);
    g.add_borrow(r2, x, true, 20);

    let scopes = borrow_scopes(&g);
    assert_eq!(scopes.len(), 2);
    assert!(!scopes[0].mutable);
    assert!(scopes[1].mutable);
}

// ═══════════════════════════════════════════════════════════════════════════
// 4.5 Ownership transfer timeline
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_ownership_timeline_no_moves() {
    let mut g = OwnershipGraph::new();
    let x = g.add_variable("x", "i32", 0);

    let timeline = ownership_timeline(&g, x);
    assert_eq!(timeline.origin, x);
    assert!(timeline.transfers.is_empty());
    assert_eq!(timeline.current_owner, x);
}

#[test]
fn test_ownership_timeline_single_move() {
    let mut g = OwnershipGraph::new();
    let a = g.add_variable("a", "String", 0);
    let b = g.add_variable("b", "String", 20);
    g.add_move(a, b, 20);

    let timeline = ownership_timeline(&g, a);
    assert_eq!(timeline.origin, a);
    assert_eq!(timeline.transfers.len(), 1);
    assert_eq!(timeline.transfers[0].from, a);
    assert_eq!(timeline.transfers[0].to, b);
    assert_eq!(timeline.transfers[0].timestamp, 20);
    assert_eq!(timeline.current_owner, b);
}

#[test]
fn test_ownership_timeline_chain() {
    let mut g = OwnershipGraph::new();
    let a = g.add_variable("a", "String", 0);
    let b = g.add_variable("b", "String", 20);
    let c = g.add_variable("c", "String", 40);
    g.add_move(a, b, 20);
    g.add_move(b, c, 40);

    let timeline = ownership_timeline(&g, a);
    assert_eq!(timeline.transfers.len(), 2);
    assert_eq!(timeline.current_owner, c);
}

#[test]
fn test_find_origin() {
    let mut g = OwnershipGraph::new();
    let a = g.add_variable("a", "String", 0);
    let b = g.add_variable("b", "String", 20);
    let c = g.add_variable("c", "String", 40);
    g.add_move(a, b, 20);
    g.add_move(b, c, 40);

    assert_eq!(find_origin(&g, c), a);
    assert_eq!(find_origin(&g, b), a);
    assert_eq!(find_origin(&g, a), a);
}

#[test]
fn test_find_current_owner() {
    let mut g = OwnershipGraph::new();
    let a = g.add_variable("a", "String", 0);
    let b = g.add_variable("b", "String", 20);
    let c = g.add_variable("c", "String", 40);
    g.add_move(a, b, 20);
    g.add_move(b, c, 40);

    assert_eq!(find_current_owner(&g, a), c);
    assert_eq!(find_current_owner(&g, b), c);
    assert_eq!(find_current_owner(&g, c), c);
}

// ═══════════════════════════════════════════════════════════════════════════
// 4.6 Reference count history
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_ref_count_history_basic() {
    let mut g = OwnershipGraph::new();
    let rc1 = g.add_variable("rc1", "Rc<i32>", 0);
    let rc2 = g.add_variable("rc2", "Rc<i32>", 10);
    g.add_rc_clone(rc2, rc1, 2, 10);
    g.mark_dropped(rc2, 50);
    g.mark_dropped(rc1, 60);

    let history = ref_count_history(&g, rc1);
    assert_eq!(history.origin, rc1);
    assert_eq!(history.peak_count, 2);
    assert_eq!(history.final_count, 0);
    assert!(!history.is_leaked);
}

#[test]
fn test_ref_count_history_leaked() {
    let mut g = OwnershipGraph::new();
    let rc1 = g.add_variable("rc1", "Rc<i32>", 0);
    let rc2 = g.add_variable("rc2", "Rc<i32>", 10);
    g.add_rc_clone(rc2, rc1, 2, 10);
    // Neither dropped - leaked!

    let history = ref_count_history(&g, rc1);
    assert!(history.is_leaked);
    assert!(history.final_count > 0);
}

#[test]
fn test_ref_count_at_timestamp() {
    let mut g = OwnershipGraph::new();
    let rc1 = g.add_variable("rc1", "Rc<i32>", 0);
    let rc2 = g.add_variable("rc2", "Rc<i32>", 10);
    let rc3 = g.add_variable("rc3", "Rc<i32>", 20);
    g.add_rc_clone(rc2, rc1, 2, 10);
    g.add_rc_clone(rc3, rc1, 3, 20);
    g.mark_dropped(rc2, 50);
    g.mark_dropped(rc3, 60);
    g.mark_dropped(rc1, 70);

    assert_eq!(ref_count_at(&g, rc1, 5), 1);   // just rc1
    assert_eq!(ref_count_at(&g, rc1, 15), 2);  // rc1 + rc2
    assert_eq!(ref_count_at(&g, rc1, 25), 3);  // rc1 + rc2 + rc3
    assert_eq!(ref_count_at(&g, rc1, 55), 2);  // rc2 dropped
    assert_eq!(ref_count_at(&g, rc1, 65), 1);  // rc3 dropped
    assert_eq!(ref_count_at(&g, rc1, 75), 0);  // rc1 dropped
}

#[test]
fn test_ref_count_peak() {
    let mut g = OwnershipGraph::new();
    let rc1 = g.add_variable("rc1", "Rc<i32>", 0);
    let rc2 = g.add_variable("rc2", "Rc<i32>", 10);
    let rc3 = g.add_variable("rc3", "Rc<i32>", 20);
    let rc4 = g.add_variable("rc4", "Rc<i32>", 30);
    g.add_rc_clone(rc2, rc1, 2, 10);
    g.add_rc_clone(rc3, rc1, 3, 20);
    g.add_rc_clone(rc4, rc1, 4, 30);
    g.mark_dropped(rc2, 40);
    g.mark_dropped(rc3, 50);
    g.mark_dropped(rc4, 60);
    g.mark_dropped(rc1, 70);

    let history = ref_count_history(&g, rc1);
    assert_eq!(history.peak_count, 4);
}

#[test]
fn test_find_leaked_refs() {
    let mut g = OwnershipGraph::new();
    let rc1 = g.add_variable("rc1", "Rc<i32>", 0);
    let rc2 = g.add_variable("rc2", "Rc<i32>", 10);
    g.add_rc_clone(rc2, rc1, 2, 10);
    // Not dropped - leaked

    let leaked = find_leaked_refs(&g);
    assert!(!leaked.is_empty());
    assert!(leaked.contains(&rc1));
}

#[test]
fn test_find_leaked_refs_none_when_all_dropped() {
    let mut g = OwnershipGraph::new();
    let rc1 = g.add_variable("rc1", "Rc<i32>", 0);
    let rc2 = g.add_variable("rc2", "Rc<i32>", 10);
    g.add_rc_clone(rc2, rc1, 2, 10);
    g.mark_dropped(rc2, 50);
    g.mark_dropped(rc1, 60);

    let leaked = find_leaked_refs(&g);
    assert!(leaked.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// Edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_empty_graph_temporal() {
    let g = OwnershipGraph::new();
    assert!(all_lifetimes(&g).is_empty());
    assert!(overlapping_lifetimes(&g).is_empty());
    assert!(alive_at(&g, 0).is_empty());
    assert!(borrow_scopes(&g).is_empty());
    assert!(find_leaked_refs(&g).is_empty());
}

#[test]
fn test_lifetime_of_nonexistent_node() {
    let g = OwnershipGraph::new();
    assert!(lifetime_of(&g, NodeId(999)).is_none());
}
