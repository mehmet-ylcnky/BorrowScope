//! RAII guards for automatic drop tracking.
//!
//! This module provides guard types that automatically call [`track_drop`](crate::track_drop)
//! when they go out of scope, eliminating the need for manual drop tracking.
//!
//! # Example
//!
//! ```rust
//! use borrowscope_runtime::*;
//!
//! reset();
//! {
//!     let _x = track_new_guard("x", 42);
//!     // No need to call track_drop - happens automatically
//! }
//!
//! let events = get_events();
//! assert!(events.last().unwrap().is_drop());
//! ```

use crate::tracker::{track_drop, track_new};

/// RAII guard that tracks drop automatically.
///
/// When this guard goes out of scope, it calls [`track_drop`] with the stored name.
/// Use [`track_new_guard`] to create instances.
///
/// # Example
///
/// ```rust
/// # use borrowscope_runtime::*;
/// # reset();
/// {
///     let data = track_new_guard("data", vec![1, 2, 3]);
///     println!("{:?}", *data); // Deref to access inner value
/// } // track_drop("data") called here automatically
///
/// assert_eq!(get_events().len(), 2); // New + Drop
/// ```
pub struct TrackGuard<T> {
    name: &'static str,
    value: T,
}

impl<T> TrackGuard<T> {
    /// Create a new tracking guard.
    #[inline]
    fn new(name: &'static str, value: T) -> Self {
        Self { name, value }
    }

    /// Get the tracked name.
    #[inline]
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Consume the guard and return the inner value without tracking drop.
    ///
    /// Use this when transferring ownership and you'll track the drop elsewhere.
    #[inline]
    pub fn into_inner(self) -> T {
        let value = unsafe { std::ptr::read(&self.value) };
        std::mem::forget(self);
        value
    }
}

impl<T> std::ops::Deref for TrackGuard<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> std::ops::DerefMut for TrackGuard<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T> Drop for TrackGuard<T> {
    #[inline]
    fn drop(&mut self) {
        track_drop(self.name);
    }
}

/// Track a new variable with automatic drop tracking.
///
/// Returns a [`TrackGuard`] that automatically calls [`track_drop`] when it goes out of scope.
///
/// # Arguments
///
/// * `name` - A static string name for the variable (must be `'static` for the guard)
/// * `value` - The value to track
///
/// # Returns
///
/// A `TrackGuard<T>` that derefs to `T` and tracks drop automatically.
///
/// # Example
///
/// ```rust
/// # use borrowscope_runtime::*;
/// # reset();
/// fn example() {
///     let data = track_new_guard("data", vec![1, 2, 3]);
///     let sum: i32 = data.iter().sum();
///     println!("Sum: {}", sum);
///     // track_drop("data") called automatically here
/// }
///
/// example();
/// let events = get_events();
/// assert!(events[0].is_new());
/// assert!(events[1].is_drop());
/// ```
#[inline]
pub fn track_new_guard<T>(name: &'static str, value: T) -> TrackGuard<T> {
    let tracked = track_new(name, value);
    TrackGuard::new(name, tracked)
}

/// RAII guard for borrow tracking (immutable).
///
/// Tracks when the borrow ends by calling [`track_drop`] on drop.
pub struct BorrowGuard<'a, T: ?Sized> {
    name: &'static str,
    value: &'a T,
}

impl<'a, T: ?Sized> BorrowGuard<'a, T> {
    #[inline]
    fn new(name: &'static str, value: &'a T) -> Self {
        Self { name, value }
    }
}

impl<'a, T: ?Sized> std::ops::Deref for BorrowGuard<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'a, T: ?Sized> Drop for BorrowGuard<'a, T> {
    #[inline]
    fn drop(&mut self) {
        track_drop(self.name);
    }
}

/// Track an immutable borrow with automatic drop tracking.
///
/// # Example
///
/// ```rust
/// # use borrowscope_runtime::*;
/// # reset();
/// let data = track_new_guard("data", vec![1, 2, 3]);
/// {
///     let r = track_borrow_guard("r", &*data);
///     println!("{:?}", *r);
/// } // track_drop("r") called automatically
/// ```
#[inline]
pub fn track_borrow_guard<'a, T: ?Sized>(name: &'static str, value: &'a T) -> BorrowGuard<'a, T> {
    let tracked = crate::track_borrow(name, value);
    BorrowGuard::new(name, tracked)
}

/// RAII guard for mutable borrow tracking.
pub struct BorrowMutGuard<'a, T: ?Sized> {
    name: &'static str,
    value: &'a mut T,
}

impl<'a, T: ?Sized> BorrowMutGuard<'a, T> {
    #[inline]
    fn new(name: &'static str, value: &'a mut T) -> Self {
        Self { name, value }
    }
}

impl<'a, T: ?Sized> std::ops::Deref for BorrowMutGuard<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'a, T: ?Sized> std::ops::DerefMut for BorrowMutGuard<'a, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

impl<'a, T: ?Sized> Drop for BorrowMutGuard<'a, T> {
    #[inline]
    fn drop(&mut self) {
        track_drop(self.name);
    }
}

/// Track a mutable borrow with automatic drop tracking.
///
/// # Example
///
/// ```rust
/// # use borrowscope_runtime::*;
/// # reset();
/// let mut data = track_new_guard("data", vec![1, 2, 3]);
/// {
///     let mut r = track_borrow_mut_guard("r", &mut *data);
///     r.push(4);
/// } // track_drop("r") called automatically
/// ```
#[inline]
pub fn track_borrow_mut_guard<'a, T: ?Sized>(
    name: &'static str,
    value: &'a mut T,
) -> BorrowMutGuard<'a, T> {
    let tracked = crate::track_borrow_mut(name, value);
    BorrowMutGuard::new(name, tracked)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{get_events, reset};
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_track_guard_auto_drop() {
        reset();
        {
            let _x = track_new_guard("x", 42);
        }
        let events = get_events();
        assert_eq!(events.len(), 2);
        assert!(events[0].is_new());
        assert!(events[1].is_drop());
    }

    #[test]
    #[serial]
    fn test_track_guard_deref() {
        reset();
        let x = track_new_guard("x", vec![1, 2, 3]);
        assert_eq!(x.len(), 3);
        assert_eq!(x[0], 1);
    }

    #[test]
    #[serial]
    fn test_track_guard_deref_mut() {
        reset();
        let mut x = track_new_guard("x", vec![1, 2, 3]);
        x.push(4);
        assert_eq!(x.len(), 4);
    }

    #[test]
    #[serial]
    fn test_into_inner_no_drop() {
        reset();
        let x = track_new_guard("x", 42);
        let _val = x.into_inner();
        let events = get_events();
        assert_eq!(events.len(), 1); // Only New, no Drop
    }

    #[test]
    #[serial]
    fn test_borrow_guard() {
        reset();
        let data = track_new_guard("data", 42);
        {
            let _r = track_borrow_guard("r", &*data);
        }
        let events = get_events();
        assert_eq!(events.len(), 3); // New, Borrow, Drop(r)
    }

    #[test]
    #[serial]
    fn test_borrow_mut_guard() {
        reset();
        let mut data = track_new_guard("data", vec![1]);
        {
            let mut r = track_borrow_mut_guard("r", &mut *data);
            r.push(2);
        }
        assert_eq!(data.len(), 2);
    }
}
