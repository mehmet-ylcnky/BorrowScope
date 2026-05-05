//! Async tracking: async blocks, await, futures

use super::TRACKER;

pub fn track_async_block_enter(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] block_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_async_block_enter(block_id, location);
    }
}

/// Track async block exit.
///
/// Records an `AsyncBlockExit` event. Use this when exiting an async block.
///
/// # Arguments
///
/// * `block_id` - Identifier matching the corresponding `track_async_block_enter`
/// * `location` - Source location
#[inline(always)]
pub fn track_async_block_exit(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] block_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_async_block_exit(block_id, location);
    }
}

/// Track await expression start.
///
/// Records an `AwaitStart` event. Use this before awaiting a future.
///
/// # Arguments
///
/// * `await_id` - Unique identifier for this await point
/// * `future_name` - Name or description of the future being awaited
/// * `location` - Source location
#[inline(always)]
pub fn track_await_start(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] await_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] future_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_await_start(await_id, future_name, location);
    }
}

/// Track await with live variable information from static analysis.
#[inline(always)]
pub fn track_await_start_with_live_vars(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] await_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] future_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] live_variables: &[&str],
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_await_start_with_live_vars(await_id, future_name, location, live_variables);
    }
}

/// Track await expression completion.
///
/// Records an `AwaitEnd` event. Use this after a future completes.
///
/// # Arguments
///
/// * `await_id` - Identifier matching the corresponding `track_await_start`
/// * `location` - Source location
#[inline(always)]
pub fn track_await_end(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] await_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_await_end(await_id, location);
    }
}
