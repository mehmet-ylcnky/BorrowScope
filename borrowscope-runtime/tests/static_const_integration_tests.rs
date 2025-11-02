//! Integration tests for static and const tracking
//!
//! These tests verify that static and const variables are properly tracked
//! including initialization, access patterns, and mutability.

use borrowscope_runtime::*;
use serial_test::serial;

#[test]
#[serial]
fn test_static_init() {
    reset();

    // Simulate static initialization
    let _value = track_static_init("GLOBAL_CONFIG", 1, "Config", false, 42);

    let events = get_events();
    assert_eq!(events.len(), 1);

    let event = &events[0];
    assert!(event.is_static());
    assert_eq!(event.var_name(), Some("GLOBAL_CONFIG"));
}

#[test]
#[serial]
fn test_static_mut_init() {
    reset();

    // Simulate static mut initialization
    let _value = track_static_init("COUNTER", 2, "i32", true, 0);

    let events = get_events();
    assert_eq!(events.len(), 1);

    let event = &events[0];
    assert!(event.is_static());
    assert_eq!(event.var_name(), Some("COUNTER"));
}

#[test]
#[serial]
fn test_static_read_access() {
    reset();

    // Simulate static read
    track_static_access(1, "GLOBAL_CONFIG", false, "test.rs:10:5");

    let events = get_events();
    assert_eq!(events.len(), 1);

    let event = &events[0];
    assert!(event.is_static());
    assert_eq!(event.var_name(), Some("GLOBAL_CONFIG"));
}

#[test]
#[serial]
fn test_static_write_access() {
    reset();

    // Simulate static mut write
    track_static_access(2, "COUNTER", true, "test.rs:15:9");

    let events = get_events();
    assert_eq!(events.len(), 1);

    let event = &events[0];
    assert!(event.is_static());
    assert_eq!(event.var_name(), Some("COUNTER"));
}

#[test]
#[serial]
fn test_static_multiple_accesses() {
    reset();

    // Simulate multiple accesses to static mut
    track_static_init("COUNTER", 1, "i32", true, 0);
    track_static_access(1, "COUNTER", true, "test.rs:20:9");
    track_static_access(1, "COUNTER", false, "test.rs:21:9");
    track_static_access(1, "COUNTER", true, "test.rs:22:9");

    let events = get_events();
    assert_eq!(events.len(), 4);

    // First event is init
    assert!(events[0].is_static());

    // Rest are accesses
    for event in &events[1..] {
        assert!(event.is_static());
        assert_eq!(event.var_name(), Some("COUNTER"));
    }
}

#[test]
#[serial]
fn test_const_eval() {
    reset();

    // Simulate const evaluation
    let _value = track_const_eval("MAX_SIZE", 1, "usize", "test.rs:5:1", 100);

    let events = get_events();
    assert_eq!(events.len(), 1);

    let event = &events[0];
    assert!(event.is_const());
    assert_eq!(event.var_name(), Some("MAX_SIZE"));
}

#[test]
#[serial]
fn test_const_multiple_uses() {
    reset();

    // Simulate const being used multiple times
    let _v1 = track_const_eval("PI", 1, "f64", "test.rs:10:5", std::f64::consts::PI);
    let _v2 = track_const_eval("PI", 1, "f64", "test.rs:11:5", std::f64::consts::PI);
    let _v3 = track_const_eval("PI", 1, "f64", "test.rs:12:5", std::f64::consts::PI);

    let events = get_events();
    assert_eq!(events.len(), 3);

    for event in &events {
        assert!(event.is_const());
        assert_eq!(event.var_name(), Some("PI"));
    }
}

