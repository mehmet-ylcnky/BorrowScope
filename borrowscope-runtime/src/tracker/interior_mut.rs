//! Interior mutability tracking: RefCell, Cell, OnceCell, OnceLock

use super::TRACKER;

pub fn track_refcell_new<T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    value: std::cell::RefCell<T>,
) -> std::cell::RefCell<T> {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_refcell_new(name);
    }
    value
}

/// Track `RefCell::borrow` operation.
///
/// Records a `RefCellBorrow` event with `is_mutable: false`.
/// Use this when obtaining a shared borrow from a RefCell.
///
/// # Arguments
///
/// * `borrow_id` - Unique identifier for this borrow
/// * `refcell_id` - Identifier of the RefCell being borrowed
/// * `location` - Source location (e.g., "file.rs:42")
/// * `value` - The Ref guard (returned unchanged)
///
/// # Returns
///
/// The input `Ref` guard, unchanged.
///
/// # Examples
///
/// ```rust
/// # use borrowscope_runtime::*;
/// use std::cell::RefCell;
/// # reset();
///
/// let cell = track_refcell_new("cell", RefCell::new(42));
/// {
///     let guard = track_refcell_borrow("borrow1", "cell", "main.rs:10", cell.borrow());
///     println!("Value: {}", *guard);
/// } // guard dropped here
/// ```
#[inline(always)]
pub fn track_refcell_borrow<'a, T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] borrow_id: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] refcell_id: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: std::cell::Ref<'a, T>,
) -> std::cell::Ref<'a, T> {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_refcell_borrow(borrow_id, refcell_id, false, location);
    }
    value
}

/// Track `RefCell::borrow_mut` operation.
///
/// Records a `RefCellBorrow` event with `is_mutable: true`.
/// Use this when obtaining an exclusive borrow from a RefCell.
///
/// # Arguments
///
/// * `borrow_id` - Unique identifier for this borrow
/// * `refcell_id` - Identifier of the RefCell being borrowed
/// * `location` - Source location (e.g., "file.rs:42")
/// * `value` - The RefMut guard (returned unchanged)
///
/// # Returns
///
/// The input `RefMut` guard, unchanged.
///
/// # Examples
///
/// ```rust
/// # use borrowscope_runtime::*;
/// use std::cell::RefCell;
/// # reset();
///
/// let cell = track_refcell_new("cell", RefCell::new(42));
/// {
///     let mut guard = track_refcell_borrow_mut("borrow1", "cell", "main.rs:10", cell.borrow_mut());
///     *guard = 100;
/// }
/// assert_eq!(*cell.borrow(), 100);
/// ```
#[inline(always)]
pub fn track_refcell_borrow_mut<'a, T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] borrow_id: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] refcell_id: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: std::cell::RefMut<'a, T>,
) -> std::cell::RefMut<'a, T> {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_refcell_borrow(borrow_id, refcell_id, true, location);
    }
    value
}

/// Track RefCell borrow drop (when Ref/RefMut is dropped).
///
/// Records a `RefCellDrop` event. Call this when a RefCell guard goes out of scope.
///
/// # Arguments
///
/// * `borrow_id` - The identifier used when the borrow was created
/// * `location` - Source location where the drop occurs
///
/// # Examples
///
/// ```rust
/// # use borrowscope_runtime::*;
/// use std::cell::RefCell;
/// # reset();
///
/// let cell = track_refcell_new("cell", RefCell::new(42));
/// {
///     let guard = track_refcell_borrow("b1", "cell", "main.rs:10", cell.borrow());
///     // use guard...
///     track_refcell_drop("b1", "main.rs:12");
/// }
/// ```
#[inline(always)]
pub fn track_refcell_drop(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] borrow_id: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_refcell_drop(borrow_id, location);
    }
}

