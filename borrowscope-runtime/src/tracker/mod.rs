//! Core tracking functionality for recording ownership and borrowing events.

use crate::event::Event;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

lazy_static! {
    /// Global tracker instance
    pub(crate) static ref TRACKER: Mutex<Tracker> = Mutex::new(Tracker::new());
}

/// Global timestamp counter
pub(crate) static TIMESTAMP: AtomicU64 = AtomicU64::new(0);

/// The main tracker that records events
pub struct Tracker {
    /// All recorded events
    pub(crate) events: Vec<Event>,
    /// Counter for generating unique variable IDs
    pub(crate) var_counter: u64,
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
            location: None,
        });
    }

    /// Record a drop with precise location from static analysis
    #[cfg_attr(not(feature = "track"), allow(dead_code))]
    pub fn record_drop_at(&mut self, var_id: &str, location: &str) {
        let timestamp = Self::next_timestamp();

        self.events.push(Event::Drop {
            timestamp,
            var_id: var_id.to_string(),
            location: Some(location.to_string()),
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

        self.events.push(Event::Drop {
            timestamp,
            var_id,
            location: None,
        });
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
            operation_kind: None,
            operation_context: None,
        });
    }

    pub fn record_unsafe_block_enter_enriched(
        &mut self,
        block_id: usize,
        location: &str,
        kind: &str,
        context: Option<&str>,
    ) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::UnsafeBlockEnter {
            timestamp,
            block_id: block_id.to_string(),
            location: location.to_string(),
            operation_kind: Some(kind.to_string()),
            operation_context: context.map(|s| s.to_string()),
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
            live_variables: vec![],
        });
    }

    pub fn record_await_start_with_live_vars(
        &mut self,
        await_id: usize,
        future_name: &str,
        location: &str,
        live_variables: &[&str],
    ) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::AwaitStart {
            timestamp,
            await_id: await_id.to_string(),
            future_name: future_name.to_string(),
            location: location.to_string(),
            live_variables: live_variables.iter().map(|s| s.to_string()).collect(),
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
    pub fn record_match_arm(
        &mut self,
        match_id: usize,
        arm_index: usize,
        pattern: &str,
        location: &str,
    ) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::MatchArm {
            timestamp,
            match_id: match_id.to_string(),
            arm_index,
            pattern: pattern.to_string(),
            location: location.to_string(),
            bindings: vec![],
        });
    }

    pub fn record_match_arm_with_bindings(
        &mut self,
        match_id: usize,
        arm_index: usize,
        pattern: &str,
        location: &str,
        bindings: &[&str],
    ) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::MatchArm {
            timestamp,
            match_id: match_id.to_string(),
            arm_index,
            pattern: pattern.to_string(),
            location: location.to_string(),
            bindings: bindings.iter().map(|s| s.to_string()).collect(),
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
    pub fn record_field_access(
        &mut self,
        access_id: usize,
        base: &str,
        field: &str,
        location: &str,
    ) {
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
            receiver_type: None,
            result_type: None,
        });
    }

    /// Record method call with type metadata
    pub fn record_method_call(
        &mut self,
        call_id: usize,
        fn_name: &str,
        location: &str,
        receiver_type: &str,
        result_type: &str,
    ) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::Call {
            timestamp,
            call_id: call_id.to_string(),
            fn_name: fn_name.to_string(),
            location: location.to_string(),
            receiver_type: Some(receiver_type.to_string()),
            result_type: Some(result_type.to_string()),
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
    pub fn record_unwrap(
        &mut self,
        unwrap_id: usize,
        method: &str,
        var_name: &str,
        location: &str,
    ) {
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

    // =========================================================================
    // Phase 6: Additional Record Methods
    // =========================================================================

    pub fn record_break(&mut self, break_id: usize, label: Option<&str>, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::Break {
            timestamp,
            break_id: break_id.to_string(),
            loop_label: label.map(|s| s.to_string()),
            location: location.to_string(),
        });
    }

    pub fn record_continue(&mut self, continue_id: usize, label: Option<&str>, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::Continue {
            timestamp,
            continue_id: continue_id.to_string(),
            loop_label: label.map(|s| s.to_string()),
            location: location.to_string(),
        });
    }

    pub fn record_closure_create(&mut self, closure_id: usize, capture_mode: &str, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::ClosureCreate {
            timestamp,
            closure_id: closure_id.to_string(),
            capture_mode: capture_mode.to_string(),
            location: location.to_string(),
            fn_trait: None,
        });
    }

    pub fn record_closure_create_with_trait(
        &mut self,
        closure_id: usize,
        capture_mode: &str,
        location: &str,
        fn_trait: &str,
    ) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::ClosureCreate {
            timestamp,
            closure_id: closure_id.to_string(),
            capture_mode: capture_mode.to_string(),
            location: location.to_string(),
            fn_trait: Some(fn_trait.to_string()),
        });
    }

    pub fn record_struct_create(&mut self, struct_id: usize, type_name: &str, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::StructCreate {
            timestamp,
            struct_id: struct_id.to_string(),
            type_name: type_name.to_string(),
            location: location.to_string(),
        });
    }

    pub fn record_tuple_create(&mut self, tuple_id: usize, len: usize, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::TupleCreate {
            timestamp,
            tuple_id: tuple_id.to_string(),
            len,
            location: location.to_string(),
        });
    }

    pub fn record_let_else(&mut self, let_id: usize, pattern: &str, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::LetElse {
            timestamp,
            let_id: let_id.to_string(),
            pattern: pattern.to_string(),
            location: location.to_string(),
        });
    }

    pub fn record_range(&mut self, range_id: usize, range_type: &str, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::Range {
            timestamp,
            range_id: range_id.to_string(),
            range_type: range_type.to_string(),
            location: location.to_string(),
        });
    }

    pub fn record_binary_op(&mut self, op_id: usize, operator: &str, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::BinaryOp {
            timestamp,
            op_id: op_id.to_string(),
            operator: operator.to_string(),
            location: location.to_string(),
        });
    }

    pub fn record_array_create(&mut self, array_id: usize, len: usize, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::ArrayCreate {
            timestamp,
            array_id: array_id.to_string(),
            len,
            location: location.to_string(),
        });
    }

    pub fn record_type_cast(&mut self, cast_id: usize, to_type: &str, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::TypeCast {
            timestamp,
            cast_id: cast_id.to_string(),
            to_type: to_type.to_string(),
            location: location.to_string(),
        });
    }

    pub fn record_region_enter(&mut self, region_id: usize, name: &str, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::RegionEnter {
            timestamp,
            region_id: region_id.to_string(),
            name: name.to_string(),
            location: location.to_string(),
        });
    }

    pub fn record_region_exit(&mut self, region_id: usize, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::RegionExit {
            timestamp,
            region_id: region_id.to_string(),
            location: location.to_string(),
        });
    }

    // =========================================================================
    // Phase 8: Enhanced Record Methods
    // =========================================================================

    pub fn record_fn_enter(&mut self, fn_id: usize, fn_name: &str, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::FnEnter {
            timestamp,
            fn_id: fn_id.to_string(),
            fn_name: fn_name.to_string(),
            location: location.to_string(),
        });
    }

    pub fn record_fn_exit(&mut self, fn_id: usize, fn_name: &str, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::FnExit {
            timestamp,
            fn_id: fn_id.to_string(),
            fn_name: fn_name.to_string(),
            location: location.to_string(),
        });
    }

    pub fn record_closure_capture(
        &mut self,
        closure_id: usize,
        var_name: &str,
        capture_mode: &str,
        location: &str,
    ) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::ClosureCapture {
            timestamp,
            closure_id: closure_id.to_string(),
            var_name: var_name.to_string(),
            capture_mode: capture_mode.to_string(),
            location: location.to_string(),
        });
    }

    // =========================================================================
    // Phase 9: Extended Smart Pointer & Concurrency Record Methods
    // =========================================================================

    pub fn record_weak_new(
        &mut self,
        var_name: &str,
        source_id: &str,
        weak_count: usize,
        location: &str,
    ) -> String {
        let timestamp = Self::next_timestamp();
        let var_id = self.next_var_id(var_name);
        self.events.push(Event::WeakNew {
            timestamp,
            var_name: var_name.to_string(),
            var_id: var_id.clone(),
            source_id: source_id.to_string(),
            weak_count,
            location: location.to_string(),
        });
        var_id
    }

    pub fn record_weak_clone(
        &mut self,
        var_name: &str,
        source_id: &str,
        weak_count: usize,
        location: &str,
    ) -> String {
        let timestamp = Self::next_timestamp();
        let var_id = self.next_var_id(var_name);
        self.events.push(Event::WeakClone {
            timestamp,
            var_name: var_name.to_string(),
            var_id: var_id.clone(),
            source_id: source_id.to_string(),
            weak_count,
            location: location.to_string(),
        });
        var_id
    }

    pub fn record_weak_upgrade(&mut self, weak_id: &str, success: bool, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::WeakUpgrade {
            timestamp,
            weak_id: weak_id.to_string(),
            success,
            location: location.to_string(),
        });
    }

    pub fn record_box_new(&mut self, var_name: &str, type_name: &str, location: &str) -> String {
        let timestamp = Self::next_timestamp();
        let var_id = self.next_var_id(var_name);
        self.events.push(Event::BoxNew {
            timestamp,
            var_name: var_name.to_string(),
            var_id: var_id.clone(),
            type_name: type_name.to_string(),
            location: location.to_string(),
        });
        var_id
    }

    pub fn record_box_into_raw(&mut self, box_id: &str, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::BoxIntoRaw {
            timestamp,
            box_id: box_id.to_string(),
            location: location.to_string(),
        });
    }

    pub fn record_box_from_raw(&mut self, var_name: &str, location: &str) -> String {
        let timestamp = Self::next_timestamp();
        let var_id = self.next_var_id(var_name);
        self.events.push(Event::BoxFromRaw {
            timestamp,
            var_name: var_name.to_string(),
            var_id: var_id.clone(),
            location: location.to_string(),
        });
        var_id
    }

    pub fn record_lock_guard_acquire(
        &mut self,
        guard_id: &str,
        lock_id: &str,
        lock_type: &str,
        location: &str,
    ) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::LockGuardAcquire {
            timestamp,
            guard_id: guard_id.to_string(),
            lock_id: lock_id.to_string(),
            lock_type: lock_type.to_string(),
            location: location.to_string(),
        });
    }

    pub fn record_lock_guard_drop(&mut self, guard_id: &str, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::LockGuardDrop {
            timestamp,
            guard_id: guard_id.to_string(),
            location: location.to_string(),
        });
    }

    pub fn record_pin_new(&mut self, var_name: &str, location: &str) -> String {
        let timestamp = Self::next_timestamp();
        let var_id = self.next_var_id(var_name);
        self.events.push(Event::PinNew {
            timestamp,
            var_name: var_name.to_string(),
            var_id: var_id.clone(),
            location: location.to_string(),
        });
        var_id
    }

    pub fn record_pin_into_inner(&mut self, pin_id: &str, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::PinIntoInner {
            timestamp,
            pin_id: pin_id.to_string(),
            location: location.to_string(),
        });
    }

    pub fn record_cow_borrowed(&mut self, var_name: &str, location: &str) -> String {
        let timestamp = Self::next_timestamp();
        let var_id = self.next_var_id(var_name);
        self.events.push(Event::CowBorrowed {
            timestamp,
            var_name: var_name.to_string(),
            var_id: var_id.clone(),
            location: location.to_string(),
        });
        var_id
    }

    pub fn record_cow_owned(&mut self, var_name: &str, location: &str) -> String {
        let timestamp = Self::next_timestamp();
        let var_id = self.next_var_id(var_name);
        self.events.push(Event::CowOwned {
            timestamp,
            var_name: var_name.to_string(),
            var_id: var_id.clone(),
            location: location.to_string(),
        });
        var_id
    }

    pub fn record_cow_to_mut(&mut self, cow_id: &str, cloned: bool, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::CowToMut {
            timestamp,
            cow_id: cow_id.to_string(),
            cloned,
            location: location.to_string(),
        });
    }

    pub fn record_thread_spawn(&mut self, thread_id: &str, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::ThreadSpawn {
            timestamp,
            thread_id: thread_id.to_string(),
            location: location.to_string(),
        });
    }

    pub fn record_thread_join(&mut self, thread_id: &str, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::ThreadJoin {
            timestamp,
            thread_id: thread_id.to_string(),
            location: location.to_string(),
        });
    }

    pub fn record_channel_sender_new(&mut self, sender_id: &str, channel_id: &str, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::ChannelSenderNew {
            timestamp,
            sender_id: sender_id.to_string(),
            channel_id: channel_id.to_string(),
            location: location.to_string(),
        });
    }

    pub fn record_channel_receiver_new(
        &mut self,
        receiver_id: &str,
        channel_id: &str,
        location: &str,
    ) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::ChannelReceiverNew {
            timestamp,
            receiver_id: receiver_id.to_string(),
            channel_id: channel_id.to_string(),
            location: location.to_string(),
        });
    }

    pub fn record_channel_send(&mut self, sender_id: &str, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::ChannelSend {
            timestamp,
            sender_id: sender_id.to_string(),
            location: location.to_string(),
        });
    }

    pub fn record_channel_recv(&mut self, receiver_id: &str, success: bool, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::ChannelRecv {
            timestamp,
            receiver_id: receiver_id.to_string(),
            success,
            location: location.to_string(),
        });
    }

    // =========================================================================
    // OnceCell / OnceLock Record Methods
    // =========================================================================

    pub fn record_once_cell_new(&mut self, name: &str, cell_type: &str, location: &str) -> String {
        let timestamp = Self::next_timestamp();
        let var_id = self.next_var_id(name);
        self.events.push(Event::OnceCellNew {
            timestamp,
            var_name: name.to_string(),
            var_id: var_id.clone(),
            cell_type: cell_type.to_string(),
            location: location.to_string(),
        });
        var_id
    }

    pub fn record_once_cell_set(&mut self, cell_id: &str, success: bool, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::OnceCellSet {
            timestamp,
            cell_id: cell_id.to_string(),
            success,
            location: location.to_string(),
        });
    }

    pub fn record_once_cell_get(&mut self, cell_id: &str, was_initialized: bool, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::OnceCellGet {
            timestamp,
            cell_id: cell_id.to_string(),
            was_initialized,
            location: location.to_string(),
        });
    }

    pub fn record_once_cell_get_or_init(
        &mut self,
        cell_id: &str,
        was_initialized: bool,
        location: &str,
    ) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::OnceCellGetOrInit {
            timestamp,
            cell_id: cell_id.to_string(),
            was_initialized,
            location: location.to_string(),
        });
    }

    // =========================================================================
    // MaybeUninit Record Methods
    // =========================================================================

    pub fn record_maybe_uninit_new(
        &mut self,
        name: &str,
        initialized: bool,
        location: &str,
    ) -> String {
        let timestamp = Self::next_timestamp();
        let var_id = self.next_var_id(name);
        self.events.push(Event::MaybeUninitNew {
            timestamp,
            var_name: name.to_string(),
            var_id: var_id.clone(),
            initialized,
            location: location.to_string(),
        });
        var_id
    }

    pub fn record_maybe_uninit_write(&mut self, var_id: &str, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::MaybeUninitWrite {
            timestamp,
            var_id: var_id.to_string(),
            location: location.to_string(),
        });
    }

    pub fn record_maybe_uninit_assume_init(&mut self, var_id: &str, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::MaybeUninitAssumeInit {
            timestamp,
            var_id: var_id.to_string(),
            location: location.to_string(),
        });
    }

    pub fn record_maybe_uninit_assume_init_read(&mut self, var_id: &str, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::MaybeUninitAssumeInitRead {
            timestamp,
            var_id: var_id.to_string(),
            location: location.to_string(),
        });
    }

    pub fn record_maybe_uninit_assume_init_drop(&mut self, var_id: &str, location: &str) {
        let timestamp = Self::next_timestamp();
        self.events.push(Event::MaybeUninitAssumeInitDrop {
            timestamp,
            var_id: var_id.to_string(),
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

mod async_tracking;
mod concurrency;
mod control_flow;
mod core;
mod expressions;
mod interior_mut;
mod maybe_uninit;
mod query;
/// Track a new variable creation.
///
/// Records a `New` event and returns the value unchanged. Use this when
// Submodules with public tracking functions
mod sampling;
mod smart_pointers;
mod statics;
mod unsafe_code;

// Re-export all public items
pub use self::core::*;
pub use async_tracking::*;
pub use concurrency::*;
pub use control_flow::*;
pub use expressions::*;
pub use interior_mut::*;
pub use maybe_uninit::*;
pub use query::*;
pub use sampling::*;
pub use smart_pointers::*;
pub use statics::*;
pub use unsafe_code::*;

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

    // =========================================================================
    // Phase 6 Tests
    // =========================================================================

    #[test]
    fn test_track_break() {
        let _lock = TEST_LOCK.lock();
        reset();

        track_break(1, None, "test.rs:1");
        track_break(2, Some("outer"), "test.rs:2");

        let events = get_events();
        assert_eq!(events.len(), 2);

        if let Event::Break {
            break_id,
            loop_label,
            ..
        } = &events[0]
        {
            assert_eq!(break_id, "1");
            assert!(loop_label.is_none());
        } else {
            panic!("Expected Break event");
        }

        if let Event::Break {
            break_id,
            loop_label,
            ..
        } = &events[1]
        {
            assert_eq!(break_id, "2");
            assert_eq!(loop_label.as_deref(), Some("outer"));
        } else {
            panic!("Expected Break event");
        }
    }

    #[test]
    fn test_track_continue() {
        let _lock = TEST_LOCK.lock();
        reset();

        track_continue(1, None, "test.rs:1");
        track_continue(2, Some("inner"), "test.rs:2");

        let events = get_events();
        assert_eq!(events.len(), 2);

        if let Event::Continue {
            continue_id,
            loop_label,
            ..
        } = &events[0]
        {
            assert_eq!(continue_id, "1");
            assert!(loop_label.is_none());
        } else {
            panic!("Expected Continue event");
        }
    }

    #[test]
    fn test_track_struct_create() {
        let _lock = TEST_LOCK.lock();
        reset();

        track_struct_create(1, "Point", "test.rs:1");

        let events = get_events();
        assert_eq!(events.len(), 1);

        if let Event::StructCreate {
            struct_id,
            type_name,
            ..
        } = &events[0]
        {
            assert_eq!(struct_id, "1");
            assert_eq!(type_name, "Point");
        } else {
            panic!("Expected StructCreate event");
        }
    }

    #[test]
    fn test_track_tuple_create() {
        let _lock = TEST_LOCK.lock();
        reset();

        track_tuple_create(1, 3, "test.rs:1");

        let events = get_events();
        assert_eq!(events.len(), 1);

        if let Event::TupleCreate { tuple_id, len, .. } = &events[0] {
            assert_eq!(tuple_id, "1");
            assert_eq!(*len, 3);
        } else {
            panic!("Expected TupleCreate event");
        }
    }

    #[test]
    fn test_track_range() {
        let _lock = TEST_LOCK.lock();
        reset();

        track_range(1, "half_open", "test.rs:1");
        track_range(2, "closed", "test.rs:2");

        let events = get_events();
        assert_eq!(events.len(), 2);

        if let Event::Range {
            range_id,
            range_type,
            ..
        } = &events[0]
        {
            assert_eq!(range_id, "1");
            assert_eq!(range_type, "half_open");
        } else {
            panic!("Expected Range event");
        }
    }

    #[test]
    fn test_track_array_create() {
        let _lock = TEST_LOCK.lock();
        reset();

        track_array_create(1, 5, "test.rs:1");

        let events = get_events();
        assert_eq!(events.len(), 1);

        if let Event::ArrayCreate { array_id, len, .. } = &events[0] {
            assert_eq!(array_id, "1");
            assert_eq!(*len, 5);
        } else {
            panic!("Expected ArrayCreate event");
        }
    }

    #[test]
    fn test_track_type_cast() {
        let _lock = TEST_LOCK.lock();
        reset();

        track_type_cast(1, "i64", "test.rs:1");

        let events = get_events();
        assert_eq!(events.len(), 1);

        if let Event::TypeCast {
            cast_id, to_type, ..
        } = &events[0]
        {
            assert_eq!(cast_id, "1");
            assert_eq!(to_type, "i64");
        } else {
            panic!("Expected TypeCast event");
        }
    }

    #[test]
    fn test_track_closure_create() {
        let _lock = TEST_LOCK.lock();
        reset();

        track_closure_create(1, "move", "test.rs:1");

        let events = get_events();
        assert_eq!(events.len(), 1);

        if let Event::ClosureCreate {
            closure_id,
            capture_mode,
            ..
        } = &events[0]
        {
            assert_eq!(closure_id, "1");
            assert_eq!(capture_mode, "move");
        } else {
            panic!("Expected ClosureCreate event");
        }
    }

    #[test]
    fn test_track_let_else() {
        let _lock = TEST_LOCK.lock();
        reset();

        track_let_else(1, "Some(x)", "test.rs:1");

        let events = get_events();
        assert_eq!(events.len(), 1);

        if let Event::LetElse {
            let_id, pattern, ..
        } = &events[0]
        {
            assert_eq!(let_id, "1");
            assert_eq!(pattern, "Some(x)");
        } else {
            panic!("Expected LetElse event");
        }
    }

    #[test]
    fn test_track_binary_op() {
        let _lock = TEST_LOCK.lock();
        reset();

        track_binary_op(1, "+", "test.rs:1");

        let events = get_events();
        assert_eq!(events.len(), 1);

        if let Event::BinaryOp {
            op_id, operator, ..
        } = &events[0]
        {
            assert_eq!(op_id, "1");
            assert_eq!(operator, "+");
        } else {
            panic!("Expected BinaryOp event");
        }
    }

    #[test]
    fn test_track_region() {
        let _lock = TEST_LOCK.lock();
        reset();

        track_region_enter(1, "scope", "test.rs:1");
        track_region_exit(1, "test.rs:2");

        let events = get_events();
        assert_eq!(events.len(), 2);

        if let Event::RegionEnter {
            region_id, name, ..
        } = &events[0]
        {
            assert_eq!(region_id, "1");
            assert_eq!(name, "scope");
        } else {
            panic!("Expected RegionEnter event");
        }

        if let Event::RegionExit { region_id, .. } = &events[1] {
            assert_eq!(region_id, "1");
        } else {
            panic!("Expected RegionExit event");
        }
    }
}
