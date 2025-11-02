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

// ============================================================================
// ADVANCED EDGE CASES
// ============================================================================

#[test]
#[serial]
fn test_ptr_alignment_check() {
    reset();

    let x = 42i32;
    let ptr = track_raw_ptr("ptr", 1, "*const i32", "test.rs:450:5", &x as *const i32);
    track_unsafe_fn_call("ptr::is_aligned", "test.rs:451:5");

    let events = get_events();
    assert_eq!(events.len(), 2);

    assert!(ptr.is_aligned());
}

#[test]
#[serial]
fn test_ptr_wrapping_offset() {
    reset();

    let arr = [1, 2, 3, 4, 5];
    let ptr = track_raw_ptr("ptr", 1, "*const i32", "test.rs:460:5", arr.as_ptr());
    track_raw_ptr_deref(1, "test.rs:461:9", false);

    let events = get_events();
    assert_eq!(events.len(), 2);

    unsafe {
        let ptr2 = ptr.wrapping_offset(2);
        assert_eq!(*ptr2, 3);
    }
}

#[test]
#[serial]
fn test_ptr_byte_offset() {
    reset();

    let arr = [1u32, 2, 3];
    let ptr = track_raw_ptr("ptr", 1, "*const u32", "test.rs:470:5", arr.as_ptr());

    let events = get_events();
    assert_eq!(events.len(), 1);

    unsafe {
        let ptr2 = ptr.byte_offset(4);
        assert_eq!(*ptr2, 2);
    }
}

#[test]
#[serial]
fn test_ptr_sub() {
    reset();

    let arr = [1, 2, 3, 4, 5];
    let ptr = track_raw_ptr("ptr", 1, "*const i32", "test.rs:480:5", unsafe {
        arr.as_ptr().add(3)
    });

    let events = get_events();
    assert_eq!(events.len(), 1);

    unsafe {
        let ptr2 = ptr.sub(2);
        assert_eq!(*ptr2, 2);
    }
}

#[test]
#[serial]
fn test_ptr_offset_from() {
    reset();

    let arr = [1, 2, 3, 4, 5];
    let ptr1 = track_raw_ptr("ptr1", 1, "*const i32", "test.rs:490:5", arr.as_ptr());
    let ptr2 = track_raw_ptr("ptr2", 2, "*const i32", "test.rs:491:5", unsafe {
        arr.as_ptr().add(3)
    });

    let events = get_events();
    assert_eq!(events.len(), 2);

    unsafe {
        let offset = ptr2.offset_from(ptr1);
        assert_eq!(offset, 3);
    }
}

#[test]
#[serial]
fn test_ptr_align_offset() {
    reset();

    let x = 42u8;
    let ptr = track_raw_ptr("ptr", 1, "*const u8", "test.rs:500:5", &x as *const u8);

    let events = get_events();
    assert_eq!(events.len(), 1);

    let _offset = ptr.align_offset(4);
}

#[test]
#[serial]
fn test_ptr_map_addr() {
    reset();

    let x = 42;
    let ptr = track_raw_ptr("ptr", 1, "*const i32", "test.rs:510:5", &x as *const i32);

    let events = get_events();
    assert_eq!(events.len(), 1);

    let _new_ptr = ptr.map_addr(|addr| addr);
}

#[test]
#[serial]
fn test_ptr_with_addr() {
    reset();

    let x = 42;
    let ptr = track_raw_ptr("ptr", 1, "*const i32", "test.rs:520:5", &x as *const i32);

    let events = get_events();
    assert_eq!(events.len(), 1);

    let addr = ptr as usize;
    let _new_ptr = ptr.with_addr(addr);
}

#[test]
#[serial]
fn test_slice_from_raw_parts_mut() {
    reset();

    let mut arr = [1, 2, 3, 4, 5];
    let ptr = track_raw_ptr_mut("ptr", 1, "*mut i32", "test.rs:530:5", arr.as_mut_ptr());
    track_unsafe_fn_call("slice::from_raw_parts_mut", "test.rs:531:5");

    let events = get_events();
    assert_eq!(events.len(), 2);

    unsafe {
        let slice = std::slice::from_raw_parts_mut(ptr, 5);
        slice[0] = 10;
    }
    assert_eq!(arr[0], 10);
}

