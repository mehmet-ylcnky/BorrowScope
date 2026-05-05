//! Smart pointer tracking: Rc, Arc, Box, Weak, Pin, Cow

use super::TRACKER;
use std::borrow::ToOwned;

pub fn track_rc_new_with_id<T: ?Sized>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] type_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: std::rc::Rc<T>,
) -> std::rc::Rc<T> {
    #[cfg(feature = "track")]
    {
        let strong_count = std::rc::Rc::strong_count(&value);
        let weak_count = std::rc::Rc::weak_count(&value);
        let mut tracker = TRACKER.lock();
        tracker.record_rc_new_with_id(id, name, type_name, location, strong_count, weak_count);
    }
    value
}

/// Track Rc::clone with explicit IDs and location (advanced API)
#[inline(always)]
pub fn track_rc_clone_with_id<T: ?Sized>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] new_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] source_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: std::rc::Rc<T>,
) -> std::rc::Rc<T> {
    #[cfg(feature = "track")]
    {
        let strong_count = std::rc::Rc::strong_count(&value);
        let weak_count = std::rc::Rc::weak_count(&value);
        let mut tracker = TRACKER.lock();
        tracker.record_rc_clone_with_id(
            new_id,
            source_id,
            name,
            location,
            strong_count,
            weak_count,
        );
    }
    value
}

/// Track Arc::new with explicit ID and location (advanced API)
#[inline(always)]
pub fn track_arc_new_with_id<T: ?Sized>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] type_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: std::sync::Arc<T>,
) -> std::sync::Arc<T> {
    #[cfg(feature = "track")]
    {
        let strong_count = std::sync::Arc::strong_count(&value);
        let weak_count = std::sync::Arc::weak_count(&value);
        let mut tracker = TRACKER.lock();
        tracker.record_arc_new_with_id(id, name, type_name, location, strong_count, weak_count);
    }
    value
}

/// Track Arc::clone with explicit IDs and location (advanced API)
#[inline(always)]
pub fn track_arc_clone_with_id<T: ?Sized>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] new_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] source_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: std::sync::Arc<T>,
) -> std::sync::Arc<T> {
    #[cfg(feature = "track")]
    {
        let strong_count = std::sync::Arc::strong_count(&value);
        let weak_count = std::sync::Arc::weak_count(&value);
        let mut tracker = TRACKER.lock();
        tracker.record_arc_clone_with_id(
            new_id,
            source_id,
            name,
            location,
            strong_count,
            weak_count,
        );
    }
    value
}

/// Track `Rc::new` allocation.
///
/// Records an `RcNew` event with the current strong and weak reference counts.
/// Use this when creating a new reference-counted pointer.
///
/// # Arguments
///
/// * `name` - A descriptive name for the Rc
/// * `value` - The Rc being tracked (returned unchanged)
///
/// # Returns
///
/// The input `Rc`, unchanged.
///
/// # Examples
///
/// ```rust
/// # use borrowscope_runtime::*;
/// use std::rc::Rc;
/// # reset();
///
/// let shared = track_rc_new("shared", Rc::new(vec![1, 2, 3]));
/// assert_eq!(Rc::strong_count(&shared), 1);
///
/// let events = get_events();
/// assert!(events[0].is_rc());
/// assert_eq!(events[0].strong_count(), Some(1));
/// ```
#[inline(always)]
pub fn track_rc_new<T: ?Sized>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    value: std::rc::Rc<T>,
) -> std::rc::Rc<T> {
    #[cfg(feature = "track")]
    {
        let strong_count = std::rc::Rc::strong_count(&value);
        let weak_count = std::rc::Rc::weak_count(&value);
        let mut tracker = TRACKER.lock();
        tracker.record_rc_new(name, strong_count, weak_count);
    }
    value
}