#[test]
#[serial]
fn test_global_event_helpers() {
    reset();

    track_static_init("STATIC_VAR", 1, "i32", false, 42);
    track_const_eval("CONST_VAR", 2, "i32", "test.rs:1:1", 100);

    let events = get_events();
    assert_eq!(events.len(), 2);

    // Both should be global events
    assert!(events[0].is_global());
    assert!(events[1].is_global());

    // But only one is static, one is const
    assert!(events[0].is_static());
    assert!(!events[0].is_const());
    assert!(!events[1].is_static());
    assert!(events[1].is_const());
}

#[test]
#[serial]
fn test_static_with_complex_type() {
    reset();

    // Simulate static with complex type
    track_static_init("GLOBAL_MAP", 1, "HashMap<String, Vec<i32>>", false, ());

    let events = get_events();
    assert_eq!(events.len(), 1);

    let event = &events[0];
    assert!(event.is_static());
    assert_eq!(event.var_name(), Some("GLOBAL_MAP"));
}

#[test]
#[serial]
fn test_lazy_static_pattern() {
    reset();

    // Simulate lazy_static initialization on first access
    track_static_init(
        "LAZY_VALUE",
        1,
        "String",
        false,
        String::from("initialized"),
    );
    track_static_access(1, "LAZY_VALUE", false, "test.rs:30:5");

    let events = get_events();
    assert_eq!(events.len(), 2);

    assert!(events[0].is_static());
    assert!(events[1].is_static());
}

#[test]
#[serial]
fn test_static_atomic_pattern() {
    reset();

    // Simulate atomic static
    track_static_init("ATOMIC_COUNTER", 1, "AtomicUsize", false, ());
    track_static_access(1, "ATOMIC_COUNTER", false, "test.rs:40:5");
    track_static_access(1, "ATOMIC_COUNTER", true, "test.rs:41:5");
    track_static_access(1, "ATOMIC_COUNTER", false, "test.rs:42:5");

    let events = get_events();
    assert_eq!(events.len(), 4);

    for event in &events {
        assert!(event.is_static());
    }
}

#[test]
#[serial]
fn test_const_in_pattern_matching() {
    reset();

    // Simulate const used in pattern matching
    let _max = track_const_eval("MAX_RETRIES", 1, "u32", "test.rs:50:5", 3);

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_const());
}

#[test]
#[serial]
fn test_const_array_size() {
    reset();

    // Simulate const used as array size
    let _size = track_const_eval("BUFFER_SIZE", 1, "usize", "test.rs:60:5", 1024);

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_const());
}

#[test]
#[serial]
fn test_static_ref_pattern() {
    reset();

    // Simulate static reference
    track_static_init("EMPTY_STRING", 1, "&'static str", false, "");
    track_static_access(1, "EMPTY_STRING", false, "test.rs:70:5");

    let events = get_events();
    assert_eq!(events.len(), 2);

    for event in &events {
        assert!(event.is_static());
    }
}

#[test]
#[serial]
fn test_mixed_global_and_local() {
    reset();

    // Mix of global and local variables
    track_const_eval("MAX", 1, "i32", "test.rs:1:1", 100);
    track_new("local_var", 2);
    track_static_access(3, "GLOBAL", false, "test.rs:5:5");
    track_borrow("borrowed", "local_var");

    let events = get_events();
    assert_eq!(events.len(), 4);

    assert!(events[0].is_const());
    assert!(events[1].is_new());
    assert!(events[2].is_static());
    assert!(events[3].is_borrow());
}

#[test]
#[serial]
fn test_static_initialization_order() {
    reset();

    // Simulate multiple static initializations
    track_static_init("FIRST", 1, "i32", false, 1);
    track_static_init("SECOND", 2, "i32", false, 2);
    track_static_init("THIRD", 3, "i32", false, 3);

    let events = get_events();
    assert_eq!(events.len(), 3);

    // Verify order is preserved
    assert_eq!(events[0].var_name(), Some("FIRST"));
    assert_eq!(events[1].var_name(), Some("SECOND"));
    assert_eq!(events[2].var_name(), Some("THIRD"));
}

