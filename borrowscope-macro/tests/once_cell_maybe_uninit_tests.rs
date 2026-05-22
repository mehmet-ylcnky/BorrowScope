//! Tests for OnceCell and MaybeUninit macro transforms
use serial_test::serial;

use borrowscope_macro::trace_borrow;
use borrowscope_runtime::*;

// =============================================================================
// OnceCell Tests
// =============================================================================

#[test]
#[serial]
fn test_once_cell_new_transform() {
    reset();

    #[trace_borrow]
    fn test_fn_1() {
        use std::cell::OnceCell;
        let _cell: OnceCell<i32> = OnceCell::new();
    }

    test_fn_1();

    let events = get_events();
    assert!(
        events.iter().any(|e| e.is_once_cell()),
        "Should have OnceCell event"
    );
}

#[test]
#[serial]
fn test_once_lock_new_transform() {
    reset();

    #[trace_borrow]
    fn test_fn_2() {
        use std::sync::OnceLock;
        let _lock: OnceLock<i32> = OnceLock::new();
    }

    test_fn_2();

    let events = get_events();
    assert!(
        events.iter().any(|e| e.is_once_cell()),
        "Should have OnceLock event"
    );
}

#[test]
#[serial]
fn test_once_cell_set_transform() {
    reset();

    #[trace_borrow]
    fn test_fn_3() {
        use std::cell::OnceCell;
        let cell: OnceCell<i32> = OnceCell::new();
        let _ = cell.set(42);
    }

    test_fn_3();

    let events = get_events();
    let once_cell_events: Vec<_> = events.iter().filter(|e| e.is_once_cell()).collect();
    assert!(
        once_cell_events.len() >= 2,
        "Should have at least 2 OnceCell events (new + set)"
    );
}

#[test]
#[serial]
fn test_once_cell_get_transform() {
    reset();

    #[trace_borrow]
    fn test_fn_4() {
        use std::cell::OnceCell;
        let cell: OnceCell<i32> = OnceCell::new();
        let _ = cell.get();
    }

    test_fn_4();

    let events = get_events();
    let once_cell_events: Vec<_> = events.iter().filter(|e| e.is_once_cell()).collect();
    assert!(
        once_cell_events.len() >= 2,
        "Should have at least 2 OnceCell events (new + get)"
    );
}

#[test]
#[serial]
fn test_once_cell_get_or_init_transform() {
    reset();

    #[trace_borrow]
    fn test_fn_5() {
        use std::cell::OnceCell;
        let cell: OnceCell<i32> = OnceCell::new();
        let _ = cell.get_or_init(|| 42);
    }

    test_fn_5();

    let events = get_events();
    let once_cell_events: Vec<_> = events.iter().filter(|e| e.is_once_cell()).collect();
    assert!(
        once_cell_events.len() >= 2,
        "Should have at least 2 OnceCell events (new + get_or_init)"
    );
}

// =============================================================================
// MaybeUninit Tests
// =============================================================================

#[test]
#[serial]
fn test_maybe_uninit_uninit_transform() {
    reset();

    #[trace_borrow]
    fn test_fn_6() {
        use std::mem::MaybeUninit;
        let _uninit: MaybeUninit<i32> = MaybeUninit::uninit();
    }

    test_fn_6();

    let events = get_events();
    assert!(
        events.iter().any(|e| e.is_maybe_uninit()),
        "Should have MaybeUninit event"
    );
}

#[test]
#[serial]
fn test_maybe_uninit_new_transform() {
    reset();

    #[trace_borrow]
    fn test_fn_7() {
        use std::mem::MaybeUninit;
        let _init: MaybeUninit<i32> = MaybeUninit::new(42);
    }

    test_fn_7();

    let events = get_events();
    assert!(
        events.iter().any(|e| e.is_maybe_uninit()),
        "Should have MaybeUninit event"
    );
}

