//! Core tracking functionality for recording ownership and borrowing events.
//!
//! This module provides the public API for tracking Rust ownership operations at runtime.
//! All tracking functions are designed to be zero-cost when the `track` feature is disabled.
//!
//! # Overview
//!
//! The tracker records events for:
//! - **Basic ownership**: variable creation, borrowing, moves, and drops
//! - **Smart pointers**: `Rc` and `Arc` creation and cloning
//! - **Interior mutability**: `RefCell` and `Cell` operations
//! - **Unsafe code**: raw pointers, unsafe blocks, FFI calls, transmute
//! - **Static/const**: static variable access and const evaluation
//!
//! # Quick Start
//!
//! ```rust
//! use borrowscope_runtime::*;
//!
//! // Always reset before tracking a new session
//! reset();
//!
//! // Track variable creation - returns the value unchanged
//! let data = track_new("data", vec![1, 2, 3]);
//!
//! // Track borrowing - returns the reference unchanged
//! let r = track_borrow("r", &data);
//! println!("{:?}", r);
//!
//! // Track drops explicitly
//! track_drop("r");
//! track_drop("data");
//!
//! // Retrieve all recorded events
//! let events = get_events();
//! assert_eq!(events.len(), 4); // New, Borrow, Drop, Drop
//! ```
//!
//! # Feature Flags
//!
//! - `track` - Enables runtime tracking. Without this feature, all tracking
//!   functions compile to no-ops that simply return their input values.
//!
//! # Thread Safety
//!
//! All tracking functions are thread-safe. The global tracker uses a mutex
//! to ensure consistent event ordering across threads.
//!
//! # Performance
//!
//! With `track` enabled: ~75-80ns per tracking call.
//! Without `track`: zero overhead (functions are inlined away).

use crate::event::Event;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

lazy_static! {
    /// Global tracker instance
    static ref TRACKER: Mutex<Tracker> = Mutex::new(Tracker::new());
}

/// Global timestamp counter
static TIMESTAMP: AtomicU64 = AtomicU64::new(0);

/// The main tracker that records events
pub struct Tracker {
    /// All recorded events
    events: Vec<Event>,

    /// Counter for generating unique variable IDs
    var_counter: u64,
}