#[test]
#[serial]
fn test_const_fn_evaluation() {
    reset();

    // Simulate const fn evaluation
    let _result = track_const_eval("COMPUTED_VALUE", 1, "i32", "test.rs:100:5", 42);

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_const());
}

#[test]
#[serial]
fn test_static_mut_unsafe_pattern() {
    reset();

    // Simulate unsafe static mut access pattern
    track_static_init("UNSAFE_COUNTER", 1, "i32", true, 0);

    // Multiple unsafe accesses
    for i in 0..5 {
        track_static_access(1, "UNSAFE_COUNTER", true, &format!("test.rs:{}:9", 110 + i));
    }

    let events = get_events();
    assert_eq!(events.len(), 6); // 1 init + 5 accesses

    assert!(events[0].is_static());
    for event in &events[1..] {
        assert!(event.is_static());
    }
}

#[test]
#[serial]
fn test_thread_local_static() {
    reset();

    // Simulate thread_local static
    track_static_init("THREAD_LOCAL_VAR", 1, "RefCell<i32>", false, ());
    track_static_access(1, "THREAD_LOCAL_VAR", false, "test.rs:120:5");

    let events = get_events();
    assert_eq!(events.len(), 2);

    for event in &events {
        assert!(event.is_static());
    }
}

#[test]
#[serial]
fn test_const_generic_parameter() {
    reset();

    // Simulate const generic parameter
    let _n = track_const_eval("N", 1, "usize", "test.rs:130:5", 10);

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_const());
}

#[test]
#[serial]
fn test_static_lifetime_bound() {
    reset();

    // Simulate static lifetime bound
    track_static_init("STATIC_DATA", 1, "&'static [u8]", false, ());
    track_static_access(1, "STATIC_DATA", false, "test.rs:140:5");

    let events = get_events();
    assert_eq!(events.len(), 2);

    for event in &events {
        assert!(event.is_static());
    }
}

// ============================================================================
// ADVANCED EDGE CASES
// ============================================================================

#[test]
#[serial]
fn test_static_with_drop_impl() {
    reset();

    // Static with Drop implementation
    track_static_init("STATIC_WITH_DROP", 1, "Box<String>", false, ());
    track_static_access(1, "STATIC_WITH_DROP", false, "test.rs:200:5");

    let events = get_events();
    assert_eq!(events.len(), 2);
}

#[test]
#[serial]
fn test_const_in_generic_context() {
    reset();

    // Const used as generic parameter
    let _n = track_const_eval("SIZE", 1, "usize", "test.rs:210:5", 32);
    let _m = track_const_eval("SIZE", 1, "usize", "test.rs:211:5", 32);

    let events = get_events();
    assert_eq!(events.len(), 2);
    assert!(events.iter().all(|e| e.is_const()));
}

#[test]
#[serial]
fn test_static_with_interior_mutability() {
    reset();

    // Static with RefCell (interior mutability)
    track_static_init("STATIC_REFCELL", 1, "RefCell<Vec<i32>>", false, ());
    track_static_access(1, "STATIC_REFCELL", false, "test.rs:220:5");

    let events = get_events();
    assert!(events.len() >= 2);
    assert!(events[0].is_static());
}

#[test]
#[serial]
fn test_static_with_mutex() {
    reset();

    // Static Mutex pattern
    track_static_init("STATIC_MUTEX", 1, "Mutex<HashMap<String, i32>>", false, ());
    track_static_access(1, "STATIC_MUTEX", false, "test.rs:230:5");
    track_static_access(1, "STATIC_MUTEX", false, "test.rs:231:5");

    let events = get_events();
    assert_eq!(events.len(), 3);
    assert!(events.iter().all(|e| e.is_static()));
}

