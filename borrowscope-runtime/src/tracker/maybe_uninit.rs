//! MaybeUninit tracking

use super::TRACKER;

pub fn track_maybe_uninit_uninit<T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: std::mem::MaybeUninit<T>,
) -> std::mem::MaybeUninit<T> {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_maybe_uninit_new(name, false, location);
    }
    value
}

/// Track MaybeUninit::new (initialized)
#[inline(always)]
pub fn track_maybe_uninit_new<T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: std::mem::MaybeUninit<T>,
) -> std::mem::MaybeUninit<T> {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_maybe_uninit_new(name, true, location);
    }
    value
}

/// Track MaybeUninit::write
#[inline(always)]
pub fn track_maybe_uninit_write<'a, T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] var_id: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: &'a mut T,
) -> &'a mut T {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_maybe_uninit_write(var_id, location);
    }
    value
}

/// Track MaybeUninit::assume_init (unsafe)
#[inline(always)]
pub fn track_maybe_uninit_assume_init<T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] var_id: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: T,
) -> T {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_maybe_uninit_assume_init(var_id, location);
    }
    value
}

/// Track MaybeUninit::assume_init_read (unsafe)
#[inline(always)]
pub fn track_maybe_uninit_assume_init_read<T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] var_id: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: T,
) -> T {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_maybe_uninit_assume_init_read(var_id, location);
    }
    value
}

/// Track MaybeUninit::assume_init_drop (unsafe)
#[inline(always)]
pub fn track_maybe_uninit_assume_init_drop(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] var_id: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_maybe_uninit_assume_init_drop(var_id, location);
    }
}
