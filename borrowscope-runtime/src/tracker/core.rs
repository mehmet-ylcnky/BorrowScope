//! Core ownership tracking: new, borrow, move, drop

use super::TRACKER;

///
/// # Arguments
///
/// * `name` - A descriptive name for the variable (used in event output)
/// * `value` - The value being tracked (returned unchanged)
///
/// # Returns
///
/// The input `value`, unchanged. This allows chaining:
/// ```rust
/// # use borrowscope_runtime::*;
/// # reset();
/// let x = track_new("x", 42);
/// assert_eq!(x, 42);
/// ```
///
/// # Examples
///
/// Basic usage:
/// ```rust
/// # use borrowscope_runtime::*;
/// # reset();
/// let data = track_new("data", vec![1, 2, 3]);
/// let events = get_events();
/// assert!(events[0].is_new());
/// ```
///
/// With structs:
/// ```rust
/// # use borrowscope_runtime::*;
/// # reset();
/// struct Point { x: i32, y: i32 }
/// let p = track_new("point", Point { x: 10, y: 20 });
/// ```
#[inline(always)]
pub fn track_new<T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    value: T,
) -> T {
    #[cfg(feature = "track")]
    {
        let type_name = std::any::type_name::<T>();
        let mut tracker = TRACKER.lock();
        tracker.record_new(name, type_name);
    }
    value
}

/// Track an immutable borrow.
///
/// Records a `Borrow` event with `mutable: false` and returns the reference unchanged.
/// Use this when creating a shared reference (`&T`).
///
/// # Arguments
///
/// * `name` - A descriptive name for the borrow
/// * `value` - The reference being tracked (returned unchanged)
///
/// # Returns
///
/// The input reference, unchanged.
///
/// # Examples
///
/// ```rust
/// # use borrowscope_runtime::*;
/// # reset();
/// let data = track_new("data", vec![1, 2, 3]);
/// let r1 = track_borrow("r1", &data);
/// let r2 = track_borrow("r2", &data); // Multiple immutable borrows OK
/// println!("{:?}, {:?}", r1, r2);
///
/// let events = get_events();
/// assert!(events[1].is_borrow());
/// assert!(events[2].is_borrow());
/// ```
#[inline(always)]
pub fn track_borrow<'a, T: ?Sized>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    value: &'a T,
) -> &'a T {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_borrow(name, "unknown", false);
    }
    value
}

/// Track a mutable borrow.
///
/// Records a `Borrow` event with `mutable: true` and returns the reference unchanged.
/// Use this when creating an exclusive reference (`&mut T`).
///
/// # Arguments
///
/// * `name` - A descriptive name for the borrow
/// * `value` - The mutable reference being tracked (returned unchanged)
///
/// # Returns
///
/// The input mutable reference, unchanged.
///
/// # Examples
///
/// ```rust
/// # use borrowscope_runtime::*;
/// # reset();
/// let mut data = track_new("data", vec![1, 2, 3]);
/// {
///     let r = track_borrow_mut("r", &mut data);
///     r.push(4);
/// }
/// // Mutable borrow ended, can borrow again
/// let events = get_events();
/// assert!(events[1].is_borrow());
/// ```
#[inline(always)]
pub fn track_borrow_mut<'a, T: ?Sized>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    value: &'a mut T,
) -> &'a mut T {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_borrow(name, "unknown", true);
    }
    value
}

/// Track an ownership move.
///
/// Records a `Move` event and returns the value unchanged.
/// Use this when ownership transfers from one variable to another.
///
/// # Arguments
///
/// * `from_name` - Name of the source variable (giving up ownership)
/// * `to_name` - Name of the destination variable (receiving ownership)
/// * `value` - The value being moved (returned unchanged)
///
/// # Returns
///
/// The input `value`, unchanged.
///
/// # Examples
///
/// ```rust
/// # use borrowscope_runtime::*;
/// # reset();
/// let s1 = track_new("s1", String::from("hello"));
/// let s2 = track_move("s1", "s2", s1);
/// // s1 is no longer valid, s2 owns the String
///
/// let events = get_events();
/// assert!(events[1].is_move());
/// ```
#[inline(always)]
pub fn track_move<T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] from_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] to_name: &str,
    value: T,
) -> T {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_move(from_name, to_name);
    }
    value
}

