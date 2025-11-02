//! Integration tests for unsafe code tracking
//!
//! These tests verify that unsafe operations are properly tracked including
//! raw pointers, unsafe blocks, FFI calls, transmute, and union field access.

use borrowscope_runtime::*;
use serial_test::serial;

// ============================================================================
// RAW POINTER TESTS
// ============================================================================

#[test]
#[serial]
fn test_raw_ptr_const_creation() {
    reset();

    let x = 42;
    let _ptr = track_raw_ptr("ptr", 1, "*const i32", "test.rs:10:5", &x as *const i32);

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_raw_ptr());
    assert!(events[0].is_unsafe());
}

#[test]
#[serial]
fn test_raw_ptr_mut_creation() {
    reset();

    let mut x = 42;
    let _ptr = track_raw_ptr_mut("ptr", 1, "*mut i32", "test.rs:20:5", &mut x as *mut i32);

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_raw_ptr());
}

#[test]
#[serial]
fn test_raw_ptr_deref_read() {
    reset();

    let x = 42;
    let ptr = &x as *const i32;
    track_raw_ptr_deref(1, "test.rs:30:9", false);

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_raw_ptr());

    unsafe {
        assert_eq!(*ptr, 42);
    }
}

#[test]
#[serial]
fn test_raw_ptr_deref_write() {
    reset();

    let mut x = 42;
    let ptr = &mut x as *mut i32;
    track_raw_ptr_deref(1, "test.rs:40:9", true);

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_raw_ptr());

    unsafe {
        *ptr = 100;
    }
    assert_eq!(x, 100);
}

#[test]
#[serial]
fn test_raw_ptr_null() {
    reset();

    let _ptr = track_raw_ptr(
        "null_ptr",
        1,
        "*const i32",
        "test.rs:50:5",
        std::ptr::null::<i32>(),
    );

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_raw_ptr());
}

#[test]
#[serial]
fn test_raw_ptr_from_integer() {
    reset();

    let _ptr = track_raw_ptr("ptr", 1, "*const u8", "test.rs:60:5", 0x1000 as *const u8);

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
#[serial]
fn test_raw_ptr_offset() {
    reset();

    let arr = [1, 2, 3, 4, 5];
    let ptr = track_raw_ptr("ptr", 1, "*const i32", "test.rs:70:5", arr.as_ptr());
    track_raw_ptr_deref(1, "test.rs:71:9", false);

    let events = get_events();
    assert_eq!(events.len(), 2);

    unsafe {
        assert_eq!(*ptr, 1);
        assert_eq!(*ptr.offset(2), 3);
    }
}

#[test]
#[serial]
fn test_raw_ptr_cast() {
    reset();

    let x: i32 = 42;
    let ptr_i32 = track_raw_ptr("ptr_i32", 1, "*const i32", "test.rs:80:5", &x as *const i32);
    let _ptr_u8 = track_raw_ptr(
        "ptr_u8",
        2,
        "*const u8",
        "test.rs:81:5",
        ptr_i32 as *const u8,
    );

    let events = get_events();
    assert_eq!(events.len(), 2);
}

// ============================================================================
// UNSAFE BLOCK TESTS
// ============================================================================

#[test]
#[serial]
fn test_unsafe_block_enter_exit() {
    reset();

    track_unsafe_block_enter(1, "test.rs:90:5");
    track_unsafe_block_exit(1, "test.rs:92:5");

    let events = get_events();
    assert_eq!(events.len(), 2);
    assert!(events[0].is_unsafe());
    assert!(events[1].is_unsafe());
}

#[test]
#[serial]
fn test_nested_unsafe_blocks() {
    reset();

    track_unsafe_block_enter(1, "test.rs:100:5");
    track_unsafe_block_enter(2, "test.rs:101:9");
    track_unsafe_block_exit(2, "test.rs:103:9");
    track_unsafe_block_exit(1, "test.rs:104:5");

    let events = get_events();
    assert_eq!(events.len(), 4);
}

#[test]
#[serial]
fn test_unsafe_block_with_operations() {
    reset();

    track_unsafe_block_enter(1, "test.rs:110:5");
    let x = 42;
    let ptr = track_raw_ptr("ptr", 2, "*const i32", "test.rs:111:9", &x as *const i32);
    track_raw_ptr_deref(2, "test.rs:112:9", false);
    track_unsafe_block_exit(1, "test.rs:113:5");

    let events = get_events();
    assert_eq!(events.len(), 4);

    unsafe {
        assert_eq!(*ptr, 42);
    }
}

// ============================================================================
// UNSAFE FUNCTION TESTS
// ============================================================================

#[test]
#[serial]
fn test_unsafe_fn_call() {
    reset();

    track_unsafe_fn_call("dangerous_operation", "test.rs:120:5");

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_unsafe());
}