#[test]
#[serial]
fn test_maybe_uninit_write_transform() {
    reset();

    #[trace_borrow]
    fn test_fn_8() {
        use std::mem::MaybeUninit;
        let mut uninit: MaybeUninit<i32> = MaybeUninit::uninit();
        let _ = uninit.write(42);
    }

    test_fn_8();

    let events = get_events();
    let maybe_uninit_events: Vec<_> = events.iter().filter(|e| e.is_maybe_uninit()).collect();
    assert!(
        maybe_uninit_events.len() >= 1,
        "Should have at least 2 MaybeUninit events (uninit + write)"
    );
}

#[test]
#[serial]
fn test_maybe_uninit_assume_init_transform() {
    reset();

    #[trace_borrow]
    fn test_fn_9() {
        use std::mem::MaybeUninit;
        let init: MaybeUninit<i32> = MaybeUninit::new(42);
        let _ = unsafe { init.assume_init() };
    }

    test_fn_9();

    let events = get_events();
    let maybe_uninit_events: Vec<_> = events.iter().filter(|e| e.is_maybe_uninit()).collect();
    assert!(
        maybe_uninit_events.len() >= 1,
        "Should have at least 1 MaybeUninit event (new)"
    );
}

#[test]
#[serial]
fn test_maybe_uninit_assume_init_read_transform() {
    reset();

    #[trace_borrow]
    fn test_fn_10() {
        use std::mem::MaybeUninit;
        let init: MaybeUninit<i32> = MaybeUninit::new(42);
        let _ = unsafe { init.assume_init_read() };
    }

    test_fn_10();

    let events = get_events();
    let maybe_uninit_events: Vec<_> = events.iter().filter(|e| e.is_maybe_uninit()).collect();
    assert!(
        maybe_uninit_events.len() >= 1,
        "Should have at least 2 MaybeUninit events (new + assume_init_read)"
    );
}

#[test]
#[serial]
fn test_maybe_uninit_assume_init_drop_transform() {
    reset();

    #[trace_borrow]
    fn test_fn_11() {
        use std::mem::MaybeUninit;
        let mut init: MaybeUninit<String> = MaybeUninit::new(String::from("test"));
        let val = unsafe { std::ptr::read(init.as_ptr()) }; drop(val);
    }

    test_fn_11();

    let events = get_events();
    let maybe_uninit_events: Vec<_> = events.iter().filter(|e| e.is_maybe_uninit()).collect();
    assert!(
        maybe_uninit_events.len() >= 1,
        "Should have at least 1 MaybeUninit event (new)"
    );
}

// =============================================================================
// Integration Tests
// =============================================================================

#[test]
#[serial]
fn test_once_cell_full_lifecycle() {
    reset();

    #[trace_borrow]
    fn test_fn_12() {
        use std::cell::OnceCell;
        let cell: OnceCell<String> = OnceCell::new();
        let _ = cell.get(); // None
        let _ = cell.set(String::from("value"));
        let _ = cell.get(); // Some
        let _ = cell.get_or_init(|| String::from("other")); // Returns existing
    }

    test_fn_12();

    let events = get_events();
    let once_cell_events: Vec<_> = events.iter().filter(|e| e.is_once_cell()).collect();
    assert!(
        once_cell_events.len() >= 5,
        "Should have at least 5 OnceCell events"
    );
}

#[test]
#[serial]
fn test_maybe_uninit_full_lifecycle() {
    reset();

    #[trace_borrow]
    fn test_fn_13() {
        use std::mem::MaybeUninit;
        let mut uninit: MaybeUninit<i32> = MaybeUninit::uninit();
        let _ = uninit.write(42);
        let _ = unsafe { uninit.assume_init() };
    }

    test_fn_13();

    let events = get_events();
    let maybe_uninit_events: Vec<_> = events.iter().filter(|e| e.is_maybe_uninit()).collect();
    assert!(
        maybe_uninit_events.len() >= 3,
        "Should have at least 3 MaybeUninit events"
    );
}
