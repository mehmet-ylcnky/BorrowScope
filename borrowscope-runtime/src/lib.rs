#![allow(dead_code)]
//! # BorrowScope Runtime
//!
//! A runtime tracking library for visualizing Rust's ownership and borrowing system.
//!
//! This crate captures ownership transfers, borrows, smart pointer operations,
//! concurrency primitives, and unsafe code as they happen at runtime, generating
//! structured event data for analysis and visualization.
//!
//! # Quick Start
//!
//! ```rust
//! use borrowscope_runtime::*;
//!
//! // Clear previous tracking data
//! reset();
//!
//! // Track variable creation
//! let data = track_new("data", vec![1, 2, 3]);
//!
//! // Track borrowing
//! let r = track_borrow("r", &data);
//! println!("{:?}", r);
//!
//! // Track drops
//! track_drop("r");
//! track_drop("data");
//!
//! // Export events as JSON
//! let events = get_events();
//! println!("{}", serde_json::to_string_pretty(&events).unwrap());
//! ```
//!
//! # Feature Flags
//!
//! - `track` - Enables runtime tracking. Without this feature, all tracking
//!   functions compile to no-ops with zero overhead.
//!
//! ```toml
//! [dependencies]
//! borrowscope-runtime = { version = "0.1", features = ["track"] }
//! ```
//!
//! # Modules
//!
//! - Core tracking functions (70+ functions for all ownership patterns)
//! - Event types and serialization (45+ event types)
//! - Ownership graph building and analysis
//! - JSON export utilities
//! - Lifetime analysis and timeline construction
//!
//! # Tracking Categories
//!
//! | Category | Functions |
//! |----------|-----------|
//! | Basic ownership | `track_new`, `track_borrow`, `track_borrow_mut`, `track_move`, `track_drop` |
//! | RAII guards | `track_new_guard`, `track_borrow_guard`, `track_borrow_mut_guard` |
//! | Smart pointers | `track_rc_new`, `track_rc_clone`, `track_arc_new`, `track_arc_clone` |
//! | Box | `track_box_new`, `track_box_into_raw`, `track_box_from_raw` |
//! | Weak references | `track_weak_new`, `track_weak_new_sync`, `track_weak_upgrade`, `track_weak_clone` |
//! | Pin | `track_pin_new`, `track_pin_into_inner` |
//! | Cow | `track_cow_borrowed`, `track_cow_owned`, `track_cow_to_mut` |
//! | Interior mutability | `track_refcell_*`, `track_cell_*` |
//! | OnceCell/OnceLock | `track_once_cell_new`, `track_once_lock_new`, `track_once_cell_set`, `track_once_cell_get` |
//! | MaybeUninit | `track_maybe_uninit_uninit`, `track_maybe_uninit_new`, `track_maybe_uninit_write`, `track_maybe_uninit_assume_init` |
//! | Threads | `track_thread_spawn`, `track_thread_join` |
//! | Channels | `track_channel`, `track_channel_send`, `track_channel_recv` |
//! | Lock guards | `track_lock_guard_acquire`, `track_lock_guard_drop` |
//! | Unsafe code | `track_raw_ptr*`, `track_unsafe_*`, `track_ffi_call`, `track_transmute` |
//!
//! # Smart Pointer Tracking
//!
//! Track all standard library smart pointers:
//!
//! ```rust
//! use borrowscope_runtime::*;
//! use std::rc::Rc;
//! use std::sync::Arc;
//!
//! reset();
//!
//! // Reference counting
//! let rc = track_rc_new("rc", Rc::new(42));
//! let rc2 = track_rc_clone("rc2", "rc", Rc::clone(&rc));
//!
//! // Atomic reference counting
//! let arc = track_arc_new("arc", Arc::new(42));
//!
//! // Box heap allocation
//! let boxed = track_box_new("boxed", "loc", Box::new(42));
//! ```
//!
//! # Weak Reference Tracking
//!
//! Track weak references and upgrade attempts:
//!
//! ```rust
//! use borrowscope_runtime::*;
//! use std::rc::{Rc, Weak};
//!
//! reset();
//!
//! let rc = Rc::new(42);
//! let weak = track_weak_new("weak", "rc", "loc", Rc::downgrade(&rc));
//!
//! // Track upgrade attempts
//! if let Some(upgraded) = track_weak_upgrade("weak", "loc", weak.upgrade()) {
//!     println!("Upgraded: {}", upgraded);
//! }
//! ```
//!
//! # Concurrency Tracking
//!
//! Track threads and channels:
//!
//! ```rust
//! use borrowscope_runtime::*;
//! use std::sync::mpsc;
//!
//! reset();
//!
//! // Track channel creation
//! let (tx, rx) = mpsc::channel();
//! let (tx, rx) = track_channel("chan", "loc", tx, rx);
//!
//! // Track send/receive
//! let _ = track_channel_send("chan", "loc", tx.send(42));
//! let received = track_channel_recv("chan", "loc", rx.recv());
//! ```
//!
//! # OnceCell/OnceLock Tracking
//!
//! Track lazy initialization:
//!
//! ```rust
//! use borrowscope_runtime::*;
//! use std::cell::OnceCell;
//!
//! reset();
//!
//! let cell: OnceCell<i32> = track_once_cell_new("config", "loc", OnceCell::new());
//! let _ = track_once_cell_set("config", "loc", cell.set(42));
//! let value = track_once_cell_get("config", "loc", cell.get());
//! ```
//!
//! # MaybeUninit Tracking
//!
//! Track uninitialized memory operations:
//!
//! ```rust
//! use borrowscope_runtime::*;
//! use std::mem::MaybeUninit;
//!
//! reset();
//!
//! let mut uninit: MaybeUninit<i32> = track_maybe_uninit_uninit("data", "loc", MaybeUninit::uninit());
//! let written = track_maybe_uninit_write("data", "loc", uninit.write(42));
//! let value = track_maybe_uninit_assume_init("data", "loc", unsafe { uninit.assume_init() });
//! ```
//!
//! # RAII Guards
//!
//! For automatic drop tracking, use the guard variants:
//!
//! ```rust
//! use borrowscope_runtime::*;
//!
//! reset();
//! {
//!     let data = track_new_guard("data", vec![1, 2, 3]);
//!     println!("{:?}", *data);
//!     // track_drop("data") called automatically when data goes out of scope
//! }
//!
//! let events = get_events();
//! assert!(events.last().unwrap().is_drop());
//! ```
//!
//! # Event Filtering
//!
//! Filter events by category using helper methods:
//!
//! ```rust
//! use borrowscope_runtime::*;
//!
//! reset();
//! // ... tracking code ...
//!
//! let events = get_events();
//!
//! // Filter by category
//! let box_events: Vec<_> = events.iter().filter(|e| e.is_box()).collect();
//! let weak_events: Vec<_> = events.iter().filter(|e| e.is_weak()).collect();
//! let thread_events: Vec<_> = events.iter().filter(|e| e.is_thread()).collect();
//! let channel_events: Vec<_> = events.iter().filter(|e| e.is_channel()).collect();
//! let once_cell_events: Vec<_> = events.iter().filter(|e| e.is_once_cell()).collect();
//! let maybe_uninit_events: Vec<_> = events.iter().filter(|e| e.is_maybe_uninit()).collect();
//! ```
//!
//! # Performance
//!
//! - With `track` feature: ~75-80ns per tracking call
//! - Without `track` feature: zero overhead (compiled away)