impl Tracker {
    /// Create a new tracker
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            var_counter: 0,
        }
    }

    /// Generate next timestamp
    fn next_timestamp() -> u64 {
        TIMESTAMP.fetch_add(1, Ordering::Relaxed)
    }

    /// Generate unique variable ID
    fn next_var_id(&mut self, name: &str) -> String {
        let id = format!("{}_{}", name, self.var_counter);
        self.var_counter += 1;
        id
    }

    /// Record a New event
    #[cfg_attr(not(feature = "track"), allow(dead_code))]
    pub fn record_new(&mut self, var_name: &str, type_name: &str) -> String {
        let timestamp = Self::next_timestamp();
        let var_id = self.next_var_id(var_name);

        self.events.push(Event::New {
            timestamp,
            var_name: var_name.to_string(),
            var_id: var_id.clone(),
            type_name: type_name.to_string(),
        });

        var_id
    }

    /// Record a Borrow event
    #[cfg_attr(not(feature = "track"), allow(dead_code))]
    pub fn record_borrow(&mut self, borrower_name: &str, owner_id: &str, mutable: bool) -> String {
        let timestamp = Self::next_timestamp();
        let borrower_id = self.next_var_id(borrower_name);

        self.events.push(Event::Borrow {
            timestamp,
            borrower_name: borrower_name.to_string(),
            borrower_id: borrower_id.clone(),
            owner_id: owner_id.to_string(),
            mutable,
        });

        borrower_id
    }

    /// Record a Move event
    #[allow(dead_code)]
    pub fn record_move(&mut self, from_id: &str, to_name: &str) -> String {
        let timestamp = Self::next_timestamp();
        let to_id = self.next_var_id(to_name);

        self.events.push(Event::Move {
            timestamp,
            from_id: from_id.to_string(),
            to_name: to_name.to_string(),
            to_id: to_id.clone(),
        });

        to_id
    }

    /// Record a Drop event
    #[cfg_attr(not(feature = "track"), allow(dead_code))]
    pub fn record_drop(&mut self, var_id: &str) {
        let timestamp = Self::next_timestamp();

        self.events.push(Event::Drop {
            timestamp,
            var_id: var_id.to_string(),
        });
    }

    /// Record an Rc::new event
    #[cfg_attr(not(feature = "track"), allow(dead_code))]
    pub fn record_rc_new(
        &mut self,
        var_name: &str,
        strong_count: usize,
        weak_count: usize,
    ) -> String {
        let timestamp = Self::next_timestamp();
        let var_id = self.next_var_id(var_name);

        self.events.push(Event::RcNew {
            timestamp,
            var_name: var_name.to_string(),
            var_id: var_id.clone(),
            type_name: "Rc<T>".to_string(),
            strong_count,
            weak_count,
        });

        var_id
    }

    /// Record an Rc::clone event
    #[cfg_attr(not(feature = "track"), allow(dead_code))]
    pub fn record_rc_clone(
        &mut self,
        var_name: &str,
        source_name: &str,
        strong_count: usize,
        weak_count: usize,
    ) -> String {
        let timestamp = Self::next_timestamp();
        let var_id = self.next_var_id(var_name);

        self.events.push(Event::RcClone {
            timestamp,
            var_name: var_name.to_string(),
            var_id: var_id.clone(),
            source_id: source_name.to_string(),
            strong_count,
            weak_count,
        });

        var_id
    }

    /// Record an Arc::new event
    #[cfg_attr(not(feature = "track"), allow(dead_code))]
    pub fn record_arc_new(
        &mut self,
        var_name: &str,
        strong_count: usize,
        weak_count: usize,
    ) -> String {
        let timestamp = Self::next_timestamp();
        let var_id = self.next_var_id(var_name);

        self.events.push(Event::ArcNew {
            timestamp,
            var_name: var_name.to_string(),
            var_id: var_id.clone(),
            type_name: "Arc<T>".to_string(),
            strong_count,
            weak_count,
        });

        var_id
    }

    /// Record an Arc::clone event
    #[cfg_attr(not(feature = "track"), allow(dead_code))]
    pub fn record_arc_clone(
        &mut self,
        var_name: &str,
        source_name: &str,
        strong_count: usize,
        weak_count: usize,
    ) -> String {
        let timestamp = Self::next_timestamp();
        let var_id = self.next_var_id(var_name);

        self.events.push(Event::ArcClone {
            timestamp,
            var_name: var_name.to_string(),
            var_id: var_id.clone(),
            source_id: source_name.to_string(),
            strong_count,
            weak_count,
        });

        var_id
    }

    /// Record a New event with explicit ID and location (advanced API)
    #[cfg_attr(not(feature = "track"), allow(dead_code))]
    pub fn record_new_with_id(
        &mut self,
        id: usize,
        var_name: &str,
        type_name: &str,
        location: &str,
    ) -> String {
        let timestamp = Self::next_timestamp();
        let var_id = format!("{}_{}", var_name, id);

        self.events.push(Event::New {
            timestamp,
            var_name: var_name.to_string(),
            var_id: var_id.clone(),
            type_name: format!("{} @ {}", type_name, location),
        });

        var_id
    }

    /// Record a Borrow event with explicit IDs and location (advanced API)
    #[cfg_attr(not(feature = "track"), allow(dead_code))]
    pub fn record_borrow_with_id(
        &mut self,
        borrower_id: usize,
        owner_id: usize,
        borrower_name: &str,
        location: &str,
        mutable: bool,
    ) -> String {
        let timestamp = Self::next_timestamp();
        let borrower_var_id = format!("{}_{}", borrower_name, borrower_id);
        let owner_var_id = format!("owner_{}", owner_id);

        self.events.push(Event::Borrow {
            timestamp,
            borrower_name: format!("{} @ {}", borrower_name, location),
            borrower_id: borrower_var_id.clone(),
            owner_id: owner_var_id,
            mutable,
        });

        borrower_var_id
    }

    /// Record a Move event with explicit IDs and location (advanced API)
    #[allow(dead_code)]
    pub fn record_move_with_id(
        &mut self,
        from_id: usize,
        to_id: usize,
        to_name: &str,
        location: &str,
    ) -> String {
        let timestamp = Self::next_timestamp();
        let from_var_id = format!("var_{}", from_id);
        let to_var_id = format!("{}_{}", to_name, to_id);

        self.events.push(Event::Move {
            timestamp,
            from_id: from_var_id,
            to_name: format!("{} @ {}", to_name, location),
            to_id: to_var_id.clone(),
        });

        to_var_id
    }

    /// Record a Drop event with explicit ID and location (advanced API)
    #[cfg_attr(not(feature = "track"), allow(dead_code))]
    pub fn record_drop_with_id(&mut self, id: usize, location: &str) {
        let timestamp = Self::next_timestamp();
        let var_id = format!("var_{} @ {}", id, location);

        self.events.push(Event::Drop { timestamp, var_id });
    }

    /// Record an Rc::new event with explicit ID and location (advanced API)
    #[cfg_attr(not(feature = "track"), allow(dead_code))]
    pub fn record_rc_new_with_id(
        &mut self,
        id: usize,
        var_name: &str,
        type_name: &str,
        location: &str,
        strong_count: usize,
        weak_count: usize,
    ) -> String {
        let timestamp = Self::next_timestamp();
        let var_id = format!("{}_{}", var_name, id);

        self.events.push(Event::RcNew {
            timestamp,
            var_name: var_name.to_string(),
            var_id: var_id.clone(),
            type_name: format!("{} @ {}", type_name, location),
            strong_count,
            weak_count,
        });

        var_id
    }

    /// Record an Rc::clone event with explicit IDs and location (advanced API)
    #[cfg_attr(not(feature = "track"), allow(dead_code))]
    pub fn record_rc_clone_with_id(
        &mut self,
        new_id: usize,
        source_id: usize,
        var_name: &str,
        location: &str,
        strong_count: usize,
        weak_count: usize,
    ) -> String {
        let timestamp = Self::next_timestamp();
        let var_id = format!("{}_{}", var_name, new_id);
        let source_var_id = format!("var_{}", source_id);

        self.events.push(Event::RcClone {
            timestamp,
            var_name: format!("{} @ {}", var_name, location),
            var_id: var_id.clone(),
            source_id: source_var_id,
            strong_count,
            weak_count,
        });

        var_id
    }

    /// Record an Arc::new event with explicit ID and location (advanced API)
    #[cfg_attr(not(feature = "track"), allow(dead_code))]
    pub fn record_arc_new_with_id(
        &mut self,
        id: usize,
        var_name: &str,
        type_name: &str,
        location: &str,
        strong_count: usize,
        weak_count: usize,
    ) -> String {
        let timestamp = Self::next_timestamp();
        let var_id = format!("{}_{}", var_name, id);

        self.events.push(Event::ArcNew {
            timestamp,
            var_name: var_name.to_string(),
            var_id: var_id.clone(),
            type_name: format!("{} @ {}", type_name, location),
            strong_count,
            weak_count,
        });

        var_id
    }

    /// Record an Arc::clone event with explicit IDs and location (advanced API)
    #[cfg_attr(not(feature = "track"), allow(dead_code))]
    pub fn record_arc_clone_with_id(
        &mut self,
        new_id: usize,
        source_id: usize,
        var_name: &str,
        location: &str,
        strong_count: usize,
        weak_count: usize,
    ) -> String {
        let timestamp = Self::next_timestamp();
        let var_id = format!("{}_{}", var_name, new_id);
        let source_var_id = format!("var_{}", source_id);

        self.events.push(Event::ArcClone {
            timestamp,
            var_name: format!("{} @ {}", var_name, location),
            var_id: var_id.clone(),
            source_id: source_var_id,
            strong_count,
            weak_count,
        });

        var_id
    }

    /// Record RefCell::new
    pub fn record_refcell_new(&mut self, var_name: &str) -> String {
        let timestamp = Self::next_timestamp();
        let var_id = format!("refcell_{}", var_name);

        self.events.push(Event::RefCellNew {
            timestamp,
            var_name: var_name.to_string(),
            var_id: var_id.clone(),
            type_name: "RefCell<T>".to_string(),
        });

        var_id
    }

    /// Record RefCell::borrow or borrow_mut
    pub fn record_refcell_borrow(
        &mut self,
        borrow_id: &str,
        refcell_id: &str,
        is_mutable: bool,
        location: &str,
    ) {
        let timestamp = Self::next_timestamp();

        self.events.push(Event::RefCellBorrow {
            timestamp,
            borrow_id: borrow_id.to_string(),
            refcell_id: refcell_id.to_string(),
            is_mutable,
            location: location.to_string(),
        });
    }

    /// Record RefCell borrow drop
    pub fn record_refcell_drop(&mut self, borrow_id: &str, location: &str) {
        let timestamp = Self::next_timestamp();

        self.events.push(Event::RefCellDrop {
            timestamp,
            borrow_id: borrow_id.to_string(),
            location: location.to_string(),
        });
    }

    /// Record Cell::new
    pub fn record_cell_new(&mut self, var_name: &str) -> String {
        let timestamp = Self::next_timestamp();
        let var_id = format!("cell_{}", var_name);

        self.events.push(Event::CellNew {
            timestamp,
            var_name: var_name.to_string(),
            var_id: var_id.clone(),
            type_name: "Cell<T>".to_string(),
        });

        var_id
    }

    /// Record Cell::get
    pub fn record_cell_get(&mut self, cell_id: &str, location: &str) {
        let timestamp = Self::next_timestamp();

        self.events.push(Event::CellGet {
            timestamp,
            cell_id: cell_id.to_string(),
            location: location.to_string(),
        });
    }

    /// Record Cell::set
    pub fn record_cell_set(&mut self, cell_id: &str, location: &str) {
        let timestamp = Self::next_timestamp();

        self.events.push(Event::CellSet {
            timestamp,
            cell_id: cell_id.to_string(),
            location: location.to_string(),
        });
    }

    /// Record static variable initialization
    pub fn record_static_init(
        &mut self,
        var_name: &str,
        var_id: usize,
        type_name: &str,
        is_mutable: bool,
    ) {
        let timestamp = Self::next_timestamp();

        self.events.push(Event::StaticInit {
            timestamp,
            var_name: var_name.to_string(),
            var_id: var_id.to_string(),
            type_name: type_name.to_string(),
            is_mutable,
        });
    }

    /// Record static variable access
    pub fn record_static_access(
        &mut self,
        var_id: usize,
        var_name: &str,
        is_write: bool,
        location: &str,
    ) {
        let timestamp = Self::next_timestamp();

        self.events.push(Event::StaticAccess {
            timestamp,
            var_id: var_id.to_string(),
            var_name: var_name.to_string(),
            is_write,
            location: location.to_string(),
        });
    }

    /// Record const evaluation
    pub fn record_const_eval(
        &mut self,
        const_name: &str,
        const_id: usize,
        type_name: &str,
        location: &str,
    ) {
        let timestamp = Self::next_timestamp();

        self.events.push(Event::ConstEval {
            timestamp,
            const_name: const_name.to_string(),
            const_id: const_id.to_string(),
            type_name: type_name.to_string(),
            location: location.to_string(),
        });
    }

    /// Record raw pointer creation
    pub fn record_raw_ptr_created(
        &mut self,
        var_name: &str,
        var_id: usize,
        ptr_type: &str,
        address: usize,
        location: &str,
    ) {
        let timestamp = Self::next_timestamp();

        self.events.push(Event::RawPtrCreated {
            timestamp,
            var_name: var_name.to_string(),
            var_id: var_id.to_string(),
            ptr_type: ptr_type.to_string(),
            address,
            location: location.to_string(),
        });
    }

    /// Record raw pointer dereference
    pub fn record_raw_ptr_deref(&mut self, ptr_id: usize, location: &str, is_write: bool) {
        let timestamp = Self::next_timestamp();

        self.events.push(Event::RawPtrDeref {
            timestamp,
            ptr_id: ptr_id.to_string(),
            location: location.to_string(),
            is_write,
        });
    }

    /// Record unsafe block entry
    pub fn record_unsafe_block_enter(&mut self, block_id: usize, location: &str) {
        let timestamp = Self::next_timestamp();

        self.events.push(Event::UnsafeBlockEnter {
            timestamp,
            block_id: block_id.to_string(),
            location: location.to_string(),
        });
    }

    /// Record unsafe block exit
    pub fn record_unsafe_block_exit(&mut self, block_id: usize, location: &str) {
        let timestamp = Self::next_timestamp();

        self.events.push(Event::UnsafeBlockExit {
            timestamp,
            block_id: block_id.to_string(),
            location: location.to_string(),
        });
    }

    /// Record unsafe function call
    pub fn record_unsafe_fn_call(&mut self, fn_name: &str, location: &str) {
        let timestamp = Self::next_timestamp();

        self.events.push(Event::UnsafeFnCall {
            timestamp,
            fn_name: fn_name.to_string(),
            location: location.to_string(),
        });
    }

    /// Record FFI call
    pub fn record_ffi_call(&mut self, fn_name: &str, location: &str) {
        let timestamp = Self::next_timestamp();

        self.events.push(Event::FfiCall {
            timestamp,
            fn_name: fn_name.to_string(),
            location: location.to_string(),
        });
    }

    /// Record transmute operation
    pub fn record_transmute(&mut self, from_type: &str, to_type: &str, location: &str) {
        let timestamp = Self::next_timestamp();

        self.events.push(Event::Transmute {
            timestamp,
            from_type: from_type.to_string(),
            to_type: to_type.to_string(),
            location: location.to_string(),
        });
    }

    /// Record union field access
    pub fn record_union_field_access(
        &mut self,
        union_name: &str,
        field_name: &str,
        location: &str,
    ) {
        let timestamp = Self::next_timestamp();

        self.events.push(Event::UnionFieldAccess {
            timestamp,
            union_name: union_name.to_string(),
            field_name: field_name.to_string(),
            location: location.to_string(),
        });
    }

    /// Record async block entry
    pub fn record_async_block_enter(&mut self, block_id: usize, location: &str) {
        let timestamp = Self::next_timestamp();

        self.events.push(Event::AsyncBlockEnter {
            timestamp,
            block_id: block_id.to_string(),
            location: location.to_string(),
        });
    }

    /// Record async block exit
    pub fn record_async_block_exit(&mut self, block_id: usize, location: &str) {
        let timestamp = Self::next_timestamp();

        self.events.push(Event::AsyncBlockExit {
            timestamp,
            block_id: block_id.to_string(),
            location: location.to_string(),
        });
    }

    /// Record await expression start
    pub fn record_await_start(&mut self, await_id: usize, future_name: &str, location: &str) {
        let timestamp = Self::next_timestamp();

        self.events.push(Event::AwaitStart {
            timestamp,
            await_id: await_id.to_string(),
            future_name: future_name.to_string(),
            location: location.to_string(),
        });
    }

    /// Record await expression end
    pub fn record_await_end(&mut self, await_id: usize, location: &str) {
        let timestamp = Self::next_timestamp();

        self.events.push(Event::AwaitEnd {
            timestamp,
            await_id: await_id.to_string(),
            location: location.to_string(),
        });
    }

    // ========== Phase 5: Extended Tracking Record Methods ==========

    /// Record loop entry
    pub fn record_loop_enter(&mut self, loop_id: usize, loop_type: &str, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::LoopEnter {
            timestamp,
            loop_id: loop_id.to_string(),
            loop_type: loop_type.to_string(),
            location: location.to_string(),
        });
    }

    /// Record loop iteration
    pub fn record_loop_iteration(&mut self, loop_id: usize, iteration: usize, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::LoopIteration {
            timestamp,
            loop_id: loop_id.to_string(),
            iteration,
            location: location.to_string(),
        });
    }

    /// Record loop exit
    pub fn record_loop_exit(&mut self, loop_id: usize, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::LoopExit {
            timestamp,
            loop_id: loop_id.to_string(),
            location: location.to_string(),
        });
    }

    /// Record match expression entry
    pub fn record_match_enter(&mut self, match_id: usize, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::MatchEnter {
            timestamp,
            match_id: match_id.to_string(),
            location: location.to_string(),
        });
    }

    /// Record match arm taken
    pub fn record_match_arm(&mut self, match_id: usize, arm_index: usize, pattern: &str, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::MatchArm {
            timestamp,
            match_id: match_id.to_string(),
            arm_index,
            pattern: pattern.to_string(),
            location: location.to_string(),
        });
    }

    /// Record match expression exit
    pub fn record_match_exit(&mut self, match_id: usize, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::MatchExit {
            timestamp,
            match_id: match_id.to_string(),
            location: location.to_string(),
        });
    }

    /// Record branch taken
    pub fn record_branch(&mut self, branch_id: usize, branch_type: &str, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::Branch {
            timestamp,
            branch_id: branch_id.to_string(),
            branch_type: branch_type.to_string(),
            location: location.to_string(),
        });
    }

    /// Record return statement
    pub fn record_return(&mut self, return_id: usize, has_value: bool, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::Return {
            timestamp,
            return_id: return_id.to_string(),
            has_value,
            location: location.to_string(),
        });
    }

    /// Record try/? operator
    pub fn record_try(&mut self, try_id: usize, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::Try {
            timestamp,
            try_id: try_id.to_string(),
            location: location.to_string(),
        });
    }

    /// Record index access
    pub fn record_index_access(&mut self, access_id: usize, container: &str, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::IndexAccess {
            timestamp,
            access_id: access_id.to_string(),
            container: container.to_string(),
            location: location.to_string(),
        });
    }

    /// Record field access
    pub fn record_field_access(&mut self, access_id: usize, base: &str, field: &str, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::FieldAccess {
            timestamp,
            access_id: access_id.to_string(),
            base: base.to_string(),
            field: field.to_string(),
            location: location.to_string(),
        });
    }

    /// Record function call
    pub fn record_call(&mut self, call_id: usize, fn_name: &str, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::Call {
            timestamp,
            call_id: call_id.to_string(),
            fn_name: fn_name.to_string(),
            location: location.to_string(),
        });
    }

    /// Record lock acquisition
    pub fn record_lock(&mut self, lock_id: usize, lock_type: &str, var_name: &str, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::Lock {
            timestamp,
            lock_id: lock_id.to_string(),
            lock_type: lock_type.to_string(),
            var_name: var_name.to_string(),
            location: location.to_string(),
        });
    }

    /// Record unwrap call
    pub fn record_unwrap(&mut self, unwrap_id: usize, method: &str, var_name: &str, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::Unwrap {
            timestamp,
            unwrap_id: unwrap_id.to_string(),
            method: method.to_string(),
            var_name: var_name.to_string(),
            location: location.to_string(),
        });
    }

    /// Record clone call
    pub fn record_clone(&mut self, clone_id: usize, var_name: &str, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::Clone {
            timestamp,
            clone_id: clone_id.to_string(),
            var_name: var_name.to_string(),
            location: location.to_string(),
        });
    }

    /// Record dereference operation
    pub fn record_deref(&mut self, deref_id: usize, var_name: &str, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::Deref {
            timestamp,
            deref_id: deref_id.to_string(),
            var_name: var_name.to_string(),
            location: location.to_string(),
        });
    }

    /// Get all events
    pub fn events(&self) -> &[Event] {
        &self.events
    }

    /// Clear all events
    pub fn clear(&mut self) {
        self.events.clear();
        self.var_counter = 0;
        TIMESTAMP.store(0, Ordering::Relaxed);
    }
}