#[test]
#[serial]
fn test_multiple_unsafe_fn_calls() {
    reset();

    track_unsafe_fn_call("fn1", "test.rs:130:5");
    track_unsafe_fn_call("fn2", "test.rs:131:5");
    track_unsafe_fn_call("fn3", "test.rs:132:5");

    let events = get_events();
    assert_eq!(events.len(), 3);
}

// ============================================================================
// FFI TESTS
// ============================================================================

#[test]
#[serial]
fn test_ffi_call() {
    reset();

    track_ffi_call("external_function", "test.rs:140:5");

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_ffi());
    assert!(events[0].is_unsafe());
}

#[test]
#[serial]
fn test_multiple_ffi_calls() {
    reset();

    track_ffi_call("malloc", "test.rs:150:5");
    track_ffi_call("free", "test.rs:151:5");
    track_ffi_call("printf", "test.rs:152:5");

    let events = get_events();
    assert_eq!(events.len(), 3);
    assert!(events.iter().all(|e| e.is_ffi()));
}

#[test]
#[serial]
fn test_ffi_with_raw_pointers() {
    reset();

    let x = 42;
    let ptr = track_raw_ptr("ptr", 1, "*const i32", "test.rs:160:5", &x as *const i32);
    track_ffi_call("process_data", "test.rs:161:5");

    let events = get_events();
    assert_eq!(events.len(), 2);
    assert!(events[0].is_raw_ptr());
    assert!(events[1].is_ffi());

    let _ = ptr;
}

// ============================================================================
// TRANSMUTE TESTS
// ============================================================================

#[test]
#[serial]
fn test_transmute() {
    reset();

    track_transmute("i32", "f32", "test.rs:170:5");

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_unsafe());
}

#[test]
#[serial]
fn test_transmute_pointer_to_usize() {
    reset();

    let x = 42;
    let ptr = &x as *const i32;
    track_transmute("*const i32", "usize", "test.rs:180:5");

    let events = get_events();
    assert_eq!(events.len(), 1);

    let _ = ptr as usize;
}

#[test]
#[serial]
fn test_transmute_array_to_struct() {
    reset();

    track_transmute("[u8; 4]", "u32", "test.rs:190:5");

    let events = get_events();
    assert_eq!(events.len(), 1);
}

// ============================================================================
// UNION TESTS
// ============================================================================

#[test]
#[serial]
fn test_union_field_access() {
    reset();

    track_union_field_access("MyUnion", "int_value", "test.rs:200:5");

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_unsafe());
}

#[test]
#[serial]
fn test_multiple_union_field_accesses() {
    reset();

    track_union_field_access("MyUnion", "int_value", "test.rs:210:5");
    track_union_field_access("MyUnion", "float_value", "test.rs:211:5");

    let events = get_events();
    assert_eq!(events.len(), 2);
}

// ============================================================================
// COMPLEX SCENARIOS
// ============================================================================

#[test]
#[serial]
fn test_raw_ptr_aliasing() {
    reset();

    let mut x = 42;
    let ptr1 = track_raw_ptr_mut("ptr1", 1, "*mut i32", "test.rs:220:5", &mut x as *mut i32);
    let ptr2 = track_raw_ptr_mut("ptr2", 2, "*mut i32", "test.rs:221:5", &mut x as *mut i32);

    let events = get_events();
    assert_eq!(events.len(), 2);

    unsafe {
        *ptr1 = 100;
        *ptr2 = 200;
    }
    assert_eq!(x, 200);
}

#[test]
#[serial]
fn test_raw_ptr_lifetime_extension() {
    reset();

    let ptr = {
        let x = 42;
        track_raw_ptr("ptr", 1, "*const i32", "test.rs:230:9", &x as *const i32)
    };

    let events = get_events();
    assert_eq!(events.len(), 1);

    // ptr is now dangling - we track it but can't prevent use
    let _ = ptr;
}

#[test]
#[serial]
fn test_slice_from_raw_parts() {
    reset();

    let arr = [1, 2, 3, 4, 5];
    let ptr = track_raw_ptr("ptr", 1, "*const i32", "test.rs:240:5", arr.as_ptr());
    track_unsafe_fn_call("slice::from_raw_parts", "test.rs:241:5");

    let events = get_events();
    assert_eq!(events.len(), 2);

    unsafe {
        let slice = std::slice::from_raw_parts(ptr, 5);
        assert_eq!(slice.len(), 5);
    }
}