#[test]
#[serial]
fn test_static_with_rwlock() {
    reset();

    // Static RwLock pattern
    track_static_init("STATIC_RWLOCK", 1, "RwLock<Vec<String>>", false, ());
    track_static_access(1, "STATIC_RWLOCK", false, "test.rs:240:5");
    track_static_access(1, "STATIC_RWLOCK", true, "test.rs:241:5");

    let events = get_events();
    assert_eq!(events.len(), 3);
}

#[test]
#[serial]
fn test_const_in_match_arm() {
    reset();

    // Const used in match arm
    let _max = track_const_eval("MAX_RETRIES", 1, "u32", "test.rs:250:9", 5);
    let _min = track_const_eval("MIN_RETRIES", 2, "u32", "test.rs:251:9", 1);

    let events = get_events();
    assert_eq!(events.len(), 2);
    assert!(events.iter().all(|e| e.is_const()));
}

#[test]
#[serial]
fn test_const_in_type_annotation() {
    reset();

    // Const used in type annotation [T; N]
    let _size = track_const_eval("BUFFER_SIZE", 1, "usize", "test.rs:260:5", 4096);

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_const());
}

#[test]
#[serial]
fn test_static_slice_pattern() {
    reset();

    // Static slice
    track_static_init("STATIC_SLICE", 1, "&'static [u8]", false, ());
    track_static_access(1, "STATIC_SLICE", false, "test.rs:270:5");

    let events = get_events();
    assert_eq!(events.len(), 2);
}

#[test]
#[serial]
fn test_static_str_pattern() {
    reset();

    // Static string slice
    track_static_init("STATIC_STR", 1, "&'static str", false, ());
    track_static_access(1, "STATIC_STR", false, "test.rs:280:5");
    track_static_access(1, "STATIC_STR", false, "test.rs:281:5");

    let events = get_events();
    assert_eq!(events.len(), 3);
}

#[test]
#[serial]
fn test_const_fn_with_generics() {
    reset();

    // Const fn with generic parameters
    let _result = track_const_eval("COMPUTED", 1, "Option<i32>", "test.rs:290:5", Some(42));

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_const());
}

#[test]
#[serial]
fn test_static_once_cell_pattern() {
    reset();

    // OnceCell/OnceLock pattern
    track_static_init("ONCE_CELL", 1, "OnceLock<String>", false, ());
    track_static_access(1, "ONCE_CELL", false, "test.rs:300:5");
    track_static_access(1, "ONCE_CELL", true, "test.rs:301:5");

    let events = get_events();
    assert_eq!(events.len(), 3);
}

#[test]
#[serial]
fn test_static_arc_pattern() {
    reset();

    // Static Arc for shared ownership
    track_static_init("STATIC_ARC", 1, "Arc<Vec<i32>>", false, ());
    track_static_access(1, "STATIC_ARC", false, "test.rs:310:5");

    let events = get_events();
    assert!(events.len() >= 2);
    assert!(events[0].is_static());
}

#[test]
#[serial]
fn test_const_trait_associated() {
    reset();

    // Const associated with trait
    let _value = track_const_eval("Trait::CONST", 1, "i32", "test.rs:320:5", 100);

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_const());
}

#[test]
#[serial]
fn test_static_function_pointer() {
    reset();

    // Static function pointer
    track_static_init("STATIC_FN", 1, "fn(i32) -> i32", false, ());
    track_static_access(1, "STATIC_FN", false, "test.rs:330:5");

    let events = get_events();
    assert_eq!(events.len(), 2);
}