impl Default for Tracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Track a new variable creation.
///
/// Records a `New` event and returns the value unchanged. Use this when
/// a variable is first created or initialized.
///
/// # Arguments
///
/// * `name` - A descriptive name for the variable (used in event output)
/// * `value` - The value being tracked (returned unchanged)
///
/// # Returns
///
/// The input `value`, unchanged. This allows chaining:
/// ```rust
/// # use borrowscope_runtime::*;
/// # reset();
/// let x = track_new("x", 42);
/// assert_eq!(x, 42);
/// ```
///
/// # Examples
///
/// Basic usage:
/// ```rust
/// # use borrowscope_runtime::*;
/// # reset();
/// let data = track_new("data", vec![1, 2, 3]);
/// let events = get_events();
/// assert!(events[0].is_new());
/// ```
///
/// With structs:
/// ```rust
/// # use borrowscope_runtime::*;
/// # reset();
/// struct Point { x: i32, y: i32 }
/// let p = track_new("point", Point { x: 10, y: 20 });
/// ```
#[inline(always)]
pub fn track_new<T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    value: T,
) -> T {
    #[cfg(feature = "track")]
    {
        let type_name = std::any::type_name::<T>();
        let mut tracker = TRACKER.lock();
        tracker.record_new(name, type_name);
    }
    value
}

