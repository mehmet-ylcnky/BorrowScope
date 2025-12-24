//! Unsafe code tracking: raw pointers, unsafe blocks, FFI, transmute

use super::TRACKER;

pub fn track_raw_ptr<T: ?Sized>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] var_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] var_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] ptr_type: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    ptr: *const T,
) -> *const T {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_raw_ptr_created(
            var_name,
            var_id,
            ptr_type,
            ptr as *const () as usize,
            location,
        );
    }
    ptr
}

/// Track mutable raw pointer creation.
///
/// Records a `RawPtrCreated` event. Use this when creating a `*mut T` pointer.
///
/// # Arguments
///
/// * `var_name` - Name for the pointer variable
/// * `var_id` - Unique identifier
/// * `ptr_type` - Type description (e.g., "*mut i32")
/// * `location` - Source location
/// * `ptr` - The raw pointer (returned unchanged)
///
/// # Returns
///
/// The input pointer, unchanged.
///
/// # Safety
///
/// This function is safe to call, but the pointer it tracks may be unsafe to dereference.
#[inline(always)]
pub fn track_raw_ptr_mut<T: ?Sized>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] var_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] var_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] ptr_type: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    ptr: *mut T,
) -> *mut T {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_raw_ptr_created(
            var_name,
            var_id,
            ptr_type,
            ptr as *const () as usize,
            location,
        );
    }
    ptr
}

/// Track raw pointer dereference.
///
/// Records a `RawPtrDeref` event. Use this when dereferencing a raw pointer.
///
/// # Arguments
///
/// * `ptr_id` - Identifier of the pointer being dereferenced
/// * `location` - Source location
/// * `is_write` - `true` if writing through the pointer, `false` if reading
#[inline(always)]
pub fn track_raw_ptr_deref(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] ptr_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] is_write: bool,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_raw_ptr_deref(ptr_id, location, is_write);
    }
}

/// Track unsafe block entry.
///
/// Records an `UnsafeBlockEnter` event. Use this when entering an unsafe block.
///
/// # Arguments
///
/// * `block_id` - Unique identifier for this unsafe block
/// * `location` - Source location
#[inline(always)]
pub fn track_unsafe_block_enter(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] block_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_unsafe_block_enter(block_id, location);
    }
}

/// Track unsafe block exit.
///
/// Records an `UnsafeBlockExit` event. Use this when exiting an unsafe block.
///
/// # Arguments
///
/// * `block_id` - Identifier matching the corresponding `track_unsafe_block_enter`
/// * `location` - Source location
#[inline(always)]
pub fn track_unsafe_block_exit(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] block_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_unsafe_block_exit(block_id, location);
    }
}

/// Track unsafe function call.
///
/// Records an `UnsafeFnCall` event. Use this when calling an unsafe function.
///
/// # Arguments
///
/// * `fn_name` - Name of the unsafe function being called
/// * `location` - Source location
#[inline(always)]
pub fn track_unsafe_fn_call(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] fn_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_unsafe_fn_call(fn_name, location);
    }
}

/// Track FFI call.
///
/// Records an `FfiCall` event. Use this when calling a foreign function.
///
/// # Arguments
///
/// * `fn_name` - Name of the foreign function being called
/// * `location` - Source location
#[inline(always)]
pub fn track_ffi_call(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] fn_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_ffi_call(fn_name, location);
    }
}

/// Track transmute operation.
///
/// Records a `Transmute` event. Use this when using `std::mem::transmute`.
///
/// # Arguments
///
/// * `from_type` - Source type name
/// * `to_type` - Destination type name
/// * `location` - Source location
#[inline(always)]
pub fn track_transmute(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] from_type: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] to_type: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_transmute(from_type, to_type, location);
    }
}

/// Track union field access.
///
/// Records a `UnionFieldAccess` event. Use this when accessing a union field.
///
/// # Arguments
///
/// * `union_name` - Name of the union
/// * `field_name` - Name of the field being accessed
/// * `location` - Source location
#[inline(always)]
pub fn track_union_field_access(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] union_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] field_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_union_field_access(union_name, field_name, location);
    }
}