#[test]
#[serial]
fn test_ptr_replace() {
    reset();

    let mut x = 42;
    let ptr = track_raw_ptr_mut("ptr", 1, "*mut i32", "test.rs:540:5", &mut x as *mut i32);
    track_unsafe_fn_call("ptr::replace", "test.rs:541:5");

    let events = get_events();
    assert_eq!(events.len(), 2);

    unsafe {
        let old = std::ptr::replace(ptr, 100);
        assert_eq!(old, 42);
    }
    assert_eq!(x, 100);
}

#[test]
#[serial]
fn test_ptr_drop_in_place() {
    reset();

    let x = Box::new(42);
    let ptr = Box::into_raw(x);
    let _tracked = track_raw_ptr_mut("ptr", 1, "*mut i32", "test.rs:550:5", ptr);
    track_unsafe_fn_call("ptr::drop_in_place", "test.rs:551:5");

    let events = get_events();
    assert_eq!(events.len(), 2);

    unsafe {
        // Drop the value and deallocate
        std::ptr::drop_in_place(ptr);
        std::alloc::dealloc(ptr as *mut u8, std::alloc::Layout::new::<i32>());
    }
}

#[test]
#[serial]
fn test_mem_size_of_val() {
    reset();

    let x = 42;
    let ptr = track_raw_ptr("ptr", 1, "*const i32", "test.rs:560:5", &x as *const i32);
    track_unsafe_fn_call("mem::size_of_val", "test.rs:561:5");

    let events = get_events();
    assert_eq!(events.len(), 2);

    // Use safe size_of_val with reference instead of raw pointer
    let size = std::mem::size_of_val(&x);
    assert_eq!(size, 4);

    let _ = ptr;
}

#[test]
#[serial]
fn test_mem_align_of_val() {
    reset();

    let x = 42;
    let ptr = track_raw_ptr("ptr", 1, "*const i32", "test.rs:570:5", &x as *const i32);
    track_unsafe_fn_call("mem::align_of_val", "test.rs:571:5");

    let events = get_events();
    assert_eq!(events.len(), 2);

    // Use safe align_of_val with reference instead of raw pointer
    let align = std::mem::align_of_val(&x);
    assert_eq!(align, 4);

    let _ = ptr;
}

#[test]
#[serial]
fn test_mem_forget_with_ptr() {
    reset();

    let x = Box::new(42);
    let ptr = Box::into_raw(x);
    let _ptr_tracked = track_raw_ptr_mut("ptr", 1, "*mut i32", "test.rs:580:5", ptr);
    track_unsafe_fn_call("mem::forget", "test.rs:581:5");

    let events = get_events();
    assert_eq!(events.len(), 2);

    unsafe {
        let x = Box::from_raw(ptr);
        std::mem::forget(x);
    }
}

#[test]
#[serial]
fn test_transmute_copy() {
    reset();

    track_transmute("[u8; 4]", "[i8; 4]", "test.rs:590:5");

    let events = get_events();
    assert_eq!(events.len(), 1);

    let bytes: [u8; 4] = [1, 2, 3, 4];
    let signed: [i8; 4] = unsafe { std::mem::transmute_copy(&bytes) };
    assert_eq!(signed[0], 1);
}

