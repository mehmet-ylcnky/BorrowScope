//! Static and const tracking

use super::TRACKER;

pub fn track_static_init<T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] var_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] var_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] type_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] is_mutable: bool,
    value: T,
) -> T {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_static_init(var_name, var_id, type_name, is_mutable);
    }
    value
}

/// Track static variable access (read or write).
///
/// Records a `StaticAccess` event. Use this when reading from or writing to a static variable.
///
/// # Arguments
///
/// * `var_id` - Identifier of the static variable
/// * `var_name` - Name of the static variable
/// * `is_write` - `true` for writes, `false` for reads
/// * `location` - Source location (e.g., "file.rs:42")
#[inline(always)]
pub fn track_static_access(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] var_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] var_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] is_write: bool,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_static_access(var_id, var_name, is_write, location);
    }
}

/// Track const evaluation.
///
/// Records a `ConstEval` event. Use this when a const value is evaluated.
///
/// # Arguments
///
/// * `const_name` - Name of the const
/// * `const_id` - Unique identifier
/// * `type_name` - Type of the const
/// * `location` - Source location
/// * `value` - The const value (returned unchanged)
///
/// # Returns
///
/// The input value, unchanged.
#[inline(always)]
pub fn track_const_eval<T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] const_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] const_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] type_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: T,
) -> T {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_const_eval(const_name, const_id, type_name, location);
    }
    value
}