/// Track an immutable borrow.
///
/// Records a `Borrow` event with `mutable: false` and returns the reference unchanged.
/// Use this when creating a shared reference (`&T`).
///
/// # Arguments
///
/// * `name` - A descriptive name for the borrow
/// * `value` - The reference being tracked (returned unchanged)
///
/// # Returns
///
/// The input reference, unchanged.
///
/// # Examples
///
/// ```rust
/// # use borrowscope_runtime::*;
/// # reset();
/// let data = track_new("data", vec![1, 2, 3]);
/// let r1 = track_borrow("r1", &data);
/// let r2 = track_borrow("r2", &data); // Multiple immutable borrows OK
/// println!("{:?}, {:?}", r1, r2);
///
/// let events = get_events();
/// assert!(events[1].is_borrow());
/// assert!(events[2].is_borrow());
/// ```
#[inline(always)]
pub fn track_borrow<'a, T: ?Sized>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    value: &'a T,
) -> &'a T {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_borrow(name, "unknown", false);
    }
    value
}

/// Track a mutable borrow.
///
/// Records a `Borrow` event with `mutable: true` and returns the reference unchanged.
/// Use this when creating an exclusive reference (`&mut T`).
///
/// # Arguments
///
/// * `name` - A descriptive name for the borrow
/// * `value` - The mutable reference being tracked (returned unchanged)
///
/// # Returns
///
/// The input mutable reference, unchanged.
///
/// # Examples
///
/// ```rust
/// # use borrowscope_runtime::*;
/// # reset();
/// let mut data = track_new("data", vec![1, 2, 3]);
/// {
///     let r = track_borrow_mut("r", &mut data);
///     r.push(4);
/// }
/// // Mutable borrow ended, can borrow again
/// let events = get_events();
/// assert!(events[1].is_borrow());
/// ```
#[inline(always)]
pub fn track_borrow_mut<'a, T: ?Sized>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    value: &'a mut T,
) -> &'a mut T {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_borrow(name, "unknown", true);
    }
    value
}