#[test]
#[serial]
fn test_transmute_lifetime() {
    reset();

    track_transmute("&'a str", "&'static str", "test.rs:600:5");

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
#[serial]
fn test_transmute_reference_to_ptr() {
    reset();

    let x = 42;
    track_transmute("&i32", "*const i32", "test.rs:610:5");

    let events = get_events();
    assert_eq!(events.len(), 1);

    // Use cast instead of transmute
    let _ptr: *const i32 = &x as *const i32;
}

#[test]
#[serial]
fn test_transmute_function_pointer() {
    reset();

    track_transmute("fn(i32) -> i32", "usize", "test.rs:620:5");

    let events = get_events();
    assert_eq!(events.len(), 1);

    fn add_one(x: i32) -> i32 {
        x + 1
    }
    // Use cast instead of transmute
    let _addr: usize = add_one as fn(i32) -> i32 as usize;
}

#[test]
#[serial]
fn test_union_with_drop() {
    reset();

    track_union_field_access("UnionWithDrop", "value", "test.rs:630:5");

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
#[serial]
fn test_union_copy_semantics() {
    reset();

    track_union_field_access("CopyUnion", "int_val", "test.rs:640:5");
    track_union_field_access("CopyUnion", "float_val", "test.rs:641:5");

    let events = get_events();
    assert_eq!(events.len(), 2);
}

#[test]
#[serial]
fn test_ptr_provenance() {
    reset();

    let x = 42;
    let ptr1 = track_raw_ptr("ptr1", 1, "*const i32", "test.rs:650:5", &x as *const i32);
    let addr = ptr1 as usize;
    let _ptr2 = track_raw_ptr("ptr2", 2, "*const i32", "test.rs:652:5", addr as *const i32);

    let events = get_events();
    assert_eq!(events.len(), 2);
}

#[test]
#[serial]
fn test_ptr_expose_addr() {
    reset();

    let x = 42;
    let ptr = track_raw_ptr("ptr", 1, "*const i32", "test.rs:660:5", &x as *const i32);

    let events = get_events();
    assert_eq!(events.len(), 1);

    // Use stable cast to usize instead of expose_addr
    let _addr = ptr as usize;
}

#[test]
#[serial]
fn test_ptr_from_exposed_addr() {
    reset();

    let x = 42;
    let ptr = &x as *const i32;
    let addr = ptr as usize;
    let _ptr2 = track_raw_ptr("ptr2", 1, "*const i32", "test.rs:670:5", addr as *const i32);

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
#[serial]
fn test_nonnull_ptr() {
    reset();

    let x = 42;
    let ptr = std::ptr::NonNull::new(&x as *const i32 as *mut i32).unwrap();
    let _tracked = track_raw_ptr_mut("ptr", 1, "*mut i32", "test.rs:680:5", ptr.as_ptr());

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
#[serial]
fn test_ptr_const_cast() {
    reset();

    let x = 42;
    let ptr_const = track_raw_ptr(
        "ptr_const",
        1,
        "*const i32",
        "test.rs:690:5",
        &x as *const i32,
    );
    let _ptr_mut = track_raw_ptr_mut(
        "ptr_mut",
        2,
        "*mut i32",
        "test.rs:691:5",
        ptr_const.cast_mut(),
    );

    let events = get_events();
    assert_eq!(events.len(), 2);
}

#[test]
#[serial]
fn test_ptr_mut_cast() {
    reset();

    let mut x = 42;
    let ptr_mut = track_raw_ptr_mut(
        "ptr_mut",
        1,
        "*mut i32",
        "test.rs:700:5",
        &mut x as *mut i32,
    );
    let _ptr_const = track_raw_ptr(
        "ptr_const",
        2,
        "*const i32",
        "test.rs:701:5",
        ptr_mut.cast_const(),
    );

    let events = get_events();
    assert_eq!(events.len(), 2);
}

#[test]
#[serial]
fn test_ptr_to_bits() {
    reset();

    let x = 42;
    let ptr = track_raw_ptr("ptr", 1, "*const i32", "test.rs:710:5", &x as *const i32);

    let events = get_events();
    assert_eq!(events.len(), 1);

    let _bits = ptr.addr();
}

#[test]
#[serial]
fn test_fat_ptr_slice() {
    reset();

    let arr = [1, 2, 3, 4, 5];
    let slice: &[i32] = &arr;
    let ptr = track_raw_ptr(
        "ptr",
        1,
        "*const [i32]",
        "test.rs:720:5",
        slice as *const [i32],
    );

    let events = get_events();
    assert_eq!(events.len(), 1);

    unsafe {
        let len = (*ptr).len();
        assert_eq!(len, 5);
    }
}

#[test]
#[serial]
fn test_fat_ptr_trait_object() {
    reset();

    #[allow(dead_code)]
    trait MyTrait {
        fn value(&self) -> i32;
    }
    #[allow(dead_code)]
    struct MyStruct(i32);
    impl MyTrait for MyStruct {
        fn value(&self) -> i32 {
            self.0
        }
    }

    let obj = MyStruct(42);
    let trait_obj: &dyn MyTrait = &obj;
    let _ptr = track_raw_ptr(
        "ptr",
        1,
        "*const dyn MyTrait",
        "test.rs:730:5",
        trait_obj as *const dyn MyTrait,
    );

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
#[serial]
fn test_cell_from_mut() {
    reset();

    let mut x = 42;
    let ptr = track_raw_ptr_mut("ptr", 1, "*mut i32", "test.rs:760:5", &mut x as *mut i32);
    track_unsafe_fn_call("Cell::from_mut", "test.rs:761:5");

    let events = get_events();
    assert_eq!(events.len(), 2);

    unsafe {
        let _cell = std::cell::Cell::from_mut(&mut *ptr);
    }
}

#[test]
#[serial]
fn test_unsafecell_raw_get() {
    reset();

    let cell = std::cell::UnsafeCell::new(42);
    let _ptr = track_raw_ptr_mut("ptr", 1, "*mut i32", "test.rs:770:5", cell.get());

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
#[serial]
fn test_maybeuninit_assume_init() {
    reset();

    track_unsafe_fn_call("MaybeUninit::assume_init", "test.rs:780:5");

    let events = get_events();
    assert_eq!(events.len(), 1);

    let x = std::mem::MaybeUninit::new(42);
    unsafe {
        let _val = x.assume_init();
    }
}

#[test]
#[serial]
fn test_ptr_write_bytes() {
    reset();

    let mut arr = [0u8; 5];
    let ptr = track_raw_ptr_mut("ptr", 1, "*mut u8", "test.rs:800:5", arr.as_mut_ptr());
    track_unsafe_fn_call("ptr::write_bytes", "test.rs:801:5");

    let events = get_events();
    assert_eq!(events.len(), 2);

    unsafe {
        std::ptr::write_bytes(ptr, 0xFF, 5);
    }
    assert_eq!(arr, [0xFF; 5]);
}

#[test]
#[serial]
fn test_intrinsics_copy() {
    reset();

    let src = [1, 2, 3];
    let mut dst = [0; 3];
    track_unsafe_fn_call("intrinsics::copy", "test.rs:810:5");

    let events = get_events();
    assert_eq!(events.len(), 1);

    unsafe {
        std::ptr::copy(src.as_ptr(), dst.as_mut_ptr(), 3);
    }
    assert_eq!(dst, src);
}

#[test]
#[serial]
fn test_atomic_from_ptr() {
    reset();

    let mut x = 42;
    let ptr = track_raw_ptr_mut("ptr", 1, "*mut i32", "test.rs:820:5", &mut x as *mut i32);
    track_unsafe_fn_call("AtomicI32::from_ptr", "test.rs:821:5");

    let events = get_events();
    assert_eq!(events.len(), 2);

    unsafe {
        let _atomic = std::sync::atomic::AtomicI32::from_ptr(ptr);
    }
}

#[test]
#[serial]
fn test_alloc_dealloc() {
    reset();

    track_unsafe_fn_call("alloc::alloc", "test.rs:840:5");
    track_unsafe_fn_call("alloc::dealloc", "test.rs:841:5");

    let events = get_events();
    assert_eq!(events.len(), 2);

    unsafe {
        let layout = std::alloc::Layout::from_size_align(4, 4).unwrap();
        let ptr = std::alloc::alloc(layout);
        if !ptr.is_null() {
            std::alloc::dealloc(ptr, layout);
        }
    }
}

#[test]
#[serial]
fn test_alloc_zeroed() {
    reset();

    track_unsafe_fn_call("alloc::alloc_zeroed", "test.rs:850:5");

    let events = get_events();
    assert_eq!(events.len(), 1);

    unsafe {
        let layout = std::alloc::Layout::from_size_align(4, 4).unwrap();
        let ptr = std::alloc::alloc_zeroed(layout);
        if !ptr.is_null() {
            std::alloc::dealloc(ptr, layout);
        }
    }
}

#[test]
#[serial]
fn test_realloc() {
    reset();

    track_unsafe_fn_call("alloc::realloc", "test.rs:860:5");

    let events = get_events();
    assert_eq!(events.len(), 1);

    unsafe {
        let layout = std::alloc::Layout::from_size_align(4, 4).unwrap();
        let ptr = std::alloc::alloc(layout);
        if !ptr.is_null() {
            let new_layout = std::alloc::Layout::from_size_align(8, 4).unwrap();
            let new_ptr = std::alloc::realloc(ptr, layout, 8);
            if !new_ptr.is_null() {
                std::alloc::dealloc(new_ptr, new_layout);
            }
        }
    }
}

#[test]
#[serial]
fn test_pin_new_unchecked() {
    reset();

    let x = 42;
    let ptr = track_raw_ptr("ptr", 1, "*const i32", "test.rs:880:5", &x as *const i32);
    track_unsafe_fn_call("Pin::new_unchecked", "test.rs:881:5");

    let events = get_events();
    assert_eq!(events.len(), 2);

    unsafe {
        let _pin = std::pin::Pin::new_unchecked(&x);
    }

    let _ = ptr;
}

#[test]
#[serial]
fn test_pin_get_unchecked_mut() {
    reset();

    let mut x = 42;
    track_unsafe_fn_call("Pin::get_unchecked_mut", "test.rs:890:5");

    let events = get_events();
    assert_eq!(events.len(), 1);

    let mut pin = std::pin::Pin::new(&mut x);
    unsafe {
        let _r = std::pin::Pin::get_unchecked_mut(pin.as_mut());
    }
}

#[test]
#[serial]
fn test_manually_drop_take() {
    reset();

    track_unsafe_fn_call("ManuallyDrop::take", "test.rs:900:5");

    let events = get_events();
    assert_eq!(events.len(), 1);

    let mut x = std::mem::ManuallyDrop::new(Box::new(42));
    unsafe {
        let _val = std::mem::ManuallyDrop::take(&mut x);
    }
}

#[test]
#[serial]
fn test_manually_drop_drop() {
    reset();

    track_unsafe_fn_call("ManuallyDrop::drop", "test.rs:910:5");

    let events = get_events();
    assert_eq!(events.len(), 1);

    let mut x = std::mem::ManuallyDrop::new(Box::new(42));
    unsafe {
        std::mem::ManuallyDrop::drop(&mut x);
    }
}

#[test]
#[serial]
fn test_ptr_guaranteed_eq() {
    reset();

    let x = 42;
    let ptr1 = track_raw_ptr("ptr1", 1, "*const i32", "test.rs:940:5", &x as *const i32);
    let ptr2 = track_raw_ptr("ptr2", 2, "*const i32", "test.rs:941:5", &x as *const i32);

    let events = get_events();
    assert_eq!(events.len(), 2);

    assert!(std::ptr::eq(ptr1, ptr2));
}

#[test]
#[serial]
fn test_ptr_guaranteed_ne() {
    reset();

    let x = 42;
    let y = 43;
    let ptr1 = track_raw_ptr("ptr1", 1, "*const i32", "test.rs:950:5", &x as *const i32);
    let ptr2 = track_raw_ptr("ptr2", 2, "*const i32", "test.rs:951:5", &y as *const i32);

    let events = get_events();
    assert_eq!(events.len(), 2);

    assert!(!std::ptr::eq(ptr1, ptr2));
}

#[test]
#[serial]
fn test_ptr_hash() {
    reset();

    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let x = 42;
    let ptr = track_raw_ptr("ptr", 1, "*const i32", "test.rs:960:5", &x as *const i32);

    let events = get_events();
    assert_eq!(events.len(), 1);

    let mut hasher = DefaultHasher::new();
    ptr.hash(&mut hasher);
    let _hash = hasher.finish();
}

#[test]
#[serial]
fn test_multiple_unsafe_contexts() {
    reset();

    track_unsafe_block_enter(1, "test.rs:970:5");
    let x = 42;
    let ptr = track_raw_ptr("ptr", 2, "*const i32", "test.rs:971:9", &x as *const i32);

    track_unsafe_block_enter(3, "test.rs:973:9");
    track_raw_ptr_deref(2, "test.rs:974:13", false);
    track_transmute("i32", "u32", "test.rs:975:13");
    track_unsafe_block_exit(3, "test.rs:976:9");

    track_ffi_call("external", "test.rs:978:9");
    track_unsafe_block_exit(1, "test.rs:979:5");

    let events = get_events();
    assert_eq!(events.len(), 8);

    unsafe {
        let _ = *ptr;
    }
}