mod error;
mod event;
mod export;
mod graph;
mod guard;
mod lifetime;
mod tracker;

#[cfg(test)]
mod test_utils;

pub use error::{BorrowScopeResult, Error};
pub use event::Event;
pub use export::{ExportData, ExportEdge, ExportMetadata};
pub use graph::{build_graph, GraphStats, OwnershipGraph, Relationship, Variable};
pub use guard::{
    track_borrow_guard, track_borrow_mut_guard, track_new_guard, BorrowGuard, BorrowMutGuard,
    TrackGuard,
};
pub use lifetime::{ElisionRule, LifetimeRelation, Timeline};
pub use tracker::{
    __track_new_with_id_helper, get_borrow_events, get_drop_events, get_event_counts, get_events,
    get_events_filtered, get_events_for_var, get_move_events, get_new_events, get_summary,
    print_summary, reset, track_arc_clone, track_arc_clone_with_id, track_arc_new,
    track_arc_new_with_id, track_array_create, track_async_block_enter, track_async_block_exit,
    track_await_end, track_await_start, track_binary_op, track_borrow, track_borrow_mut,
    track_borrow_mut_with_id, track_borrow_with_id, track_box_from_raw, track_box_into_raw,
    track_box_new, track_branch, track_break, track_call, track_cell_get, track_cell_new,
    track_cell_set, track_channel, track_channel_recv, track_channel_send, track_channel_try_recv,
    track_clone, track_closure_capture, track_closure_create, track_const_eval, track_continue,
    track_cow_borrowed, track_cow_owned, track_cow_to_mut, track_deref, track_drop, track_drop_batch,
    track_drop_with_id, track_ffi_call, track_field_access, track_fn_enter, track_fn_exit,
    track_index_access, track_let_else, track_lock, track_lock_guard_acquire, track_lock_guard_drop,
    track_loop_enter, track_loop_exit, track_loop_iteration, track_match_arm, track_match_enter,
    track_match_exit, track_maybe_uninit_assume_init, track_maybe_uninit_assume_init_drop,
    track_maybe_uninit_assume_init_read, track_maybe_uninit_new, track_maybe_uninit_uninit,
    track_maybe_uninit_write, track_move, track_move_with_id, track_new, track_new_with_id,
    track_once_cell_get, track_once_cell_get_or_init, track_once_cell_new, track_once_cell_set,
    track_once_lock_new, track_pin_into_inner, track_pin_new, track_range, track_raw_ptr,
    track_raw_ptr_deref, track_raw_ptr_mut, track_rc_clone, track_rc_clone_with_id, track_rc_new,
    track_rc_new_with_id, track_refcell_borrow, track_refcell_borrow_mut, track_refcell_drop,
    track_refcell_new, track_region_enter, track_region_exit, track_return, track_static_access,
    track_static_init, track_struct_create, track_thread_join, track_thread_spawn, track_transmute,
    track_try, track_tuple_create, track_type_cast, track_union_field_access,
    track_unsafe_block_enter, track_unsafe_block_exit, track_unsafe_fn_call, track_unwrap,
    track_weak_clone, track_weak_clone_sync, track_weak_new, track_weak_new_sync, track_weak_upgrade,
    track_weak_upgrade_sync, TrackingSummary,
};

