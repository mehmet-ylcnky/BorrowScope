//! Sampling functions for probabilistic tracking

use super::TRACKER;
use std::sync::atomic::Ordering;

pub fn should_sample(rate: f64) -> bool {
    use std::sync::atomic::AtomicU64;

    // Use a simple but fast PRNG (xorshift64)
    static SEED: AtomicU64 = AtomicU64::new(0x853c49e6748fea9b);

    // xorshift64 step
    let mut x = SEED.load(Ordering::Relaxed);
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    SEED.store(x, Ordering::Relaxed);

    // Convert to 0.0-1.0 range and compare
    (x as f64 / u64::MAX as f64) < rate
}

/// Track variable creation with sampling.
/// Only records the event if the sample check passes.
/// Returns the value unchanged regardless of sampling.
///
/// # Arguments
/// * `name` - Variable name
/// * `value` - The value being tracked
/// * `sample_rate` - Probability of tracking (0.0 to 1.0)
///
/// # Examples
/// ```rust
/// # use borrowscope_runtime::*;
/// # reset();
/// // Track only ~10% of calls
/// let x = track_new_sampled("x", 42, 0.1);
/// ```
#[inline(always)]
pub fn track_new_sampled<T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    value: T,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] sample_rate: f64,
) -> T {
    #[cfg(feature = "track")]
    {
        if should_sample(sample_rate) {
            let type_name = std::any::type_name::<T>();
            let mut tracker = TRACKER.lock();
            tracker.record_new(name, type_name);
        }
    }
    value
}

/// Track variable creation with ID and sampling.
/// Only records the event if the sample check passes.
///
/// # Arguments
/// * `id` - Unique variable ID
/// * `name` - Variable name  
/// * `location` - Source location string
/// * `value` - The value being tracked
/// * `sample_rate` - Probability of tracking (0.0 to 1.0)
#[inline(always)]
pub fn track_new_with_id_sampled<T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: T,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] sample_rate: f64,
) -> T {
    #[cfg(feature = "track")]
    {
        if should_sample(sample_rate) {
            let type_name = std::any::type_name::<T>();
            let mut tracker = TRACKER.lock();
            tracker.record_new_with_id(id, name, type_name, location);
        }
    }
    value
}

/// Track immutable borrow with sampling.
#[inline(always)]
pub fn track_borrow_sampled<'a, T: ?Sized>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    value: &'a T,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] sample_rate: f64,
) -> &'a T {
    #[cfg(feature = "track")]
    {
        if should_sample(sample_rate) {
            let mut tracker = TRACKER.lock();
            tracker.record_borrow(name, "unknown", false);
        }
    }
    value
}

/// Track mutable borrow with sampling.
#[inline(always)]
pub fn track_borrow_mut_sampled<'a, T: ?Sized>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    value: &'a mut T,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] sample_rate: f64,
) -> &'a mut T {
    #[cfg(feature = "track")]
    {
        if should_sample(sample_rate) {
            let mut tracker = TRACKER.lock();
            tracker.record_borrow(name, "unknown", true);
        }
    }
    value
}

/// Track drop with sampling.
#[inline(always)]
pub fn track_drop_sampled(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] sample_rate: f64,
) {
    #[cfg(feature = "track")]
    {
        if should_sample(sample_rate) {
            let mut tracker = TRACKER.lock();
            tracker.record_drop(name);
        }
    }
}

/// Track move with sampling.
#[inline(always)]
pub fn track_move_sampled<T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] from: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] to: &str,
    value: T,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] sample_rate: f64,
) -> T {
    #[cfg(feature = "track")]
    {
        if should_sample(sample_rate) {
            let mut tracker = TRACKER.lock();
            tracker.record_move(from, to);
        }
    }
    value
}