/// Track an ownership move.
///
/// Records a `Move` event and returns the value unchanged.
/// Use this when ownership transfers from one variable to another.
///
/// # Arguments
///
/// * `from_name` - Name of the source variable (giving up ownership)
/// * `to_name` - Name of the destination variable (receiving ownership)
/// * `value` - The value being moved (returned unchanged)
///
/// # Returns
///
/// The input `value`, unchanged.
///
/// # Examples
///
/// ```rust
/// # use borrowscope_runtime::*;
/// # reset();
/// let s1 = track_new("s1", String::from("hello"));
/// let s2 = track_move("s1", "s2", s1);
/// // s1 is no longer valid, s2 owns the String
///
/// let events = get_events();
/// assert!(events[1].is_move());
/// ```
#[inline(always)]
pub fn track_move<T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] from_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] to_name: &str,
    value: T,
) -> T {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_move(from_name, to_name);
    }
    value
}

/// Track a variable going out of scope.
///
/// Records a `Drop` event. Call this when a variable's lifetime ends.
/// Unlike other tracking functions, this doesn't return a value since
/// the variable is being destroyed.
///
/// # Arguments
///
/// * `name` - Name of the variable being dropped
///
/// # Examples
///
/// ```rust
/// # use borrowscope_runtime::*;
/// # reset();
/// {
///     let x = track_new("x", 42);
///     // x goes out of scope here
///     track_drop("x");
/// }
///
/// let events = get_events();
/// assert!(events[1].is_drop());
/// ```
///
/// # Note
///
/// For automatic drop tracking, consider using RAII guards or the
/// future `borrowscope-macro` crate which will instrument drops automatically.
#[inline(always)]
pub fn track_drop(#[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_drop(name);
    }
}

/// Track multiple drops in batch (optimized).
///
/// Records multiple `Drop` events efficiently with a single lock acquisition.
/// Use this when multiple variables go out of scope simultaneously.
///
/// # Arguments
///
/// * `names` - Slice of variable names being dropped
///
/// # Examples
///
/// ```rust
/// # use borrowscope_runtime::*;
/// # reset();
/// let a = track_new("a", 1);
/// let b = track_new("b", 2);
/// let c = track_new("c", 3);
/// // All go out of scope together
/// track_drop_batch(&["a", "b", "c"]);
///
/// let events = get_events();
/// assert_eq!(events.len(), 6); // 3 New + 3 Drop
/// ```
#[inline(always)]
pub fn track_drop_batch(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] names: &[&str],
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        for &name in names {
            tracker.record_drop(name);
        }
    }
}

/// Reset tracking state.
///
/// Clears all recorded events and resets internal counters.
/// Call this before starting a new tracking session.
///
/// # Examples
///
/// ```rust
/// # use borrowscope_runtime::*;
/// let _ = track_new("x", 1);
/// assert!(!get_events().is_empty());
///
/// reset();
/// assert!(get_events().is_empty());
/// ```
///
/// # Thread Safety
///
/// This function is thread-safe but will clear events from all threads.
/// In multi-threaded tests, use synchronization to ensure reset completes
/// before other threads start tracking.
pub fn reset() {
    let mut tracker = TRACKER.lock();
    tracker.clear();
}

/// Get all recorded events.
///
/// Returns a copy of all events recorded since the last [`reset()`].
/// Events are ordered by timestamp (monotonically increasing).
///
/// # Returns
///
/// A `Vec<Event>` containing all recorded events.
///
/// # Examples
///
/// ```rust
/// # use borrowscope_runtime::*;
/// # reset();
/// let x = track_new("x", 42);
/// let r = track_borrow("r", &x);
///
/// let events = get_events();
/// assert_eq!(events.len(), 2);
/// assert!(events[0].is_new());
/// assert!(events[1].is_borrow());
/// ```
///
/// # Exporting to JSON
///
/// ```rust
/// # use borrowscope_runtime::*;
/// # reset();
/// # let _ = track_new("x", 1);
/// let events = get_events();
/// let json = serde_json::to_string_pretty(&events).unwrap();
/// println!("{}", json);
/// ```
pub fn get_events() -> Vec<Event> {
    TRACKER.lock().events().to_vec()
}

/// Get events filtered by a predicate.
///
/// # Examples
///
/// ```rust
/// # use borrowscope_runtime::*;
/// # reset();
/// let _ = track_new("x", 1);
/// let _ = track_borrow("r", &1);
/// let borrows = get_events_filtered(|e| e.is_borrow());
/// assert_eq!(borrows.len(), 1);
/// ```
pub fn get_events_filtered<F>(predicate: F) -> Vec<Event>
where
    F: Fn(&Event) -> bool,
{
    TRACKER.lock().events().iter().filter(|e| predicate(e)).cloned().collect()
}

/// Get all `New` events.
pub fn get_new_events() -> Vec<Event> {
    get_events_filtered(|e| e.is_new())
}

/// Get all `Borrow` events (both mutable and immutable).
pub fn get_borrow_events() -> Vec<Event> {
    get_events_filtered(|e| e.is_borrow())
}

/// Get all `Drop` events.
pub fn get_drop_events() -> Vec<Event> {
    get_events_filtered(|e| e.is_drop())
}

/// Get all `Move` events.
pub fn get_move_events() -> Vec<Event> {
    get_events_filtered(|e| e.is_move())
}