/// Track `Rc::clone` operation.
///
/// Records an `RcClone` event with the updated reference counts.
/// Use this when cloning an Rc to share ownership.
///
/// # Arguments
///
/// * `name` - A descriptive name for the new clone
/// * `source_name` - Name of the Rc being cloned from
/// * `value` - The cloned Rc (returned unchanged)
///
/// # Returns
///
/// The input `Rc`, unchanged.
///
/// # Examples
///
/// ```rust
/// # use borrowscope_runtime::*;
/// use std::rc::Rc;
/// # reset();
///
/// let original = track_rc_new("original", Rc::new(42));
/// let clone1 = track_rc_clone("clone1", "original", Rc::clone(&original));
/// let clone2 = track_rc_clone("clone2", "original", Rc::clone(&original));
///
/// assert_eq!(Rc::strong_count(&original), 3);
///
/// let events = get_events();
/// assert_eq!(events[1].strong_count(), Some(2)); // After first clone
/// assert_eq!(events[2].strong_count(), Some(3)); // After second clone
/// ```
#[inline(always)]
pub fn track_rc_clone<T: ?Sized>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] source_name: &str,
    value: std::rc::Rc<T>,
) -> std::rc::Rc<T> {
    #[cfg(feature = "track")]
    {
        let strong_count = std::rc::Rc::strong_count(&value);
        let weak_count = std::rc::Rc::weak_count(&value);
        let mut tracker = TRACKER.lock();
        tracker.record_rc_clone(name, source_name, strong_count, weak_count);
    }
    value
}

/// Track `Arc::new` allocation.
///
/// Records an `ArcNew` event with the current strong and weak reference counts.
/// Use this when creating a new thread-safe reference-counted pointer.
///
/// # Arguments
///
/// * `name` - A descriptive name for the Arc
/// * `value` - The Arc being tracked (returned unchanged)
///
/// # Returns
///
/// The input `Arc`, unchanged.
///
/// # Examples
///
/// ```rust
/// # use borrowscope_runtime::*;
/// use std::sync::Arc;
/// # reset();
///
/// let shared = track_arc_new("shared", Arc::new(vec![1, 2, 3]));
/// assert_eq!(Arc::strong_count(&shared), 1);
///
/// let events = get_events();
/// assert!(events[0].is_arc());
/// ```
#[inline(always)]
pub fn track_arc_new<T: ?Sized>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    value: std::sync::Arc<T>,
) -> std::sync::Arc<T> {
    #[cfg(feature = "track")]
    {
        let strong_count = std::sync::Arc::strong_count(&value);
        let weak_count = std::sync::Arc::weak_count(&value);
        let mut tracker = TRACKER.lock();
        tracker.record_arc_new(name, strong_count, weak_count);
    }
    value
}

/// Track `Arc::clone` operation.
///
/// Records an `ArcClone` event with the updated reference counts.
/// Use this when cloning an Arc for thread-safe shared ownership.
///
/// # Arguments
///
/// * `name` - A descriptive name for the new clone
/// * `source_name` - Name of the Arc being cloned from
/// * `value` - The cloned Arc (returned unchanged)
///
/// # Returns
///
/// The input `Arc`, unchanged.
///
/// # Examples
///
/// ```rust
/// # use borrowscope_runtime::*;
/// use std::sync::Arc;
/// use std::thread;
/// # reset();
///
/// let data = track_arc_new("data", Arc::new(42));
/// let data_clone = track_arc_clone("thread_copy", "data", Arc::clone(&data));
///
/// let handle = thread::spawn(move || {
///     println!("Value: {}", *data_clone);
/// });
/// handle.join().unwrap();
/// ```
#[inline(always)]
pub fn track_arc_clone<T: ?Sized>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] source_name: &str,
    value: std::sync::Arc<T>,
) -> std::sync::Arc<T> {
    #[cfg(feature = "track")]
    {
        let strong_count = std::sync::Arc::strong_count(&value);
        let weak_count = std::sync::Arc::weak_count(&value);
        let mut tracker = TRACKER.lock();
        tracker.record_arc_clone(name, source_name, strong_count, weak_count);
    }
    value
}

/// Track `RefCell::new` allocation.
///
/// Records a `RefCellNew` event. Use this when creating a new RefCell
/// for interior mutability.
///
/// # Arguments
///
/// * `name` - A descriptive name for the RefCell
/// * `value` - The RefCell being tracked (returned unchanged)
///
/// # Returns
///
/// The input `RefCell`, unchanged.
///
/// # Examples
///
/// ```rust
/// # use borrowscope_runtime::*;
/// use std::cell::RefCell;
/// # reset();
///
/// let cell = track_refcell_new("cell", RefCell::new(42));
///
/// let events = get_events();
/// assert!(events[0].is_refcell());
/// ```
#[inline(always)]
pub fn track_weak_new<T: ?Sized>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] source_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: std::rc::Weak<T>,
) -> std::rc::Weak<T> {
    #[cfg(feature = "track")]
    {
        let weak_count = std::rc::Weak::weak_count(&value);
        let mut tracker = TRACKER.lock();
        tracker.record_weak_new(name, source_name, weak_count, location);
    }
    value
}