/// Track a variable going out of scope.
///
/// Records a `Drop` event. Call this when a variable's lifetime ends.
/// Unlike other tracking functions, this doesn't return a value since
/// the variable is being destroyed.
///
/// # Arguments
///
/// * `name` - Name of the variable being dropped
///
/// # Examples
///
/// ```rust
/// # use borrowscope_runtime::*;
/// # reset();
/// {
///     let x = track_new("x", 42);
///     // x goes out of scope here
///     track_drop("x");
/// }
///
/// let events = get_events();
/// assert!(events[1].is_drop());
/// ```
///
/// # Note
///
/// For automatic drop tracking, consider using RAII guards or the
/// future `borrowscope-macro` crate which will instrument drops automatically.
#[inline(always)]
pub fn track_drop(#[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_drop(name);
    }
}

/// Track variable drop with precise location from static analysis.
///
/// Like `track_drop`, but attaches the exact source location where the variable
/// goes out of scope (as determined by the analyzer).
#[inline(always)]
pub fn track_drop_at(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_drop_at(name, location);
    }
}

/// Track multiple drops in batch (optimized).
///
/// Records multiple `Drop` events efficiently with a single lock acquisition.
/// Use this when multiple variables go out of scope simultaneously.
///
/// # Arguments
///
/// * `names` - Slice of variable names being dropped
///
/// # Examples
///
/// ```rust
/// # use borrowscope_runtime::*;
/// # reset();
/// let a = track_new("a", 1);
/// let b = track_new("b", 2);
/// let c = track_new("c", 3);
/// // All go out of scope together
/// track_drop_batch(&["a", "b", "c"]);
///
/// let events = get_events();
/// assert_eq!(events.len(), 6); // 3 New + 3 Drop
/// ```
#[inline(always)]
pub fn track_drop_batch(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] names: &[&str],
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        for &name in names {
            tracker.record_drop(name);
        }
    }
}

/// Reset tracking state.
///
/// Clears all recorded events and resets internal counters.
/// Call this before starting a new tracking session.
///
/// # Examples
///
/// ```rust
/// # use borrowscope_runtime::*;
/// let _ = track_new("x", 1);
/// assert!(!get_events().is_empty());
///
/// reset();
/// assert!(get_events().is_empty());
/// ```
///
/// # Thread Safety
///
/// This function is thread-safe but will clear events from all threads.
/// In multi-threaded tests, use synchronization to ensure reset completes
/// before other threads start tracking.
pub fn __track_new_with_id_helper<T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: T,
) -> T {
    #[cfg(feature = "track")]
    {
        let type_name = std::any::type_name::<T>();
        let mut tracker = TRACKER.lock();
        tracker.record_new_with_id(id, name, type_name, location);
    }
    value
}

/// Track a new variable with explicit ID and location (advanced API)
#[inline(always)]
pub fn track_new_with_id<T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] type_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: T,
) -> T {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_new_with_id(id, name, type_name, location);
    }
    value
}

/// Track an immutable borrow with full metadata (advanced API)
#[inline(always)]
pub fn track_borrow_with_id<'a, T: ?Sized>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] borrower_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] owner_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] mutable: bool,
    value: &'a T,
) -> &'a T {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_borrow_with_id(borrower_id, owner_id, name, location, mutable);
    }
    value
}

/// Track a mutable borrow with full metadata (advanced API)
#[inline(always)]
pub fn track_borrow_mut_with_id<'a, T: ?Sized>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] borrower_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] owner_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: &'a mut T,
) -> &'a mut T {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_borrow_with_id(borrower_id, owner_id, name, location, true);
    }
    value
}

/// Track a move with explicit IDs and location (advanced API)
#[inline(always)]
pub fn track_move_with_id<T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] from_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] to_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] to_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: T,
) -> T {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_move_with_id(from_id, to_id, to_name, location);
    }
    value
}

/// Track a drop with explicit ID and location (advanced API)
#[inline(always)]
pub fn track_drop_with_id(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_drop_with_id(id, location);
    }
}