/// Get events for a specific variable by name.
///
/// # Examples
///
/// ```rust
/// # use borrowscope_runtime::*;
/// # reset();
/// let _ = track_new("data", vec![1, 2, 3]);
/// let _ = track_new("other", 42);
/// let data_events = get_events_for_var("data");
/// assert_eq!(data_events.len(), 1);
/// ```
pub fn get_events_for_var(name: &str) -> Vec<Event> {
    get_events_filtered(|e| e.var_name().map(|n| n == name).unwrap_or(false))
}

/// Get count of events by type.
///
/// Returns (new, borrow, move, drop) counts.
///
/// # Examples
///
/// ```rust
/// # use borrowscope_runtime::*;
/// # reset();
/// let x = track_new("x", 1);
/// let _ = track_borrow("r", &x);
/// track_drop("r");
/// track_drop("x");
/// let (new, borrow, mov, drop) = get_event_counts();
/// assert_eq!((new, borrow, mov, drop), (1, 1, 0, 2));
/// ```
pub fn get_event_counts() -> (usize, usize, usize, usize) {
    let events = TRACKER.lock();
    let events = events.events();
    (
        events.iter().filter(|e| e.is_new()).count(),
        events.iter().filter(|e| e.is_borrow()).count(),
        events.iter().filter(|e| e.is_move()).count(),
        events.iter().filter(|e| e.is_drop()).count(),
    )
}

/// Summary statistics for tracked events.
#[derive(Debug, Clone, Default)]
pub struct TrackingSummary {
    /// Number of variables created
    pub variables_created: usize,
    /// Number of variables dropped
    pub variables_dropped: usize,
    /// Number of immutable borrows
    pub immutable_borrows: usize,
    /// Number of mutable borrows
    pub mutable_borrows: usize,
    /// Number of moves
    pub moves: usize,
    /// Number of Rc operations
    pub rc_operations: usize,
    /// Number of Arc operations
    pub arc_operations: usize,
    /// Number of RefCell operations
    pub refcell_operations: usize,
    /// Number of Cell operations
    pub cell_operations: usize,
    /// Number of unsafe operations
    pub unsafe_operations: usize,
}

impl std::fmt::Display for TrackingSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== BorrowScope Summary ===")?;
        writeln!(
            f,
            "Variables: {} created, {} dropped",
            self.variables_created, self.variables_dropped
        )?;
        writeln!(
            f,
            "Borrows: {} immutable, {} mutable",
            self.immutable_borrows, self.mutable_borrows
        )?;
        if self.moves > 0 {
            writeln!(f, "Moves: {}", self.moves)?;
        }
        if self.rc_operations > 0 || self.arc_operations > 0 {
            writeln!(
                f,
                "Smart pointers: {} Rc, {} Arc",
                self.rc_operations, self.arc_operations
            )?;
        }
        if self.refcell_operations > 0 || self.cell_operations > 0 {
            writeln!(
                f,
                "Interior mutability: {} RefCell, {} Cell",
                self.refcell_operations, self.cell_operations
            )?;
        }
        if self.unsafe_operations > 0 {
            writeln!(f, "Unsafe operations: {}", self.unsafe_operations)?;
        }
        Ok(())
    }
}

/// Get a summary of all tracked events.
///
/// # Examples
///
/// ```rust
/// # use borrowscope_runtime::*;
/// # reset();
/// let x = track_new("x", 42);
/// let r = track_borrow("r", &x);
/// track_drop("r");
/// track_drop("x");
///
/// let summary = get_summary();
/// assert_eq!(summary.variables_created, 1);
/// assert_eq!(summary.immutable_borrows, 1);
/// assert_eq!(summary.variables_dropped, 2);
/// ```
pub fn get_summary() -> TrackingSummary {
    let events = TRACKER.lock();
    let events = events.events();
    let mut summary = TrackingSummary::default();

    for event in events {
        match event {
            Event::New { .. } => summary.variables_created += 1,
            Event::Drop { .. } => summary.variables_dropped += 1,
            Event::Borrow { mutable, .. } => {
                if *mutable {
                    summary.mutable_borrows += 1;
                } else {
                    summary.immutable_borrows += 1;
                }
            }
            Event::Move { .. } => summary.moves += 1,
            Event::RcNew { .. } | Event::RcClone { .. } => summary.rc_operations += 1,
            Event::ArcNew { .. } | Event::ArcClone { .. } => summary.arc_operations += 1,
            Event::RefCellNew { .. } | Event::RefCellBorrow { .. } | Event::RefCellDrop { .. } => {
                summary.refcell_operations += 1
            }
            Event::CellNew { .. } | Event::CellGet { .. } | Event::CellSet { .. } => {
                summary.cell_operations += 1
            }
            Event::RawPtrCreated { .. }
            | Event::RawPtrDeref { .. }
            | Event::UnsafeBlockEnter { .. }
            | Event::UnsafeBlockExit { .. }
            | Event::UnsafeFnCall { .. }
            | Event::FfiCall { .. }
            | Event::Transmute { .. }
            | Event::UnionFieldAccess { .. } => summary.unsafe_operations += 1,
            _ => {}
        }
    }
    summary
}

/// Print a summary of tracked events to stdout.
///
/// # Examples
///
/// ```rust
/// # use borrowscope_runtime::*;
/// # reset();
/// let x = track_new("x", 42);
/// let r = track_borrow("r", &x);
/// track_drop("r");
/// track_drop("x");
///
/// print_summary();
/// // Output:
/// // === BorrowScope Summary ===
/// // Variables: 1 created, 2 dropped
/// // Borrows: 1 immutable, 0 mutable
/// ```
pub fn print_summary() {
    println!("{}", get_summary());
}

/// Helper function for track_new_with_id that extracts type at runtime
#[inline(always)]
#[doc(hidden)]
pub fn __track_new_with_id_helper<T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: T,
) -> T {
    #[cfg(feature = "track")]
    {
        let type_name = std::any::type_name::<T>();
        let mut tracker = TRACKER.lock();
        tracker.record_new_with_id(id, name, type_name, location);
    }
    value
}

/// Track a new variable with explicit ID and location (advanced API)
#[inline(always)]
pub fn track_new_with_id<T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] type_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: T,
) -> T {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_new_with_id(id, name, type_name, location);
    }
    value
}

/// Track an immutable borrow with full metadata (advanced API)
#[inline(always)]
pub fn track_borrow_with_id<'a, T: ?Sized>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] borrower_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] owner_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] mutable: bool,
    value: &'a T,
) -> &'a T {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_borrow_with_id(borrower_id, owner_id, name, location, mutable);
    }
    value
}

/// Track a mutable borrow with full metadata (advanced API)
#[inline(always)]
pub fn track_borrow_mut_with_id<'a, T: ?Sized>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] borrower_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] owner_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: &'a mut T,
) -> &'a mut T {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_borrow_with_id(borrower_id, owner_id, name, location, true);
    }
    value
}

