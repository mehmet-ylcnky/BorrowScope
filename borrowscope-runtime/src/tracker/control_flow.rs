//! Control flow tracking: loops, match, branches, break, continue, return

use super::TRACKER;

pub fn track_loop_enter(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] loop_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] loop_type: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_loop_enter(loop_id, loop_type, location);
    }
}

/// Track loop iteration
#[inline(always)]
pub fn track_loop_iteration(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] loop_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] iteration: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_loop_iteration(loop_id, iteration, location);
    }
}

/// Track loop exit
#[inline(always)]
pub fn track_loop_exit(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] loop_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_loop_exit(loop_id, location);
    }
}

/// Track match expression entry
#[inline(always)]
pub fn track_match_enter(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] match_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_match_enter(match_id, location);
    }
}

/// Track match arm taken
#[inline(always)]
pub fn track_match_arm(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] match_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] arm_index: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] pattern: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_match_arm(match_id, arm_index, pattern, location);
    }
}

/// Track match arm with binding names from static analysis.
#[inline(always)]
pub fn track_match_arm_with_bindings(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] match_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] arm_index: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] pattern: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] bindings: &[&str],
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_match_arm_with_bindings(match_id, arm_index, pattern, location, bindings);
    }
}

/// Track match expression exit
#[inline(always)]
pub fn track_match_exit(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] match_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_match_exit(match_id, location);
    }
}

/// Track branch taken (if/else)
#[inline(always)]
pub fn track_branch(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] branch_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] branch_type: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_branch(branch_id, branch_type, location);
    }
}

/// Track return statement
#[inline(always)]
pub fn track_return(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] return_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] has_value: bool,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_return(return_id, has_value, location);
    }
}

/// Track try/? operator
#[inline(always)]
pub fn track_try(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] try_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_try(try_id, location);
    }
}

/// Track index access
#[inline(always)]
pub fn track_break(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] break_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] label: Option<&str>,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_break(break_id, label, location);
    }
}

/// Track continue statement
#[inline(always)]
pub fn track_continue(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] continue_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] label: Option<&str>,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_continue(continue_id, label, location);
    }
}

/// Track closure creation
#[inline(always)]
pub fn track_region_enter(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] region_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] region_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_region_enter(region_id, region_name, location);
    }
}

/// Track region/scope exit
#[inline(always)]
pub fn track_region_exit(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] region_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_region_exit(region_id, location);
    }
}

// =============================================================================
// Phase 8: Enhanced Tracking Functions
// =============================================================================

/// Track function entry
#[inline(always)]
pub fn track_fn_enter(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] fn_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] fn_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_fn_enter(fn_id, fn_name, location);
    }
}

/// Track function exit
#[inline(always)]
pub fn track_fn_exit(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] fn_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] fn_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_fn_exit(fn_id, fn_name, location);
    }
}

/// Track closure variable capture
#[inline(always)]
pub fn track_closure_capture(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] closure_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] var_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] capture_mode: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_closure_capture(closure_id, var_name, capture_mode, location);
    }
}

