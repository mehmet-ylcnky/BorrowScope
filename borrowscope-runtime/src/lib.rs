#![allow(dead_code)]
//! # BorrowScope Runtime
//!
//! A runtime tracking library for visualizing Rust's ownership and borrowing system.
//!
//! This crate captures ownership transfers, borrows, smart pointer operations,
//! concurrency primitives, and unsafe code as they happen at runtime, generating
//! structured event data for analysis and visualization.
//!
//! ## Quick Start
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
//! ## Feature Flags
//!
//! - `track` - Enables runtime tracking. Without this feature, all tracking
//!   functions compile to no-ops with zero overhead.
//!
//! ```toml
//! [dependencies]
//! borrowscope-runtime = { version = "0.1", features = ["track"] }
//! ```
//!
//! ## Tracking Functions by Category
//!
//! ### Basic Ownership
//!
//! Core ownership tracking for variables, borrows, moves, and drops.
//!
//! | Function | Description |
//! |----------|-------------|
//! | [`track_new`] | Track variable creation |
//! | [`track_new_with_id`] | Track creation with explicit ID and location |
//! | [`track_borrow`] | Track immutable borrow (`&T`) |
//! | [`track_borrow_with_id`] | Track immutable borrow with explicit IDs |
//! | [`track_borrow_mut`] | Track mutable borrow (`&mut T`) |
//! | [`track_borrow_mut_with_id`] | Track mutable borrow with explicit IDs |
//! | [`track_move`] | Track ownership transfer |
//! | [`track_move_with_id`] | Track move with explicit IDs |
//! | [`track_drop`] | Track variable going out of scope |
//! | [`track_drop_with_id`] | Track drop with explicit ID |
//! | [`track_drop_batch`] | Track multiple drops efficiently |
//!
//! ### Smart Pointers
//!
//! Track reference-counted and heap-allocated smart pointers.
//!
//! | Function | Description |
//! |----------|-------------|
//! | [`track_rc_new`] | Track `Rc::new` |
//! | [`track_rc_clone`] | Track `Rc::clone` |
//! | [`track_arc_new`] | Track `Arc::new` |
//! | [`track_arc_clone`] | Track `Arc::clone` |
//! | [`track_weak_new`] | Track `Rc::downgrade` |
//! | [`track_weak_new_sync`] | Track `Arc::downgrade` |
//! | [`track_weak_clone`] | Track `Weak::clone` (Rc) |
//! | [`track_weak_clone_sync`] | Track `Weak::clone` (Arc) |
//! | [`track_weak_upgrade`] | Track `Weak::upgrade` (Rc) |
//! | [`track_weak_upgrade_sync`] | Track `Weak::upgrade` (Arc) |
//! | [`track_box_new`] | Track `Box::new` |
//! | [`track_box_into_raw`] | Track `Box::into_raw` |
//! | [`track_box_from_raw`] | Track `Box::from_raw` |
//! | [`track_pin_new`] | Track `Pin::new` |
//! | [`track_pin_into_inner`] | Track `Pin::into_inner` |
//! | [`track_cow_borrowed`] | Track `Cow::Borrowed` |
//! | [`track_cow_owned`] | Track `Cow::Owned` |
//! | [`track_cow_to_mut`] | Track `Cow::to_mut` |
//!
//! ### Interior Mutability
//!
//! Track runtime borrow checking and cell types.
//!
//! | Function | Description |
//! |----------|-------------|
//! | [`track_refcell_new`] | Track `RefCell::new` |
//! | [`track_refcell_borrow`] | Track `RefCell::borrow` |
//! | [`track_refcell_borrow_mut`] | Track `RefCell::borrow_mut` |
//! | [`track_refcell_drop`] | Track `Ref`/`RefMut` guard drop |
//! | [`track_cell_new`] | Track `Cell::new` |
//! | [`track_cell_get`] | Track `Cell::get` |
//! | [`track_cell_set`] | Track `Cell::set` |
//! | [`track_once_cell_new`] | Track `OnceCell::new` |
//! | [`track_once_lock_new`] | Track `OnceLock::new` |
//! | [`track_once_cell_set`] | Track `OnceCell::set` |
//! | [`track_once_cell_get`] | Track `OnceCell::get` |
//! | [`track_once_cell_get_or_init`] | Track `OnceCell::get_or_init` |
//! | [`track_maybe_uninit_uninit`] | Track `MaybeUninit::uninit` |
//! | [`track_maybe_uninit_new`] | Track `MaybeUninit::new` |
//! | [`track_maybe_uninit_write`] | Track `MaybeUninit::write` |
//! | [`track_maybe_uninit_assume_init`] | Track `MaybeUninit::assume_init` |
//! | [`track_maybe_uninit_assume_init_read`] | Track `MaybeUninit::assume_init_read` |
//! | [`track_maybe_uninit_assume_init_drop`] | Track `MaybeUninit::assume_init_drop` |
//!
//! ### Unsafe Code
//!
//! Track unsafe operations, raw pointers, and FFI.
//!
//! | Function | Description |
//! |----------|-------------|
//! | [`track_raw_ptr`] | Track `*const T` creation |
//! | [`track_raw_ptr_mut`] | Track `*mut T` creation |
//! | [`track_raw_ptr_deref`] | Track raw pointer dereference |
//! | [`track_unsafe_block_enter`] | Track entering `unsafe` block |
//! | [`track_unsafe_block_exit`] | Track exiting `unsafe` block |
//! | [`track_unsafe_fn_call`] | Track unsafe function call |
//! | [`track_ffi_call`] | Track FFI function call |
//! | [`track_transmute`] | Track `std::mem::transmute` |
//! | [`track_union_field_access`] | Track union field access |
//!
//! ### Concurrency
//!
//! Track threads, channels, and synchronization primitives.
//!
//! | Function | Description |
//! |----------|-------------|
//! | [`track_thread_spawn`] | Track `thread::spawn` |
//! | [`track_thread_join`] | Track `JoinHandle::join` |
//! | [`track_channel`] | Track `mpsc::channel` creation |
//! | [`track_channel_send`] | Track `Sender::send` |
//! | [`track_channel_recv`] | Track `Receiver::recv` |
//! | [`track_channel_try_recv`] | Track `Receiver::try_recv` |
//! | [`track_lock`] | Track lock acquisition |
//! | [`track_lock_guard_acquire`] | Track lock guard creation |
//! | [`track_lock_guard_drop`] | Track lock guard drop |
//!
//! ### Async/Await
//!
//! Track async blocks and await points.
//!
//! | Function | Description |
//! |----------|-------------|
//! | [`track_async_block_enter`] | Track entering async block |
//! | [`track_async_block_exit`] | Track exiting async block |
//! | [`track_await_start`] | Track await expression start |
//! | [`track_await_end`] | Track await expression completion |
//!
//! ### Control Flow
//!
//! Track loops, matches, branches, and function boundaries.
//!
//! | Function | Description |
//! |----------|-------------|
//! | [`track_loop_enter`] | Track loop entry |
//! | [`track_loop_iteration`] | Track loop iteration |
//! | [`track_loop_exit`] | Track loop exit |
//! | [`track_match_enter`] | Track match expression entry |
//! | [`track_match_arm`] | Track match arm taken |
//! | [`track_match_exit`] | Track match expression exit |
//! | [`track_branch`] | Track if/else branch |
//! | [`track_return`] | Track return statement |
//! | [`track_try`] | Track `?` operator |
//! | [`track_break`] | Track break statement |
//! | [`track_continue`] | Track continue statement |
//! | [`track_fn_enter`] | Track function entry |
//! | [`track_fn_exit`] | Track function exit |
//! | [`track_region_enter`] | Track scope/region entry |
//! | [`track_region_exit`] | Track scope/region exit |
//!
//! ### Expressions
//!
//! Track various expression types and operations.
//!
//! | Function | Description |
//! |----------|-------------|
//! | [`track_index_access`] | Track array/slice indexing |
//! | [`track_field_access`] | Track struct field access |
//! | [`track_call`] | Track function/method call |
//! | [`track_unwrap`] | Track `unwrap`/`expect` calls |
//! | [`track_clone`] | Track `clone` calls |
//! | [`track_deref`] | Track dereference operations |
//! | [`track_closure_create`] | Track closure creation |
//! | [`track_closure_capture`] | Track closure variable capture |
//! | [`track_struct_create`] | Track struct instantiation |
//! | [`track_tuple_create`] | Track tuple creation |
//! | [`track_array_create`] | Track array creation |
//! | [`track_let_else`] | Track let-else patterns |
//! | [`track_range`] | Track range expressions |
//! | [`track_binary_op`] | Track binary operations |
//! | [`track_type_cast`] | Track type casts (`as`) |
//!
//! ### Static/Const
//!
//! Track static variables and const evaluation.
//!
//! | Function | Description |
//! |----------|-------------|
//! | [`track_static_init`] | Track static variable initialization |
//! | [`track_static_access`] | Track static variable access |
//! | [`track_const_eval`] | Track const evaluation |
//!
//! ### Sampling
//!
//! Probabilistic tracking for reduced overhead.
//!
//! | Function | Description |
//! |----------|-------------|
//! | [`should_sample`] | Check if call should be sampled |
//! | [`track_new_sampled`] | Track creation with sampling |
//! | [`track_new_with_id_sampled`] | Track creation with ID and sampling |
//! | [`track_borrow_sampled`] | Track borrow with sampling |
//! | [`track_borrow_mut_sampled`] | Track mutable borrow with sampling |
//! | [`track_drop_sampled`] | Track drop with sampling |
//! | [`track_move_sampled`] | Track move with sampling |
//!
//! ### Query Functions
//!
//! Filter and summarize tracked events.
//!
//! | Function | Description |
//! |----------|-------------|
//! | [`get_events`] | Get all recorded events |
//! | [`get_events_filtered`] | Get events matching predicate |
//! | [`get_new_events`] | Get all `New` events |
//! | [`get_borrow_events`] | Get all `Borrow` events |
//! | [`get_drop_events`] | Get all `Drop` events |
//! | [`get_move_events`] | Get all `Move` events |
//! | [`get_events_for_var`] | Get events for specific variable |
//! | [`get_event_counts`] | Get (new, borrow, move, drop) counts |
//! | [`get_summary`] | Get [`TrackingSummary`] statistics |
//! | [`print_summary`] | Print summary to stdout |
//! | [`reset`] | Clear all recorded events |
//!
//! ## RAII Guards
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
//! ## Performance
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
    print_summary, reset, should_sample, track_arc_clone, track_arc_clone_with_id, track_arc_new,
    track_arc_new_with_id, track_array_create, track_async_block_enter, track_async_block_exit,
    track_atomic_new, track_autoderef, track_autoref, track_await_end, track_await_start,
    track_await_start_with_live_vars, track_binary_op, track_borrow, track_borrow_mut,
    track_borrow_mut_sampled, track_borrow_mut_with_id, track_borrow_sampled, track_borrow_span,
    track_borrow_with_id, track_box_from_raw, track_box_into_raw, track_box_new, track_branch,
    track_break, track_call, track_cell_get, track_cell_new, track_cell_set, track_channel,
    track_channel_recv, track_channel_send, track_channel_try_recv, track_clone,
    track_closure_capture, track_closure_create, track_closure_create_with_trait, track_const_eval,
    track_continue, track_cow_borrowed, track_cow_owned, track_cow_to_mut, track_deref,
    track_destructure, track_drop, track_drop_at, track_drop_batch, track_drop_sampled,
    track_drop_with_id, track_duration_new, track_ffi_call, track_field_access, track_fn_enter,
    track_fn_exit, track_index_access, track_instant_new, track_let_else, track_lock,
    track_lock_guard_acquire, track_lock_guard_drop, track_loop_enter, track_loop_exit,
    track_loop_iteration, track_match_arm, track_match_arm_with_bindings, track_match_enter,
    track_match_exit, track_maybe_uninit_assume_init, track_maybe_uninit_assume_init_drop,
    track_maybe_uninit_assume_init_read, track_maybe_uninit_new, track_maybe_uninit_uninit,
    track_maybe_uninit_write, track_method_call, track_move, track_move_sampled,
    track_move_with_id, track_new, track_new_sampled, track_new_with_id, track_new_with_id_sampled,
    track_once_cell_get, track_once_cell_get_or_init, track_once_cell_new, track_once_cell_set,
    track_once_lock_new, track_pin_into_inner, track_pin_new, track_range, track_raw_ptr,
    track_raw_ptr_deref, track_raw_ptr_mut, track_rc_clone, track_rc_clone_with_id, track_rc_new,
    track_rc_new_with_id, track_refcell_borrow, track_refcell_borrow_mut, track_refcell_drop,
    track_refcell_new, track_region_enter, track_region_exit, track_return, track_static_access,
    track_static_init, track_struct_create, track_thread_join, track_thread_spawn, track_transmute,
    track_try, track_tuple_create, track_type_cast, track_union_field_access,
    track_unsafe_block_enter, track_unsafe_block_enter_enriched, track_unsafe_block_exit,
    track_unsafe_fn_call, track_unwrap, track_var_read, track_var_write, track_variant_construct,
    track_weak_clone, track_weak_clone_sync, track_weak_new, track_weak_new_sync,
    track_weak_upgrade, track_weak_upgrade_sync, TrackingSummary,
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