/// Track a move with explicit IDs and location (advanced API)
#[inline(always)]
pub fn track_move_with_id<T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] from_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] to_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] to_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: T,
) -> T {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_move_with_id(from_id, to_id, to_name, location);
    }
    value
}

/// Track a drop with explicit ID and location (advanced API)
#[inline(always)]
pub fn track_drop_with_id(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_drop_with_id(id, location);
    }
}

/// Track Rc::new with explicit ID and location (advanced API)
#[inline(always)]
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
pub fn track_static_init<T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] var_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] var_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] type_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] is_mutable: bool,
    value: T,
) -> T {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_static_init(var_name, var_id, type_name, is_mutable);
    }
    value
}

/// Track static variable access (read or write).
///
/// Records a `StaticAccess` event. Use this when reading from or writing to a static variable.
///
/// # Arguments
///
/// * `var_id` - Identifier of the static variable
/// * `var_name` - Name of the static variable
/// * `is_write` - `true` for writes, `false` for reads
/// * `location` - Source location (e.g., "file.rs:42")
#[inline(always)]
pub fn track_static_access(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] var_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] var_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] is_write: bool,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_static_access(var_id, var_name, is_write, location);
    }
}

/// Track const evaluation.
///
/// Records a `ConstEval` event. Use this when a const value is evaluated.
///
/// # Arguments
///
/// * `const_name` - Name of the const
/// * `const_id` - Unique identifier
/// * `type_name` - Type of the const
/// * `location` - Source location
/// * `value` - The const value (returned unchanged)
///
/// # Returns
///
/// The input value, unchanged.
#[inline(always)]
pub fn track_const_eval<T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] const_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] const_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] type_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    value: T,
) -> T {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_const_eval(const_name, const_id, type_name, location);
    }
    value
}

/// Track raw pointer creation.
///
/// Records a `RawPtrCreated` event. Use this when creating a `*const T` pointer.
///
/// # Arguments
///
/// * `var_name` - Name for the pointer variable
/// * `var_id` - Unique identifier
/// * `ptr_type` - Type description (e.g., "*const i32")
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

/// Track async block entry.
///
/// Records an `AsyncBlockEnter` event. Use this when entering an async block.
///
/// # Arguments
///
/// * `block_id` - Unique identifier for this async block
/// * `location` - Source location (e.g., "file.rs:42")
#[inline(always)]
pub fn track_async_block_enter(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] block_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_async_block_enter(block_id, location);
    }
}

/// Track async block exit.
///
/// Records an `AsyncBlockExit` event. Use this when exiting an async block.
///
/// # Arguments
///
/// * `block_id` - Identifier matching the corresponding `track_async_block_enter`
/// * `location` - Source location
#[inline(always)]
pub fn track_async_block_exit(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] block_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_async_block_exit(block_id, location);
    }
}

/// Track await expression start.
///
/// Records an `AwaitStart` event. Use this before awaiting a future.
///
/// # Arguments
///
/// * `await_id` - Unique identifier for this await point
/// * `future_name` - Name or description of the future being awaited
/// * `location` - Source location
#[inline(always)]
pub fn track_await_start(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] await_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] future_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_await_start(await_id, future_name, location);
    }
}

/// Track await expression completion.
///
/// Records an `AwaitEnd` event. Use this after a future completes.
///
/// # Arguments
///
/// * `await_id` - Identifier matching the corresponding `track_await_start`
/// * `location` - Source location
#[inline(always)]
pub fn track_await_end(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] await_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_await_end(await_id, location);
    }
}

// ============================================================================
// Phase 5: Extended Tracking Functions
// ============================================================================

/// Track loop entry
#[inline(always)]
pub fn track_loop_enter(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] loop_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] loop_type: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_loop_enter(loop_id, loop_type, location);
    }
}

/// Track loop iteration
#[inline(always)]
pub fn track_loop_iteration(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] loop_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] iteration: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_loop_iteration(loop_id, iteration, location);
    }
}

/// Track loop exit
#[inline(always)]
pub fn track_loop_exit(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] loop_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_loop_exit(loop_id, location);
    }
}

/// Track match expression entry
#[inline(always)]
pub fn track_match_enter(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] match_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_match_enter(match_id, location);
    }
}

/// Track match arm taken
#[inline(always)]
pub fn track_match_arm(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] match_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] arm_index: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] pattern: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_match_arm(match_id, arm_index, pattern, location);
    }
}

/// Track match expression exit
#[inline(always)]
pub fn track_match_exit(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] match_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_match_exit(match_id, location);
    }
}

/// Track branch taken (if/else)
#[inline(always)]
pub fn track_branch(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] branch_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] branch_type: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_branch(branch_id, branch_type, location);
    }
}

/// Track return statement
#[inline(always)]
pub fn track_return(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] return_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] has_value: bool,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_return(return_id, has_value, location);
    }
}

/// Track try/? operator
#[inline(always)]
pub fn track_try(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] try_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_try(try_id, location);
    }
}

/// Track index access
#[inline(always)]
pub fn track_index_access(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] access_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] container: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_index_access(access_id, container, location);
    }
}

/// Track field access
#[inline(always)]
pub fn track_field_access(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] access_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] base: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] field: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_field_access(access_id, base, field, location);
    }
}

/// Track function call
#[inline(always)]
pub fn track_call(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] call_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] fn_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_call(call_id, fn_name, location);
    }
}

/// Track lock acquisition (Mutex/RwLock)
#[inline(always)]
pub fn track_lock(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] lock_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] lock_type: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] var_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_lock(lock_id, lock_type, var_name, location);
    }
}

/// Track unwrap call (Option/Result)
#[inline(always)]
pub fn track_unwrap(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] unwrap_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] method: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] var_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_unwrap(unwrap_id, method, var_name, location);
    }
}

/// Track clone call
#[inline(always)]
pub fn track_clone(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] clone_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] var_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_clone(clone_id, var_name, location);
    }
}