/// Convenience macro for RefCell borrow tracking with auto file:line capture
#[macro_export]
macro_rules! refcell_borrow {
    ($name:expr, $id:expr, $guard:expr) => {
        $crate::track_refcell_borrow($name, $id, concat!(file!(), ":", line!()), $guard)
    };
}

/// Convenience macro for RefCell borrow_mut tracking with auto file:line capture
#[macro_export]
macro_rules! refcell_borrow_mut {
    ($name:expr, $id:expr, $guard:expr) => {
        $crate::track_refcell_borrow_mut($name, $id, concat!(file!(), ":", line!()), $guard)
    };
}

/// Convenience macro for RefCell drop tracking with auto file:line capture
#[macro_export]
macro_rules! refcell_drop {
    ($name:expr) => {
        $crate::track_refcell_drop($name, concat!(file!(), ":", line!()))
    };
}

/// Get the ownership graph built from current events
pub fn get_graph() -> OwnershipGraph {
    let events = get_events();
    build_graph(&events)
}

/// Export current tracking data to JSON file
pub fn export_json<P: AsRef<std::path::Path>>(path: P) -> BorrowScopeResult<()> {
    let events = get_events();
    let graph = build_graph(&events);
    let export = ExportData::new(graph, events);
    export.to_file(path)
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::test_utils::TEST_LOCK;

    #[test]
    fn test_simple_tracking() {
        let _lock = TEST_LOCK.lock();

        reset();

        let x = track_new("x", 5);
        assert_eq!(x, 5);

        let events = get_events();
        assert_eq!(events.len(), 1);
        assert!(events[0].is_new());
    }

    #[test]
    fn test_borrow_tracking() {
        let _lock = TEST_LOCK.lock();

        reset();

        let s = track_new("s", String::from("hello"));
        let r = track_borrow("r", &s);

        assert_eq!(r, "hello");

        let events = get_events();
        assert_eq!(events.len(), 2);
        assert!(events[0].is_new());
        assert!(events[1].is_borrow());
    }

    #[test]
    fn test_multiple_variables() {
        let _lock = TEST_LOCK.lock();

        reset();

        let x = track_new("x", 5);
        let y = track_new("y", 10);
        let z = x + y;

        track_drop("y");
        track_drop("x");

        let events = get_events();
        assert_eq!(events.len(), 4);

        assert_eq!(z, 15);
    }

    #[test]
    fn test_mutable_borrow() {
        let _lock = TEST_LOCK.lock();

        reset();

        let mut x = track_new("x", vec![1, 2, 3]);
        let r = track_borrow_mut("r", &mut x);
        r.push(4);

        assert_eq!(r.len(), 4);

        let events = get_events();
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_graph_building() {
        let _lock = TEST_LOCK.lock();

        reset();

        let x = track_new("x", 5);
        let _r = track_borrow("r", &x);
        // Note: borrowers don't get track_drop calls in current implementation
        track_drop("x");

        let graph = get_graph();
        // Only x is tracked as a variable
        assert_eq!(graph.nodes.len(), 1);
        // No edges because borrow wasn't ended with a drop
        assert_eq!(graph.edges.len(), 0);

        let stats = graph.stats();
        assert_eq!(stats.total_variables, 1);
    }

    #[test]
    fn test_export_json() {
        let _lock = TEST_LOCK.lock();

        reset();

        let x = track_new("x", 5);
        let _r = track_borrow("r", &x);
        track_drop("x");

        // Export to temporary file
        let temp_path = std::env::temp_dir().join("borrowscope_test.json");
        export_json(&temp_path).unwrap();

        // Verify file exists and is valid JSON
        let contents = std::fs::read_to_string(&temp_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&contents).unwrap();

        assert!(parsed["nodes"].is_array());
        assert!(parsed["events"].is_array());
        assert!(parsed["metadata"].is_object());

        // Cleanup
        std::fs::remove_file(&temp_path).ok();
    }
}