#[test]
#[serial]
fn test_ptr_read_write() {
    reset();

    let mut x = 42;
    let ptr = track_raw_ptr_mut("ptr", 1, "*mut i32", "test.rs:250:5", &mut x as *mut i32);
    track_unsafe_fn_call("ptr::read", "test.rs:251:5");
    track_unsafe_fn_call("ptr::write", "test.rs:252:5");

    let events = get_events();
    assert_eq!(events.len(), 3);

    unsafe {
        let val = std::ptr::read(ptr);
        std::ptr::write(ptr, val + 1);
    }
    assert_eq!(x, 43);
}

#[test]
#[serial]
fn test_ptr_swap() {
    reset();

    let mut x = 1;
    let mut y = 2;
    let ptr_x = track_raw_ptr_mut("ptr_x", 1, "*mut i32", "test.rs:260:5", &mut x as *mut i32);
    let ptr_y = track_raw_ptr_mut("ptr_y", 2, "*mut i32", "test.rs:261:5", &mut y as *mut i32);
    track_unsafe_fn_call("ptr::swap", "test.rs:262:5");

    let events = get_events();
    assert_eq!(events.len(), 3);

    unsafe {
        std::ptr::swap(ptr_x, ptr_y);
    }
    assert_eq!(x, 2);
    assert_eq!(y, 1);
}

#[test]
#[serial]
fn test_volatile_operations() {
    reset();

    let mut x = 42;
    let ptr = track_raw_ptr_mut("ptr", 1, "*mut i32", "test.rs:270:5", &mut x as *mut i32);
    track_unsafe_fn_call("ptr::read_volatile", "test.rs:271:5");
    track_unsafe_fn_call("ptr::write_volatile", "test.rs:272:5");

    let events = get_events();
    assert_eq!(events.len(), 3);

    unsafe {
        let val = std::ptr::read_volatile(ptr);
        std::ptr::write_volatile(ptr, val + 1);
    }
}

#[test]
#[serial]
fn test_unaligned_access() {
    reset();

    let arr = [1u8, 2, 3, 4];
    let ptr = track_raw_ptr("ptr", 1, "*const u8", "test.rs:280:5", arr.as_ptr());
    track_unsafe_fn_call("ptr::read_unaligned", "test.rs:281:5");

    let events = get_events();
    assert_eq!(events.len(), 2);

    unsafe {
        let _val = std::ptr::read_unaligned(ptr as *const u32);
    }
}

#[test]
#[serial]
fn test_ptr_copy() {
    reset();

    let src = [1, 2, 3, 4, 5];
    let mut dst = [0; 5];
    let src_ptr = track_raw_ptr("src_ptr", 1, "*const i32", "test.rs:290:5", src.as_ptr());
    let dst_ptr = track_raw_ptr_mut("dst_ptr", 2, "*mut i32", "test.rs:291:5", dst.as_mut_ptr());
    track_unsafe_fn_call("ptr::copy", "test.rs:292:5");

    let events = get_events();
    assert_eq!(events.len(), 3);

    unsafe {
        std::ptr::copy(src_ptr, dst_ptr, 5);
    }
    assert_eq!(dst, src);
}

#[test]
#[serial]
fn test_ptr_copy_nonoverlapping() {
    reset();

    let src = [1, 2, 3];
    let mut dst = [0; 3];
    track_unsafe_fn_call("ptr::copy_nonoverlapping", "test.rs:300:5");

    let events = get_events();
    assert_eq!(events.len(), 1);

    unsafe {
        std::ptr::copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr(), 3);
    }
    assert_eq!(dst, src);
}

#[test]
#[serial]
fn test_mem_zeroed() {
    reset();

    track_unsafe_fn_call("mem::zeroed", "test.rs:310:5");

    let events = get_events();
    assert_eq!(events.len(), 1);

    unsafe {
        let _x: i32 = std::mem::zeroed();
    }
}

