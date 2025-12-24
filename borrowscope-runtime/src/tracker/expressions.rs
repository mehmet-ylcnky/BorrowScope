//! Expression tracking: index, field access, calls, operators

use super::TRACKER;

pub fn track_index_access(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] access_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] container: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_index_access(access_id, container, location);
    }
}

/// Track field access
#[inline(always)]
pub fn track_field_access(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] access_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] base: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] field: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_field_access(access_id, base, field, location);
    }
}

/// Track function call
#[inline(always)]
pub fn track_call(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] call_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] fn_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_call(call_id, fn_name, location);
    }
}

/// Track lock acquisition (Mutex/RwLock)
#[inline(always)]
pub fn track_lock(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] lock_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] lock_type: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] var_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_lock(lock_id, lock_type, var_name, location);
    }
}

/// Track unwrap call (Option/Result)
#[inline(always)]
pub fn track_unwrap(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] unwrap_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] method: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] var_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_unwrap(unwrap_id, method, var_name, location);
    }
}

/// Track clone call
#[inline(always)]
pub fn track_clone(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] clone_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] var_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_clone(clone_id, var_name, location);
    }
}

/// Track dereference operation
#[inline(always)]
pub fn track_deref(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] deref_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] var_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_deref(deref_id, var_name, location);
    }
}

pub fn track_closure_create(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] closure_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] capture_mode: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_closure_create(closure_id, capture_mode, location);
    }
}

/// Track struct creation
#[inline(always)]
pub fn track_struct_create(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] struct_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] struct_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_struct_create(struct_id, struct_name, location);
    }
}

/// Track tuple creation
#[inline(always)]
pub fn track_tuple_create(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] tuple_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] arity: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_tuple_create(tuple_id, arity, location);
    }
}

/// Track let-else divergence
#[inline(always)]
pub fn track_let_else(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] let_else_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] pattern: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_let_else(let_else_id, pattern, location);
    }
}

/// Track range expression
#[inline(always)]
pub fn track_range(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] range_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] range_type: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_range(range_id, range_type, location);
    }
}

/// Track binary operation
#[inline(always)]
pub fn track_binary_op(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] op_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] operator: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_binary_op(op_id, operator, location);
    }
}

/// Track array creation
#[inline(always)]
pub fn track_array_create(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] array_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] length: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_array_create(array_id, length, location);
    }
}

/// Track type cast
#[inline(always)]
pub fn track_type_cast(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] cast_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] target_type: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_type_cast(cast_id, target_type, location);
    }
}