/// Track sync Weak::new or Arc::downgrade
#[inline(always)]
pub fn track_weak_new_sync<T: ?Sized>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] source_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: std::sync::Weak<T>,
) -> std::sync::Weak<T> {
    #[cfg(feature = "track")]
    {
        let weak_count = std::sync::Weak::weak_count(&value);
        let mut tracker = TRACKER.lock();
        tracker.record_weak_new(name, source_name, weak_count, location);
    }
    value
}

/// Track Weak::clone
#[inline(always)]
pub fn track_weak_clone<T: ?Sized>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] source_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: std::rc::Weak<T>,
) -> std::rc::Weak<T> {
    #[cfg(feature = "track")]
    {
        let weak_count = std::rc::Weak::weak_count(&value);
        let mut tracker = TRACKER.lock();
        tracker.record_weak_clone(name, source_name, weak_count, location);
    }
    value
}

/// Track sync Weak::clone
#[inline(always)]
pub fn track_weak_clone_sync<T: ?Sized>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] source_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: std::sync::Weak<T>,
) -> std::sync::Weak<T> {
    #[cfg(feature = "track")]
    {
        let weak_count = std::sync::Weak::weak_count(&value);
        let mut tracker = TRACKER.lock();
        tracker.record_weak_clone(name, source_name, weak_count, location);
    }
    value
}

/// Track Weak::upgrade
#[inline(always)]
pub fn track_weak_upgrade<T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] weak_id: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: Option<std::rc::Rc<T>>,
) -> Option<std::rc::Rc<T>> {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_weak_upgrade(weak_id, value.is_some(), location);
    }
    value
}

/// Track sync Weak::upgrade
#[inline(always)]
pub fn track_weak_upgrade_sync<T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] weak_id: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: Option<std::sync::Arc<T>>,
) -> Option<std::sync::Arc<T>> {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_weak_upgrade(weak_id, value.is_some(), location);
    }
    value
}

/// Track Box::new
#[inline(always)]
pub fn track_box_new<T: ?Sized>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: Box<T>,
) -> Box<T> {
    #[cfg(feature = "track")]
    {
        let type_name = std::any::type_name::<T>();
        let mut tracker = TRACKER.lock();
        tracker.record_box_new(name, type_name, location);
    }
    value
}

/// Track Box::into_raw
#[inline(always)]
pub fn track_box_into_raw<T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] box_id: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    ptr: *mut T,
) -> *mut T {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_box_into_raw(box_id, location);
    }
    ptr
}

/// Track Box::from_raw
#[inline(always)]
pub fn track_box_from_raw<T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: Box<T>,
) -> Box<T> {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_box_from_raw(name, location);
    }
    value
}

/// Track lock guard acquisition
#[inline(always)]
pub fn track_lock_guard_acquire(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] guard_id: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] lock_id: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] lock_type: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_lock_guard_acquire(guard_id, lock_id, lock_type, location);
    }
}

/// Track lock guard drop
#[inline(always)]
pub fn track_lock_guard_drop(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] guard_id: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_lock_guard_drop(guard_id, location);
    }
}

/// Track Pin::new
#[inline(always)]
pub fn track_pin_new<P>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: std::pin::Pin<P>,
) -> std::pin::Pin<P> {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_pin_new(name, location);
    }
    value
}

/// Track Pin::into_inner
#[inline(always)]
pub fn track_pin_into_inner<T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] pin_id: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: T,
) -> T {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_pin_into_inner(pin_id, location);
    }
    value
}

/// Track Cow::Borrowed
#[inline(always)]
pub fn track_cow_borrowed<'a, B: ?Sized + ToOwned>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: std::borrow::Cow<'a, B>,
) -> std::borrow::Cow<'a, B> {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_cow_borrowed(name, location);
    }
    value
}

/// Track Cow::Owned
#[inline(always)]
pub fn track_cow_owned<'a, B: ?Sized + ToOwned>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: std::borrow::Cow<'a, B>,
) -> std::borrow::Cow<'a, B> {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_cow_owned(name, location);
    }
    value
}

/// Track Cow::to_mut (clone-on-write)
#[inline(always)]
pub fn track_cow_to_mut(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] cow_id: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] cloned: bool,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_cow_to_mut(cow_id, cloned, location);
    }
}