#[test]
#[serial]
fn test_mem_uninitialized() {
    reset();

    track_unsafe_fn_call("mem::uninitialized", "test.rs:320:5");

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
#[serial]
fn test_assume() {
    reset();

    track_unsafe_fn_call("hint::unreachable_unchecked", "test.rs:330:5");

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
#[serial]
fn test_mixed_unsafe_operations() {
    reset();

    track_unsafe_block_enter(1, "test.rs:340:5");
    let x = 42;
    let ptr = track_raw_ptr("ptr", 2, "*const i32", "test.rs:341:9", &x as *const i32);
    track_raw_ptr_deref(2, "test.rs:342:9", false);
    track_transmute("i32", "u32", "test.rs:343:9");
    track_ffi_call("external_fn", "test.rs:344:9");
    track_unsafe_block_exit(1, "test.rs:345:5");

    let events = get_events();
    assert_eq!(events.len(), 6);
    assert!(events.iter().all(|e| e.is_unsafe()));

    unsafe {
        let _ = *ptr;
    }
}

#[test]
#[serial]
fn test_raw_ptr_arithmetic() {
    reset();

    let arr = [1, 2, 3, 4, 5];
    let ptr = track_raw_ptr("ptr", 1, "*const i32", "test.rs:350:5", arr.as_ptr());
    track_raw_ptr_deref(1, "test.rs:351:9", false);
    track_raw_ptr_deref(1, "test.rs:352:9", false);

    let events = get_events();
    assert_eq!(events.len(), 3);

    unsafe {
        assert_eq!(*ptr, 1);
        assert_eq!(*ptr.add(2), 3);
    }
}

#[test]
#[serial]
fn test_ptr_as_ref() {
    reset();

    let x = 42;
    let ptr = track_raw_ptr("ptr", 1, "*const i32", "test.rs:360:5", &x as *const i32);
    track_unsafe_fn_call("ptr::as_ref", "test.rs:361:5");

    let events = get_events();
    assert_eq!(events.len(), 2);

    unsafe {
        let _ref = ptr.as_ref();
    }
}

#[test]
#[serial]
fn test_ptr_as_mut() {
    reset();

    let mut x = 42;
    let ptr = track_raw_ptr_mut("ptr", 1, "*mut i32", "test.rs:370:5", &mut x as *mut i32);
    track_unsafe_fn_call("ptr::as_mut", "test.rs:371:5");

    let events = get_events();
    assert_eq!(events.len(), 2);

    unsafe {
        if let Some(r) = ptr.as_mut() {
            *r = 100;
        }
    }
    assert_eq!(x, 100);
}

#[test]
#[serial]
fn test_box_from_raw() {
    reset();

    let x = Box::new(42);
    let ptr = Box::into_raw(x);
    let _ptr_tracked = track_raw_ptr_mut("ptr", 1, "*mut i32", "test.rs:380:5", ptr);
    track_unsafe_fn_call("Box::from_raw", "test.rs:381:5");

    let events = get_events();
    assert_eq!(events.len(), 2);

    unsafe {
        let _x = Box::from_raw(ptr);
    }
}

#[test]
#[serial]
fn test_rc_from_raw() {
    reset();

    use std::rc::Rc;
    let x = Rc::new(42);
    let ptr = Rc::into_raw(x);
    let _ptr_tracked = track_raw_ptr("ptr", 1, "*const i32", "test.rs:390:5", ptr);
    track_unsafe_fn_call("Rc::from_raw", "test.rs:391:5");

    let events = get_events();
    assert_eq!(events.len(), 2);

    unsafe {
        let _x = Rc::from_raw(ptr);
    }
}

#[test]
#[serial]
fn test_arc_from_raw() {
    reset();

    use std::sync::Arc;
    let x = Arc::new(42);
    let ptr = Arc::into_raw(x);
    let _ptr_tracked = track_raw_ptr("ptr", 1, "*const i32", "test.rs:400:5", ptr);
    track_unsafe_fn_call("Arc::from_raw", "test.rs:401:5");

    let events = get_events();
    assert_eq!(events.len(), 2);

    unsafe {
        let _x = Arc::from_raw(ptr);
    }
}

#[test]
#[serial]
fn test_string_from_raw_parts() {
    reset();

    track_unsafe_fn_call("String::from_raw_parts", "test.rs:410:5");

    let events = get_events();
    assert_eq!(events.len(), 1);

    let s = String::from("hello");
    let ptr = s.as_ptr();
    let len = s.len();
    let cap = s.capacity();
    std::mem::forget(s);

    unsafe {
        let _s = String::from_raw_parts(ptr as *mut u8, len, cap);
    }
}

#[test]
#[serial]
fn test_vec_from_raw_parts() {
    reset();

    track_unsafe_fn_call("Vec::from_raw_parts", "test.rs:420:5");

    let events = get_events();
    assert_eq!(events.len(), 1);

    let v = vec![1, 2, 3];
    let ptr = v.as_ptr();
    let len = v.len();
    let cap = v.capacity();
    std::mem::forget(v);

    unsafe {
        let _v = Vec::from_raw_parts(ptr as *mut i32, len, cap);
    }
}

#[test]
#[serial]
fn test_ptr_metadata() {
    reset();

    let x = 42;
    let ptr = track_raw_ptr("ptr", 1, "*const i32", "test.rs:430:5", &x as *const i32);

    let events = get_events();
    assert_eq!(events.len(), 1);

    let _ = ptr;
}

#[test]
#[serial]
fn test_dangling_ptr() {
    reset();

    let _ptr = track_raw_ptr(
        "dangling",
        1,
        "*const i32",
        "test.rs:440:5",
        std::ptr::NonNull::dangling().as_ptr() as *const i32,
    );

    let events = get_events();
    assert_eq!(events.len(), 1);
}