/// Track `Cell::new` allocation.
///
/// Records a `CellNew` event. Use this when creating a new Cell
/// for interior mutability with Copy types.
///
/// # Arguments
///
/// * `name` - A descriptive name for the Cell
/// * `value` - The Cell being tracked (returned unchanged)
///
/// # Returns
///
/// The input `Cell`, unchanged.
///
/// # Examples
///
/// ```rust
/// # use borrowscope_runtime::*;
/// use std::cell::Cell;
/// # reset();
///
/// let counter = track_cell_new("counter", Cell::new(0));
///
/// let events = get_events();
/// assert!(events[0].is_cell());
/// ```
#[inline(always)]
pub fn track_cell_new<T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    value: std::cell::Cell<T>,
) -> std::cell::Cell<T> {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_cell_new(name);
    }
    value
}

/// Track `Cell::get` operation.
///
/// Records a `CellGet` event. Use this when reading a value from a Cell.
///
/// # Arguments
///
/// * `cell_id` - Identifier of the Cell being read
/// * `location` - Source location (e.g., "file.rs:42")
/// * `value` - The value read from the Cell (returned unchanged)
///
/// # Returns
///
/// The input value, unchanged.
///
/// # Examples
///
/// ```rust
/// # use borrowscope_runtime::*;
/// use std::cell::Cell;
/// # reset();
///
/// let counter = track_cell_new("counter", Cell::new(42));
/// let value = track_cell_get("counter", "main.rs:5", counter.get());
/// assert_eq!(value, 42);
/// ```
#[inline(always)]
pub fn track_cell_get<T: Copy>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] cell_id: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: T,
) -> T {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_cell_get(cell_id, location);
    }
    value
}

/// Track `Cell::set` operation.
///
/// Records a `CellSet` event. Use this when writing a value to a Cell.
///
/// # Arguments
///
/// * `cell_id` - Identifier of the Cell being written
/// * `location` - Source location (e.g., "file.rs:42")
///
/// # Examples
///
/// ```rust
/// # use borrowscope_runtime::*;
/// use std::cell::Cell;
/// # reset();
///
/// let counter = track_cell_new("counter", Cell::new(0));
/// counter.set(1);
/// track_cell_set("counter", "main.rs:5");
/// counter.set(2);
/// track_cell_set("counter", "main.rs:6");
///
/// let events = get_events();
/// assert_eq!(events.iter().filter(|e| matches!(e, borrowscope_runtime::Event::CellSet { .. })).count(), 2);
/// ```
#[inline(always)]
pub fn track_cell_set(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] cell_id: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_cell_set(cell_id, location);
    }
}

/// Track static variable initialization.
///
/// Records a `StaticInit` event. Use this when a static variable is first initialized.
///
/// # Arguments
///
/// * `var_name` - Name of the static variable
/// * `var_id` - Unique identifier for the variable
/// * `type_name` - Type of the static variable
/// * `is_mutable` - Whether this is a `static mut`
/// * `value` - The initial value (returned unchanged)
///
/// # Returns
///
/// The input value, unchanged.
#[inline(always)]
pub fn track_once_cell_new<T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: std::cell::OnceCell<T>,
) -> std::cell::OnceCell<T> {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_once_cell_new(name, "OnceCell", location);
    }
    value
}

/// Track OnceLock::new
#[inline(always)]
pub fn track_once_lock_new<T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: std::sync::OnceLock<T>,
) -> std::sync::OnceLock<T> {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_once_cell_new(name, "OnceLock", location);
    }
    value
}

/// Track OnceCell::set
#[inline(always)]
pub fn track_once_cell_set<T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] cell_id: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    result: Result<(), T>,
) -> Result<(), T> {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_once_cell_set(cell_id, result.is_ok(), location);
    }
    result
}

/// Track OnceCell::get
#[inline(always)]
pub fn track_once_cell_get<'a, T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] cell_id: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: Option<&'a T>,
) -> Option<&'a T> {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_once_cell_get(cell_id, value.is_some(), location);
    }
    value
}

/// Track OnceCell::get_or_init
#[inline(always)]
pub fn track_once_cell_get_or_init<'a, T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] cell_id: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] was_initialized: bool,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: &'a T,
) -> &'a T {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_once_cell_get_or_init(cell_id, was_initialized, location);
    }
    value
}

// =============================================================================
// Phase 11: MaybeUninit Tracking Functions
// =============================================================================

