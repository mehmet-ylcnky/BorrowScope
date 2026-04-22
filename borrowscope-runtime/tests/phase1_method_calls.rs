//! Phase 1: Method Call Semantic Dispatch — Runtime Verification
//! Tests all track_* functions that correspond to semantic_op dispatch patterns.
//! Uses runtime directly (no macro) to verify event generation.

use borrowscope_runtime::*;

// === Cell operations ===

#[test]
fn test_phase1_cell_set() {
    reset();
    let c = std::cell::Cell::new(0);
    track_cell_set("cell_c", "test:1");
    c.set(42);
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::CellSet { .. })));
}

#[test]
fn test_phase1_cell_get() {
    reset();
    let c = std::cell::Cell::new(42);
    let _v = track_cell_get("cell_c", "test:1", c.get());
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::CellGet { .. })));
}

// === RefCell operations ===

#[test]
fn test_phase1_refcell_borrow() {
    reset();
    let r = std::cell::RefCell::new(String::from("hello"));
    let _g = track_refcell_borrow("borrow_0", "refcell_r", "test:1", r.borrow());
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::RefCellBorrow { .. })));
}

#[test]
fn test_phase1_refcell_borrow_mut() {
    reset();
    let r = std::cell::RefCell::new(42);
    let _g = track_refcell_borrow_mut("borrow_0", "refcell_r", "test:1", r.borrow_mut());
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::RefCellBorrow { is_mutable: true, .. })),
        "RefCell::borrow_mut should produce RefCellBorrow {{ is_mutable: true }}");
}

// === Mutex/RwLock lock operations ===

#[test]
fn test_phase1_mutex_lock() {
    reset();
    let m = std::sync::Mutex::new(0);
    track_lock(1, "mutex_m", "mutex", "test:1");
    let _g = m.lock().unwrap();
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::Lock { .. })));
}

#[test]
fn test_phase1_rwlock_read() {
    reset();
    let r = std::sync::RwLock::new(0);
    track_lock(1, "rwlock_r", "rwlock_read", "test:1");
    let _g = r.read().unwrap();
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::Lock { .. })));
}

#[test]
fn test_phase1_rwlock_write() {
    reset();
    let r = std::sync::RwLock::new(0);
    track_lock(1, "rwlock_r", "rwlock_write", "test:1");
    let _g = r.write().unwrap();
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::Lock { .. })));
}

// === Cow::to_mut ===

#[test]
fn test_phase1_cow_to_mut() {
    reset();
    track_cow_to_mut("cow_c", true, "test:1");
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::CowToMut { .. })));
}

// === OnceCell operations ===

#[test]
fn test_phase1_once_cell_set() {
    reset();
    let once = std::cell::OnceCell::new();
    let _ = track_once_cell_set("once_o", "test:1", once.set(42));
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::OnceCellSet { .. })));
}

#[test]
fn test_phase1_once_cell_get() {
    reset();
    let once = std::cell::OnceCell::new();
    let _ = once.set(42);
    let _ = track_once_cell_get("once_o", "test:1", once.get());
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::OnceCellGet { .. })));
}

// === MaybeUninit operations ===

#[test]
fn test_phase1_maybe_uninit_write() {
    reset();
    let mut mu = std::mem::MaybeUninit::<i32>::uninit();
    let _ = track_maybe_uninit_write("uninit_mu", "test:1", mu.write(42));
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::MaybeUninitWrite { .. })));
}

#[test]
fn test_phase1_maybe_uninit_assume_init() {
    reset();
    let mu = std::mem::MaybeUninit::<i32>::new(42);
    let _ = track_maybe_uninit_assume_init("uninit_mu", "test:1", unsafe { mu.assume_init() });
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::MaybeUninitAssumeInit { .. })));
}

// === Channel operations ===

#[test]
fn test_phase1_channel_send_recv() {
    reset();
    let (tx, rx) = std::sync::mpsc::channel();
    let _ = track_channel_send("sender_tx", "test:1", tx.send(42));
    let _ = track_channel_recv("receiver_rx", "test:1", rx.recv());
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::ChannelSend { .. })));
    assert!(events.iter().any(|e| matches!(e, Event::ChannelRecv { .. })));
}

#[test]
fn test_phase1_channel_try_recv() {
    reset();
    let (_tx, rx) = std::sync::mpsc::channel::<i32>();
    let _ = track_channel_try_recv("receiver_rx", "test:1", rx.try_recv());
    let events = get_events();
    // try_recv produces ChannelRecv event
    assert!(events.len() >= 1);
}

// === JoinHandle::join ===

#[test]
fn test_phase1_thread_join() {
    reset();
    let h = std::thread::spawn(|| 42);
    let _ = track_thread_join("thread_h", "test:1", h.join());
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::ThreadJoin { .. })));
}

// === Unwrap methods (5 variants) ===

#[test]
fn test_phase1_unwrap_all_variants() {
    reset();
    track_unwrap(1, "unwrap", "opt_a", "test:1");
    track_unwrap(2, "opt_b", "expect", "test:2");
    track_unwrap(3, "opt_c", "unwrap_or", "test:3");
    track_unwrap(4, "opt_d", "unwrap_or_else", "test:4");
    track_unwrap(5, "opt_e", "unwrap_or_default", "test:5");
    let events = get_events();
    let unwrap_count = events.iter().filter(|e| matches!(e, Event::Unwrap { .. })).count();
    assert_eq!(unwrap_count, 5, "All 5 unwrap variants should produce events");
}

// === Unsafe fn call ===

#[test]
fn test_phase1_unsafe_fn_call() {
    reset();
    track_unsafe_fn_call("dangerous_fn", "test:1");
    let events = get_events();
    assert!(events.iter().any(|e| matches!(e, Event::UnsafeFnCall { .. })));
}

// === Method call with receiver/result type ===

#[test]
fn test_phase1_method_call_metadata() {
    reset();
    track_method_call(1, "v::push", "test:1", "Vec<i32>", "()");
    let events = get_events();
    let call_event = events.iter().find(|e| matches!(e, Event::Call { .. }));
    assert!(call_event.is_some(), "track_method_call should produce Call event");
}