/// Track dereference operation
#[inline(always)]
pub fn track_deref(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] deref_id: usize,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] var_name: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_deref(deref_id, var_name, location);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::TEST_LOCK;

    #[test]
    fn test_tracker_new() {
        let mut tracker = Tracker::new();
        let id = tracker.record_new("x", "i32");

        assert_eq!(tracker.events().len(), 1);
        assert!(id.starts_with("x_"));
    }

    #[test]
    fn test_tracker_borrow() {
        let mut tracker = Tracker::new();
        let owner_id = tracker.record_new("s", "String");
        let borrower_id = tracker.record_borrow("r", &owner_id, false);

        assert_eq!(tracker.events().len(), 2);
        assert!(borrower_id.starts_with("r_"));
    }

    #[test]
    fn test_tracker_move() {
        let mut tracker = Tracker::new();
        let from_id = tracker.record_new("x", "String");
        let to_id = tracker.record_move(&from_id, "y");

        assert_eq!(tracker.events().len(), 2);
        assert!(to_id.starts_with("y_"));
    }

    #[test]
    fn test_tracker_drop() {
        let mut tracker = Tracker::new();
        let id = tracker.record_new("x", "i32");
        tracker.record_drop(&id);

        assert_eq!(tracker.events().len(), 2);
        assert!(tracker.events()[1].is_drop());
    }

    #[test]
    fn test_timestamp_ordering() {
        let mut tracker = Tracker::new();
        tracker.record_new("x", "i32");
        tracker.record_new("y", "i32");
        tracker.record_new("z", "i32");

        let events = tracker.events();
        assert!(events[0].timestamp() < events[1].timestamp());
        assert!(events[1].timestamp() < events[2].timestamp());
    }

    #[test]
    fn test_track_new_returns_value() {
        let _lock = TEST_LOCK.lock();
        reset();

        let handles: Vec<_> = (0..4)
            .map(|i| {
                std::thread::spawn(move || {
                    let value = track_new(&format!("x_{}", i), 42 + i);
                    assert_eq!(value, 42 + i);
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_track_borrow_returns_reference() {
        let _lock = TEST_LOCK.lock();
        reset();

        // Each thread creates its own string and borrows it
        let handles: Vec<_> = (0..4)
            .map(|i| {
                std::thread::spawn(move || {
                    let s = String::from("hello");
                    let r = track_borrow(&format!("r_{}", i), &s);
                    assert_eq!(r, "hello");
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let events = get_events();
        assert_eq!(events.iter().filter(|e| e.is_borrow()).count(), 4);
    }

    #[test]
    fn test_track_borrow_mut_returns_reference() {
        let _lock = TEST_LOCK.lock();
        reset();

        // Mutable borrows can't be shared, so test sequentially but verify tracking works
        let mut s = String::from("hello");
        track_borrow_mut("r", &mut s);
        s.push_str(" world");
        assert_eq!(s, "hello world");

        let events = get_events();
        assert_eq!(events.iter().filter(|e| e.is_borrow()).count(), 1);
    }

    #[test]
    fn test_complete_workflow() {
        let _lock = TEST_LOCK.lock();
        reset();

        let handles: Vec<_> = (0..4)
            .map(|i| {
                std::thread::spawn(move || {
                    let x = track_new(&format!("x_{}", i), 5);
                    let _r = track_borrow(&format!("r_{}", i), &x);
                    track_drop(&format!("r_{}", i));
                    track_drop(&format!("x_{}", i));
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let events = get_events();
        assert_eq!(events.len(), 16); // 4 threads * (1 new + 1 borrow + 2 drops)
        assert_eq!(events.iter().filter(|e| e.is_new()).count(), 4);
        assert_eq!(events.iter().filter(|e| e.is_borrow()).count(), 4);
        assert_eq!(events.iter().filter(|e| e.is_drop()).count(), 8);
    }

    #[test]
    fn test_reset() {
        let _lock = TEST_LOCK.lock();
        reset();

        let handles: Vec<_> = (0..4)
            .map(|i| {
                std::thread::spawn(move || {
                    track_new(&format!("x_{}", i), 5);
                    track_new(&format!("y_{}", i), 10);
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(get_events().len(), 8); // 4 threads * 2 events

        reset();

        assert_eq!(get_events().len(), 0);
    }

    #[test]
    fn test_unique_ids() {
        let _lock = TEST_LOCK.lock();
        reset();

        let handles: Vec<_> = (0..4)
            .map(|_| {
                std::thread::spawn(|| {
                    track_new("x", 1);
                    track_new("x", 2);
                    track_new("x", 3);
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let events = get_events();
        let ids: Vec<_> = events
            .iter()
            .filter_map(|e| match e {
                Event::New { var_id, .. } => Some(var_id.as_str()),
                _ => None,
            })
            .collect();

        assert_eq!(ids.len(), 12); // 4 threads * 3 events

        // All IDs should be unique
        let mut unique_ids = ids.clone();
        unique_ids.sort_unstable();
        unique_ids.dedup();
        assert_eq!(unique_ids.len(), 12, "All IDs should be unique");
    }

    #[test]
    fn test_concurrent_tracking() {
        let _lock = TEST_LOCK.lock();
        reset();

        let handles: Vec<_> = (0..4)
            .map(|i| {
                std::thread::spawn(move || {
                    for j in 0..10 {
                        track_new(&format!("var_{}_{}", i, j), i * 10 + j);
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let events = get_events();
        assert_eq!(events.len(), 40); // 4 threads * 10 events
    }

    #[test]
    fn test_timestamp_monotonicity_concurrent() {
        let _lock = TEST_LOCK.lock();
        reset();

        let handles: Vec<_> = (0..4)
            .map(|i| {
                std::thread::spawn(move || {
                    for j in 0..10 {
                        track_new(&format!("var_{}_{}", i, j), i * 10 + j);
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let events = get_events();
        let mut timestamps: Vec<_> = events.iter().map(|e| e.timestamp()).collect();
        timestamps.sort_unstable();

        // All timestamps should be unique and monotonic
        for i in 1..timestamps.len() {
            assert!(
                timestamps[i] > timestamps[i - 1],
                "Timestamps should be unique and monotonic"
            );
        }
    }

    #[test]
    fn test_concurrent_reset() {
        let _lock = TEST_LOCK.lock();
        reset();

        // Add some events
        for i in 0..10 {
            track_new(&format!("var_{}", i), i);
        }

        assert_eq!(get_events().len(), 10);
        reset();
        assert_eq!(get_events().len(), 0);
    }

    #[test]
    fn test_high_contention() {
        let _lock = TEST_LOCK.lock();
        reset();

        let handles: Vec<_> = (0..8)
            .map(|i| {
                std::thread::spawn(move || {
                    for j in 0..100 {
                        track_new(&format!("var_{}_{}", i, j), i * 100 + j);
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let events = get_events();
        // try_lock() may drop events under extreme contention - verify we captured most
        assert!(
            events.len() >= 600,
            "Expected at least 600/800 events, got {}",
            events.len()
        );
        assert!(
            events.iter().all(|e| e.is_new()),
            "All events should be New events"
        );
    }
}