#[test]
#[serial]
fn test_const_in_where_clause() {
    reset();

    // Const in where clause
    let _n = track_const_eval("N", 1, "usize", "test.rs:340:5", 16);

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
#[serial]
fn test_static_mut_race_condition_pattern() {
    reset();

    // Simulate potential race condition
    track_static_init("SHARED_STATE", 1, "i32", true, 0);

    // Multiple threads accessing
    for i in 0..10 {
        track_static_access(
            1,
            "SHARED_STATE",
            i % 2 == 0,
            &format!("test.rs:{}:9", 350 + i),
        );
    }

    let events = get_events();
    assert_eq!(events.len(), 11); // 1 init + 10 accesses
}

#[test]
#[serial]
fn test_const_array_initialization() {
    reset();

    // Const used for array initialization
    let _size = track_const_eval("ARRAY_SIZE", 1, "usize", "test.rs:360:5", 100);
    let _default = track_const_eval("DEFAULT_VALUE", 2, "i32", "test.rs:361:5", 0);

    let events = get_events();
    assert_eq!(events.len(), 2);
}

#[test]
#[serial]
fn test_static_with_lifetime_elision() {
    reset();

    // Static with lifetime elision
    track_static_init("STATIC_REF", 1, "&str", false, ());
    track_static_access(1, "STATIC_REF", false, "test.rs:370:5");

    let events = get_events();
    assert_eq!(events.len(), 2);
}

#[test]
#[serial]
fn test_const_in_impl_block() {
    reset();

    // Const in impl block
    let _value = track_const_eval("Self::CONSTANT", 1, "usize", "test.rs:380:5", 42);

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
#[serial]
fn test_static_with_box() {
    reset();

    // Static with Box
    track_static_init("STATIC_BOX", 1, "Box<[i32]>", false, ());
    track_static_access(1, "STATIC_BOX", false, "test.rs:390:5");

    let events = get_events();
    assert_eq!(events.len(), 2);
}

#[test]
#[serial]
fn test_const_bit_flags() {
    reset();

    // Const bit flags
    let _read = track_const_eval("READ", 1, "u32", "test.rs:400:5", 0b0001);
    let _write = track_const_eval("WRITE", 2, "u32", "test.rs:401:5", 0b0010);
    let _exec = track_const_eval("EXEC", 3, "u32", "test.rs:402:5", 0b0100);

    let events = get_events();
    assert_eq!(events.len(), 3);
}

#[test]
#[serial]
fn test_static_with_phantom_data() {
    reset();

    // Static with PhantomData
    track_static_init("STATIC_PHANTOM", 1, "PhantomData<i32>", false, ());

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
#[serial]
fn test_const_tuple_destructuring() {
    reset();

    // Const tuple
    let _pair = track_const_eval("PAIR", 1, "(i32, i32)", "test.rs:410:5", (1, 2));

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
#[serial]
fn test_static_with_cell() {
    reset();

    // Static with Cell
    track_static_init("STATIC_CELL", 1, "Cell<i32>", false, ());
    track_static_access(1, "STATIC_CELL", false, "test.rs:420:5");
    track_cell_get("STATIC_CELL", "test.rs:421:5", 42);
    track_cell_set("STATIC_CELL", "test.rs:422:5");

    let events = get_events();
    assert!(events.len() >= 4);
}

#[test]
#[serial]
fn test_const_option_unwrap() {
    reset();

    // Const Option
    let _some = track_const_eval("SOME_VALUE", 1, "Option<i32>", "test.rs:430:5", Some(42));
    let _none = track_const_eval("NONE_VALUE", 2, "Option<i32>", "test.rs:431:5", None::<i32>);

    let events = get_events();
    assert_eq!(events.len(), 2);
}

#[test]
#[serial]
fn test_static_with_result() {
    reset();

    // Static with Result type
    track_static_init("STATIC_RESULT", 1, "Result<i32, String>", false, ());
    track_static_access(1, "STATIC_RESULT", false, "test.rs:440:5");

    let events = get_events();
    assert_eq!(events.len(), 2);
}

#[test]
#[serial]
fn test_const_range() {
    reset();

    // Const range
    let _start = track_const_eval("RANGE_START", 1, "usize", "test.rs:450:5", 0);
    let _end = track_const_eval("RANGE_END", 2, "usize", "test.rs:451:5", 100);

    let events = get_events();
    assert_eq!(events.len(), 2);
}

#[test]
#[serial]
fn test_static_with_nested_types() {
    reset();

    // Static with deeply nested types
    track_static_init(
        "NESTED",
        1,
        "HashMap<String, Vec<Option<Arc<Mutex<i32>>>>>",
        false,
        (),
    );

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
#[serial]
fn test_const_zero_sized_type() {
    reset();

    // Const ZST
    track_const_eval("UNIT", 1, "()", "test.rs:460:5", ());

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
#[serial]
fn test_static_initialization_panic_safety() {
    reset();

    // Static that might panic during init
    track_static_init("PANIC_STATIC", 1, "Result<i32, String>", false, ());

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
#[serial]
fn test_const_in_closure() {
    reset();

    // Const captured in closure
    let _max = track_const_eval("MAX", 1, "i32", "test.rs:470:5", 100);

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
#[serial]
fn test_static_with_dyn_trait() {
    reset();

    // Static with trait object
    track_static_init("STATIC_DYN", 1, "Box<dyn Fn() -> i32>", false, ());
    track_static_access(1, "STATIC_DYN", false, "test.rs:480:5");

    let events = get_events();
    assert_eq!(events.len(), 2);
}

#[test]
#[serial]
fn test_const_pointer_cast() {
    reset();

    // Const pointer
    let _ptr = track_const_eval("NULL_PTR", 1, "*const i32", "test.rs:490:5", 0);

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
#[serial]
fn test_static_with_unsync_type() {
    reset();

    // Static with !Sync type (wrapped in Mutex)
    track_static_init("UNSYNC_STATIC", 1, "Mutex<Rc<i32>>", false, ());

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
#[serial]
fn test_const_transmute_pattern() {
    reset();

    // Const with transmute-like pattern
    let _bytes = track_const_eval("BYTES", 1, "[u8; 4]", "test.rs:500:5", [0u8; 4]);

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
#[serial]
fn test_static_circular_dependency() {
    reset();

    // Simulate circular static dependency
    track_static_init("STATIC_A", 1, "i32", false, 1);
    track_static_init("STATIC_B", 2, "i32", false, 2);
    track_static_access(2, "STATIC_B", false, "test.rs:510:5");
    track_static_access(1, "STATIC_A", false, "test.rs:511:5");

    let events = get_events();
    assert_eq!(events.len(), 4);
}

#[test]
#[serial]
fn test_const_inline_asm_compatible() {
    reset();

    // Const compatible with inline asm
    let _offset = track_const_eval("OFFSET", 1, "usize", "test.rs:520:5", 8);

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
#[serial]
fn test_static_with_weak_ptr() {
    reset();

    // Static with Weak pointer
    track_static_init("STATIC_WEAK", 1, "Mutex<Weak<i32>>", false, ());
    track_static_access(1, "STATIC_WEAK", false, "test.rs:530:5");

    let events = get_events();
    assert_eq!(events.len(), 2);
}

#[test]
#[serial]
fn test_const_simd_compatible() {
    reset();

    // Const for SIMD operations
    let _lanes = track_const_eval("LANES", 1, "usize", "test.rs:540:5", 4);

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
#[serial]
fn test_static_promotion_pattern() {
    reset();

    // Static promotion from const
    track_const_eval("TEMP", 1, "&'static str", "test.rs:550:5", "promoted");
    track_static_init("PROMOTED", 2, "&'static str", false, ());

    let events = get_events();
    assert_eq!(events.len(), 2);
}

#[test]
#[serial]
fn test_const_discriminant() {
    reset();

    // Const enum discriminant
    let _disc = track_const_eval("DISCRIMINANT", 1, "u8", "test.rs:560:5", 0);

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
#[serial]
fn test_static_with_custom_allocator() {
    reset();

    // Static with custom allocator
    track_static_init("CUSTOM_ALLOC", 1, "Vec<i32, CustomAlloc>", false, ());

    let events = get_events();
    assert_eq!(events.len(), 1);
}
