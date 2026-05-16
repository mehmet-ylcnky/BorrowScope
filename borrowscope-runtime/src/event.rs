//! Event types for tracking ownership operations.
//!
//! This module defines the [`Event`] enum which represents all possible
//! ownership and borrowing events that can be tracked at runtime.
//!
//! # Event Categories
//!
//! - **Basic ownership**: `New`, `Borrow`, `Move`, `Drop`
//! - **Smart pointers**: `RcNew`, `RcClone`, `ArcNew`, `ArcClone`
//! - **Box**: `BoxNew`, `BoxIntoRaw`, `BoxFromRaw`
//! - **Weak references**: `WeakNew`, `WeakClone`, `WeakUpgrade`
//! - **Pin**: `PinNew`, `PinIntoInner`
//! - **Cow**: `CowBorrowed`, `CowOwned`, `CowToMut`
//! - **Interior mutability**: `RefCellNew`, `RefCellBorrow`, `RefCellDrop`, `CellNew`, `CellGet`, `CellSet`
//! - **OnceCell/OnceLock**: `OnceCellNew`, `OnceCellSet`, `OnceCellGet`, `OnceCellGetOrInit`
//! - **MaybeUninit**: `MaybeUninitNew`, `MaybeUninitWrite`, `MaybeUninitAssumeInit`, `MaybeUninitAssumeInitRead`, `MaybeUninitAssumeInitDrop`
//! - **Threads**: `ThreadSpawn`, `ThreadJoin`
//! - **Channels**: `ChannelNew`, `ChannelSend`, `ChannelRecv`
//! - **Lock guards**: `LockGuardNew`, `LockGuardDrop`
//! - **Unsafe operations**: `RawPtrCreated`, `RawPtrDeref`, `UnsafeBlockEnter`, `UnsafeBlockExit`
//!
//! # Serialization
//!
//! All events serialize to JSON with a `type` tag for easy filtering.
//!
//! # Helper Methods
//!
//! Events provide helper methods for filtering by category:
//!
//! ```rust
//! use borrowscope_runtime::*;
//!
//! reset();
//! // ... tracking code ...
//!
//! for event in get_events() {
//!     if event.is_box() { /* Box event */ }
//!     if event.is_weak() { /* Weak reference event */ }
//!     if event.is_pin() { /* Pin event */ }
//!     if event.is_cow() { /* Cow event */ }
//!     if event.is_thread() { /* Thread event */ }
//!     if event.is_channel() { /* Channel event */ }
//!     if event.is_once_cell() { /* OnceCell/OnceLock event */ }
//!     if event.is_maybe_uninit() { /* MaybeUninit event */ }
//!     if event.is_lock_guard() { /* Lock guard event */ }
//! }
//! ```

use serde::{Deserialize, Serialize};

/// An ownership or borrowing event recorded at runtime.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    /// Variable created via [`track_new`](crate::track_new).
    New {
        timestamp: u64,
        var_name: String,
        var_id: String,
        type_name: String,
    },

    /// Variable borrowed via [`track_borrow`](crate::track_borrow).
    Borrow {
        timestamp: u64,
        borrower_name: String,
        borrower_id: String,
        owner_id: String,
        mutable: bool,
    },

    /// Ownership moved via [`track_move`](crate::track_move).
    Move {
        timestamp: u64,
        from_id: String,
        to_name: String,
        to_id: String,
    },

    /// Variable dropped via [`track_drop`](crate::track_drop).
    Drop {
        timestamp: u64,
        var_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        location: Option<String>,
    },

    /// `Rc::new` allocation with reference counting.
    RcNew {
        timestamp: u64,
        var_name: String,
        var_id: String,
        type_name: String,
        strong_count: usize,
        weak_count: usize,
    },

    /// `Rc::clone` operation (shared ownership).
    RcClone {
        timestamp: u64,
        var_name: String,
        var_id: String,
        source_id: String,
        strong_count: usize,
        weak_count: usize,
    },

    /// Arc::new allocation with atomic reference counting
    ArcNew {
        timestamp: u64,
        var_name: String,
        var_id: String,
        type_name: String,
        strong_count: usize,
        weak_count: usize,
    },

    /// Arc::clone operation (thread-safe shared ownership)
    ArcClone {
        timestamp: u64,
        var_name: String,
        var_id: String,
        source_id: String,
        strong_count: usize,
        weak_count: usize,
    },

    /// RefCell::new allocation
    RefCellNew {
        timestamp: u64,
        var_name: String,
        var_id: String,
        type_name: String,
    },

    /// RefCell::borrow or borrow_mut operation
    RefCellBorrow {
        timestamp: u64,
        borrow_id: String,
        refcell_id: String,
        is_mutable: bool,
        location: String,
    },

    /// RefCell borrow dropped (Ref/RefMut dropped)
    RefCellDrop {
        timestamp: u64,
        borrow_id: String,
        location: String,
    },

    /// Cell::new allocation
    CellNew {
        timestamp: u64,
        var_name: String,
        var_id: String,
        type_name: String,
    },

    /// Cell::get operation
    CellGet {
        timestamp: u64,
        cell_id: String,
        location: String,
    },

    /// Cell::set operation
    CellSet {
        timestamp: u64,
        cell_id: String,
        location: String,
    },

    /// Static variable initialization
    StaticInit {
        timestamp: u64,
        var_name: String,
        var_id: String,
        type_name: String,
        is_mutable: bool,
    },

    /// Static variable access (read or write)
    StaticAccess {
        timestamp: u64,
        var_id: String,
        var_name: String,
        is_write: bool,
        location: String,
    },

    /// Const evaluation (compile-time constant)
    ConstEval {
        timestamp: u64,
        const_name: String,
        const_id: String,
        type_name: String,
        location: String,
    },

    /// Raw pointer created
    RawPtrCreated {
        timestamp: u64,
        var_name: String,
        var_id: String,
        ptr_type: String,
        address: usize,
        location: String,
    },

    /// Raw pointer dereferenced
    RawPtrDeref {
        timestamp: u64,
        ptr_id: String,
        location: String,
        is_write: bool,
    },

    /// Unsafe block entered
    UnsafeBlockEnter {
        timestamp: u64,
        block_id: String,
        location: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        operation_kind: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        operation_context: Option<String>,
    },

    /// Unsafe block exited
    UnsafeBlockExit {
        timestamp: u64,
        block_id: String,
        location: String,
    },

    /// Unsafe function called
    UnsafeFnCall {
        timestamp: u64,
        fn_name: String,
        location: String,
    },

    /// FFI (Foreign Function Interface) call
    FfiCall {
        timestamp: u64,
        fn_name: String,
        location: String,
    },

    /// Transmute operation
    Transmute {
        timestamp: u64,
        from_type: String,
        to_type: String,
        location: String,
    },

    /// Union field access
    UnionFieldAccess {
        timestamp: u64,
        union_name: String,
        field_name: String,
        location: String,
    },

    /// Async block entered
    AsyncBlockEnter {
        timestamp: u64,
        block_id: String,
        location: String,
    },

    /// Async block exited
    AsyncBlockExit {
        timestamp: u64,
        block_id: String,
        location: String,
    },

    /// Await expression started
    AwaitStart {
        timestamp: u64,
        await_id: String,
        future_name: String,
        location: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        live_variables: Vec<String>,
    },

    /// Await expression completed
    AwaitEnd {
        timestamp: u64,
        await_id: String,
        location: String,
    },

    // ========== Phase 5: Extended Tracking ==========
    /// Loop entered (for, while, loop)
    LoopEnter {
        timestamp: u64,
        loop_id: String,
        loop_type: String,
        location: String,
    },

    /// Loop iteration
    LoopIteration {
        timestamp: u64,
        loop_id: String,
        iteration: usize,
        location: String,
    },

    /// Loop exited
    LoopExit {
        timestamp: u64,
        loop_id: String,
        location: String,
    },

    /// Match expression entered
    MatchEnter {
        timestamp: u64,
        match_id: String,
        location: String,
    },

    /// Match arm taken
    MatchArm {
        timestamp: u64,
        match_id: String,
        arm_index: usize,
        pattern: String,
        location: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        bindings: Vec<String>,
    },

    /// Match expression exited
    MatchExit {
        timestamp: u64,
        match_id: String,
        location: String,
    },

    /// Branch taken (if/else)
    Branch {
        timestamp: u64,
        branch_id: String,
        branch_type: String,
        location: String,
    },

    /// Return statement
    Return {
        timestamp: u64,
        return_id: String,
        has_value: bool,
        location: String,
    },

    /// Try/? operator
    Try {
        timestamp: u64,
        try_id: String,
        location: String,
    },

    /// Index access (arr\[i\])
    IndexAccess {
        timestamp: u64,
        access_id: String,
        container: String,
        location: String,
    },

    /// Field access (obj.field)
    FieldAccess {
        timestamp: u64,
        access_id: String,
        base: String,
        field: String,
        location: String,
    },

    /// Function call
    Call {
        timestamp: u64,
        call_id: String,
        fn_name: String,
        location: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        receiver_type: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        result_type: Option<String>,
    },

    /// Lock acquired (Mutex/RwLock)
    Lock {
        timestamp: u64,
        lock_id: String,
        lock_type: String,
        var_name: String,
        location: String,
    },

    /// Unwrap called (Option/Result)
    Unwrap {
        timestamp: u64,
        unwrap_id: String,
        method: String,
        var_name: String,
        location: String,
    },

    /// Clone called
    Clone {
        timestamp: u64,
        clone_id: String,
        var_name: String,
        location: String,
    },

    /// Dereference operation
    Deref {
        timestamp: u64,
        deref_id: String,
        var_name: String,
        location: String,
    },

    // ========== Phase 6: Additional Tracking ==========
    /// Break statement
    Break {
        timestamp: u64,
        break_id: String,
        loop_label: Option<String>,
        location: String,
    },

    /// Continue statement
    Continue {
        timestamp: u64,
        continue_id: String,
        loop_label: Option<String>,
        location: String,
    },

    /// Closure creation
    ClosureCreate {
        timestamp: u64,
        closure_id: String,
        capture_mode: String,
        location: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fn_trait: Option<String>,
    },

    /// Struct construction
    StructCreate {
        timestamp: u64,
        struct_id: String,
        type_name: String,
        location: String,
    },

    /// Tuple construction
    TupleCreate {
        timestamp: u64,
        tuple_id: String,
        len: usize,
        location: String,
    },

    /// Let-else pattern
    LetElse {
        timestamp: u64,
        let_id: String,
        pattern: String,
        location: String,
    },

    /// Range expression
    Range {
        timestamp: u64,
        range_id: String,
        range_type: String,
        location: String,
    },

    /// Binary operation
    BinaryOp {
        timestamp: u64,
        op_id: String,
        operator: String,
        location: String,
    },

    /// Array creation
    ArrayCreate {
        timestamp: u64,
        array_id: String,
        len: usize,
        location: String,
    },

    /// Type cast (non-pointer)
    TypeCast {
        timestamp: u64,
        cast_id: String,
        to_type: String,
        location: String,
    },

    /// Region enter
    RegionEnter {
        timestamp: u64,
        region_id: String,
        name: String,
        location: String,
    },

    /// Region exit
    RegionExit {
        timestamp: u64,
        region_id: String,
        location: String,
    },

    // =========================================================================
    // Phase 8: Enhanced Tracking Events
    // =========================================================================
    /// Function entry
    FnEnter {
        timestamp: u64,
        fn_id: String,
        fn_name: String,
        location: String,
    },

    /// Function exit
    FnExit {
        timestamp: u64,
        fn_id: String,
        fn_name: String,
        location: String,
    },

    /// Closure variable capture
    ClosureCapture {
        timestamp: u64,
        closure_id: String,
        var_name: String,
        capture_mode: String,
        location: String,
    },

    // =========================================================================
    // Phase 9: Extended Smart Pointer & Concurrency Tracking
    // =========================================================================
    /// Weak::new or Rc::downgrade
    WeakNew {
        timestamp: u64,
        var_name: String,
        var_id: String,
        source_id: String,
        weak_count: usize,
        location: String,
    },

    /// Weak::clone
    WeakClone {
        timestamp: u64,
        var_name: String,
        var_id: String,
        source_id: String,
        weak_count: usize,
        location: String,
    },

    /// Weak::upgrade attempt
    WeakUpgrade {
        timestamp: u64,
        weak_id: String,
        success: bool,
        location: String,
    },

    /// Box::new allocation
    BoxNew {
        timestamp: u64,
        var_name: String,
        var_id: String,
        type_name: String,
        location: String,
    },

    /// Box::into_raw
    BoxIntoRaw {
        timestamp: u64,
        box_id: String,
        location: String,
    },

    /// Box::from_raw
    BoxFromRaw {
        timestamp: u64,
        var_name: String,
        var_id: String,
        location: String,
    },

    /// Mutex/RwLock guard acquired
    LockGuardAcquire {
        timestamp: u64,
        guard_id: String,
        lock_id: String,
        lock_type: String,
        location: String,
    },

    /// Mutex/RwLock guard dropped
    LockGuardDrop {
        timestamp: u64,
        guard_id: String,
        location: String,
    },

    /// Pin::new
    PinNew {
        timestamp: u64,
        var_name: String,
        var_id: String,
        location: String,
    },

    /// Pin::into_inner
    PinIntoInner {
        timestamp: u64,
        pin_id: String,
        location: String,
    },

    /// Cow::Borrowed
    CowBorrowed {
        timestamp: u64,
        var_name: String,
        var_id: String,
        location: String,
    },

    /// Cow::Owned
    CowOwned {
        timestamp: u64,
        var_name: String,
        var_id: String,
        location: String,
    },

    /// Cow::to_mut (clone-on-write triggered)
    CowToMut {
        timestamp: u64,
        cow_id: String,
        cloned: bool,
        location: String,
    },

    /// Thread spawn
    ThreadSpawn {
        timestamp: u64,
        thread_id: String,
        location: String,
    },

    /// Thread join
    ThreadJoin {
        timestamp: u64,
        thread_id: String,
        location: String,
    },

    /// Channel sender created
    ChannelSenderNew {
        timestamp: u64,
        sender_id: String,
        channel_id: String,
        location: String,
    },

    /// Channel receiver created
    ChannelReceiverNew {
        timestamp: u64,
        receiver_id: String,
        channel_id: String,
        location: String,
    },

    /// Channel send
    ChannelSend {
        timestamp: u64,
        sender_id: String,
        location: String,
    },

    /// Channel receive
    ChannelRecv {
        timestamp: u64,
        receiver_id: String,
        success: bool,
        location: String,
    },

    // =========================================================================
    // OnceCell / OnceLock Events
    // =========================================================================
    /// OnceCell::new or OnceLock::new
    OnceCellNew {
        timestamp: u64,
        var_name: String,
        var_id: String,
        cell_type: String, // "OnceCell" or "OnceLock"
        location: String,
    },

    /// OnceCell::set or OnceLock::set
    OnceCellSet {
        timestamp: u64,
        cell_id: String,
        success: bool,
        location: String,
    },

    /// OnceCell::get or OnceLock::get
    OnceCellGet {
        timestamp: u64,
        cell_id: String,
        was_initialized: bool,
        location: String,
    },

    /// OnceCell::get_or_init or OnceLock::get_or_init
    OnceCellGetOrInit {
        timestamp: u64,
        cell_id: String,
        was_initialized: bool,
        location: String,
    },

    // =========================================================================
    // MaybeUninit Events
    // =========================================================================
    /// MaybeUninit::uninit or MaybeUninit::new
    MaybeUninitNew {
        timestamp: u64,
        var_name: String,
        var_id: String,
        initialized: bool,
        location: String,
    },

    /// MaybeUninit::write
    MaybeUninitWrite {
        timestamp: u64,
        var_id: String,
        location: String,
    },

    /// MaybeUninit::assume_init (unsafe)
    MaybeUninitAssumeInit {
        timestamp: u64,
        var_id: String,
        location: String,
    },

    /// MaybeUninit::assume_init_read (unsafe)
    MaybeUninitAssumeInitRead {
        timestamp: u64,
        var_id: String,
        location: String,
    },

    /// MaybeUninit::assume_init_drop (unsafe)
    MaybeUninitAssumeInitDrop {
        timestamp: u64,
        var_id: String,
        location: String,
    },

    // =========================================================================
    // Phase 10: Memory Layout Tracking
    // =========================================================================
    /// Stack variable address recorded
    StackAddr {
        timestamp: u64,
        var_name: String,
        var_id: String,
        addr: usize,
        size: usize,
        type_name: String,
        location: String,
    },

    /// Stack variable field (ptr/len/cap) recorded
    StackField {
        timestamp: u64,
        var_id: String,
        field_name: String,
        field_value: String,
        offset: usize,
    },

    /// Heap allocation address recorded
    HeapAddr {
        timestamp: u64,
        var_id: String,
        owner_name: String,
        addr: usize,
        size: usize,
        capacity: usize,
        content_preview: String,
    },

    /// Heap reallocation (Vec grow, String grow)
    HeapRealloc {
        timestamp: u64,
        var_id: String,
        old_addr: usize,
        new_addr: usize,
        old_size: usize,
        new_size: usize,
    },

    /// Padding detected between stack variables
    StackPadding {
        timestamp: u64,
        after_var: String,
        addr: usize,
        bytes: usize,
    },
}

impl Event {
    /// Get the timestamp of this event
    pub fn timestamp(&self) -> u64 {
        match self {
            Event::New { timestamp, .. }
            | Event::Borrow { timestamp, .. }
            | Event::Move { timestamp, .. }
            | Event::Drop { timestamp, .. }
            | Event::RcNew { timestamp, .. }
            | Event::RcClone { timestamp, .. }
            | Event::ArcNew { timestamp, .. }
            | Event::ArcClone { timestamp, .. }
            | Event::RefCellNew { timestamp, .. }
            | Event::RefCellBorrow { timestamp, .. }
            | Event::RefCellDrop { timestamp, .. }
            | Event::CellNew { timestamp, .. }
            | Event::CellGet { timestamp, .. }
            | Event::CellSet { timestamp, .. }
            | Event::StaticInit { timestamp, .. }
            | Event::StaticAccess { timestamp, .. }
            | Event::ConstEval { timestamp, .. }
            | Event::RawPtrCreated { timestamp, .. }
            | Event::RawPtrDeref { timestamp, .. }
            | Event::UnsafeBlockEnter { timestamp, .. }
            | Event::UnsafeBlockExit { timestamp, .. }
            | Event::UnsafeFnCall { timestamp, .. }
            | Event::FfiCall { timestamp, .. }
            | Event::Transmute { timestamp, .. }
            | Event::UnionFieldAccess { timestamp, .. }
            | Event::AsyncBlockEnter { timestamp, .. }
            | Event::AsyncBlockExit { timestamp, .. }
            | Event::AwaitStart { timestamp, .. }
            | Event::AwaitEnd { timestamp, .. }
            | Event::LoopEnter { timestamp, .. }
            | Event::LoopIteration { timestamp, .. }
            | Event::LoopExit { timestamp, .. }
            | Event::MatchEnter { timestamp, .. }
            | Event::MatchArm { timestamp, .. }
            | Event::MatchExit { timestamp, .. }
            | Event::Branch { timestamp, .. }
            | Event::Return { timestamp, .. }
            | Event::Try { timestamp, .. }
            | Event::IndexAccess { timestamp, .. }
            | Event::FieldAccess { timestamp, .. }
            | Event::Call { timestamp, .. }
            | Event::Lock { timestamp, .. }
            | Event::Unwrap { timestamp, .. }
            | Event::Clone { timestamp, .. }
            | Event::Deref { timestamp, .. }
            | Event::Break { timestamp, .. }
            | Event::Continue { timestamp, .. }
            | Event::ClosureCreate { timestamp, .. }
            | Event::StructCreate { timestamp, .. }
            | Event::TupleCreate { timestamp, .. }
            | Event::LetElse { timestamp, .. }
            | Event::Range { timestamp, .. }
            | Event::BinaryOp { timestamp, .. }
            | Event::ArrayCreate { timestamp, .. }
            | Event::TypeCast { timestamp, .. }
            | Event::RegionEnter { timestamp, .. }
            | Event::RegionExit { timestamp, .. }
            | Event::FnEnter { timestamp, .. }
            | Event::FnExit { timestamp, .. }
            | Event::ClosureCapture { timestamp, .. }
            | Event::WeakNew { timestamp, .. }
            | Event::WeakClone { timestamp, .. }
            | Event::WeakUpgrade { timestamp, .. }
            | Event::BoxNew { timestamp, .. }
            | Event::BoxIntoRaw { timestamp, .. }
            | Event::BoxFromRaw { timestamp, .. }
            | Event::LockGuardAcquire { timestamp, .. }
            | Event::LockGuardDrop { timestamp, .. }
            | Event::PinNew { timestamp, .. }
            | Event::PinIntoInner { timestamp, .. }
            | Event::CowBorrowed { timestamp, .. }
            | Event::CowOwned { timestamp, .. }
            | Event::CowToMut { timestamp, .. }
            | Event::ThreadSpawn { timestamp, .. }
            | Event::ThreadJoin { timestamp, .. }
            | Event::ChannelSenderNew { timestamp, .. }
            | Event::ChannelReceiverNew { timestamp, .. }
            | Event::ChannelSend { timestamp, .. }
            | Event::ChannelRecv { timestamp, .. }
            | Event::OnceCellNew { timestamp, .. }
            | Event::OnceCellSet { timestamp, .. }
            | Event::OnceCellGet { timestamp, .. }
            | Event::OnceCellGetOrInit { timestamp, .. }
            | Event::MaybeUninitNew { timestamp, .. }
            | Event::MaybeUninitWrite { timestamp, .. }
            | Event::MaybeUninitAssumeInit { timestamp, .. }
            | Event::MaybeUninitAssumeInitRead { timestamp, .. }
            | Event::MaybeUninitAssumeInitDrop { timestamp, .. } => *timestamp,
            | Event::StackAddr { timestamp, .. } => *timestamp,
            | Event::StackField { timestamp, .. } => *timestamp,
            | Event::HeapAddr { timestamp, .. } => *timestamp,
            | Event::HeapRealloc { timestamp, .. } => *timestamp,
            | Event::StackPadding { timestamp, .. } => *timestamp,
        }
    }

    /// Get the variable name (if applicable)
    pub fn var_name(&self) -> Option<&str> {
        match self {
            Event::New { var_name, .. }
            | Event::RcNew { var_name, .. }
            | Event::RcClone { var_name, .. }
            | Event::ArcNew { var_name, .. }
            | Event::ArcClone { var_name, .. }
            | Event::RefCellNew { var_name, .. }
            | Event::CellNew { var_name, .. }
            | Event::StaticInit { var_name, .. }
            | Event::StaticAccess { var_name, .. }
            | Event::RawPtrCreated { var_name, .. }
            | Event::ConstEval {
                const_name: var_name,
                ..
            } => Some(var_name),
            Event::Borrow { borrower_name, .. } => Some(borrower_name),
            Event::Move { to_name, .. } => Some(to_name),
            Event::Drop { var_id, .. } => Some(var_id),
            Event::RefCellBorrow { .. }
            | Event::RefCellDrop { .. }
            | Event::CellGet { .. }
            | Event::CellSet { .. }
            | Event::RawPtrDeref { .. }
            | Event::UnsafeBlockEnter { .. }
            | Event::UnsafeBlockExit { .. }
            | Event::UnsafeFnCall { .. }
            | Event::FfiCall { .. }
            | Event::Transmute { .. }
            | Event::UnionFieldAccess { .. }
            | Event::AsyncBlockEnter { .. }
            | Event::AsyncBlockExit { .. }
            | Event::AwaitStart { .. }
            | Event::AwaitEnd { .. }
            | Event::LoopEnter { .. }
            | Event::LoopIteration { .. }
            | Event::LoopExit { .. }
            | Event::MatchEnter { .. }
            | Event::MatchArm { .. }
            | Event::MatchExit { .. }
            | Event::Branch { .. }
            | Event::Return { .. }
            | Event::Try { .. }
            | Event::IndexAccess { .. }
            | Event::FieldAccess { .. }
            | Event::Call { .. }
            | Event::Lock { .. }
            | Event::Unwrap { .. }
            | Event::Clone { .. }
            | Event::Deref { .. }
            | Event::Break { .. }
            | Event::Continue { .. }
            | Event::ClosureCreate { .. }
            | Event::StructCreate { .. }
            | Event::TupleCreate { .. }
            | Event::LetElse { .. }
            | Event::Range { .. }
            | Event::BinaryOp { .. }
            | Event::ArrayCreate { .. }
            | Event::TypeCast { .. }
            | Event::RegionEnter { .. }
            | Event::RegionExit { .. }
            | Event::FnEnter { .. }
            | Event::FnExit { .. }
            | Event::ClosureCapture { .. }
            | Event::WeakUpgrade { .. }
            | Event::BoxIntoRaw { .. }
            | Event::LockGuardAcquire { .. }
            | Event::LockGuardDrop { .. }
            | Event::PinIntoInner { .. }
            | Event::CowToMut { .. }
            | Event::ThreadSpawn { .. }
            | Event::ThreadJoin { .. }
            | Event::ChannelSend { .. }
            | Event::ChannelRecv { .. }
            | Event::OnceCellSet { .. }
            | Event::OnceCellGet { .. }
            | Event::OnceCellGetOrInit { .. }
            | Event::MaybeUninitWrite { .. }
            | Event::MaybeUninitAssumeInit { .. }
            | Event::MaybeUninitAssumeInitRead { .. }
            | Event::MaybeUninitAssumeInitDrop { .. } => None,
            | Event::StackAddr { ref var_name, .. } => Some(var_name.as_str()),
            | Event::StackField { .. } => None,
            | Event::HeapAddr { ref owner_name, .. } => Some(owner_name.as_str()),
            | Event::HeapRealloc { .. } => None,
            | Event::StackPadding { .. } => None,
            Event::WeakNew { var_name, .. }
            | Event::WeakClone { var_name, .. }
            | Event::BoxNew { var_name, .. }
            | Event::BoxFromRaw { var_name, .. }
            | Event::PinNew { var_name, .. }
            | Event::CowBorrowed { var_name, .. }
            | Event::CowOwned { var_name, .. }
            | Event::OnceCellNew { var_name, .. }
            | Event::MaybeUninitNew { var_name, .. } => Some(var_name),
            Event::ChannelSenderNew { sender_id, .. } => Some(sender_id),
            Event::ChannelReceiverNew { receiver_id, .. } => Some(receiver_id),
        }
    }

    /// Check if this is a New event
    pub fn is_new(&self) -> bool {
        matches!(self, Event::New { .. })
    }

    /// Check if this is a Borrow event
    pub fn is_borrow(&self) -> bool {
        matches!(self, Event::Borrow { .. })
    }

    /// Check if this is a Move event
    pub fn is_move(&self) -> bool {
        matches!(self, Event::Move { .. })
    }

    /// Check if this is a Drop event
    pub fn is_drop(&self) -> bool {
        matches!(self, Event::Drop { .. })
    }

    /// Check if this is an Rc event (new or clone)
    pub fn is_rc(&self) -> bool {
        matches!(self, Event::RcNew { .. } | Event::RcClone { .. })
    }

    /// Check if this is an Arc event (new or clone)
    pub fn is_arc(&self) -> bool {
        matches!(self, Event::ArcNew { .. } | Event::ArcClone { .. })
    }

    /// Check if this is a reference-counted event
    pub fn is_refcounted(&self) -> bool {
        self.is_rc() || self.is_arc()
    }

    /// Check if this is a RefCell event
    pub fn is_refcell(&self) -> bool {
        matches!(
            self,
            Event::RefCellNew { .. } | Event::RefCellBorrow { .. } | Event::RefCellDrop { .. }
        )
    }

    /// Check if this is a Cell event
    pub fn is_cell(&self) -> bool {
        matches!(
            self,
            Event::CellNew { .. } | Event::CellGet { .. } | Event::CellSet { .. }
        )
    }

    /// Check if this is an interior mutability event
    pub fn is_interior_mutability(&self) -> bool {
        self.is_refcell() || self.is_cell()
    }

    /// Check if this is a static event
    pub fn is_static(&self) -> bool {
        matches!(self, Event::StaticInit { .. } | Event::StaticAccess { .. })
    }

    /// Check if this is a const event
    pub fn is_const(&self) -> bool {
        matches!(self, Event::ConstEval { .. })
    }

    /// Check if this is a global variable event (static or const)
    pub fn is_global(&self) -> bool {
        self.is_static() || self.is_const()
    }

    /// Check if this is an unsafe event
    pub fn is_unsafe(&self) -> bool {
        matches!(
            self,
            Event::RawPtrCreated { .. }
                | Event::RawPtrDeref { .. }
                | Event::UnsafeBlockEnter { .. }
                | Event::UnsafeBlockExit { .. }
                | Event::UnsafeFnCall { .. }
                | Event::FfiCall { .. }
                | Event::Transmute { .. }
                | Event::UnionFieldAccess { .. }
        )
    }

    /// Check if this is a raw pointer event
    pub fn is_raw_ptr(&self) -> bool {
        matches!(
            self,
            Event::RawPtrCreated { .. } | Event::RawPtrDeref { .. }
        )
    }

    /// Check if this is an FFI event
    pub fn is_ffi(&self) -> bool {
        matches!(self, Event::FfiCall { .. })
    }

    /// Get strong count if this is a reference-counted event
    pub fn strong_count(&self) -> Option<usize> {
        match self {
            Event::RcNew { strong_count, .. }
            | Event::RcClone { strong_count, .. }
            | Event::ArcNew { strong_count, .. }
            | Event::ArcClone { strong_count, .. } => Some(*strong_count),
            _ => None,
        }
    }

    /// Get weak count if this is a reference-counted event
    pub fn weak_count(&self) -> Option<usize> {
        match self {
            Event::RcNew { weak_count, .. }
            | Event::RcClone { weak_count, .. }
            | Event::ArcNew { weak_count, .. }
            | Event::ArcClone { weak_count, .. }
            | Event::WeakNew { weak_count, .. }
            | Event::WeakClone { weak_count, .. } => Some(*weak_count),
            _ => None,
        }
    }

    /// Check if this is a Weak reference event
    pub fn is_weak(&self) -> bool {
        matches!(
            self,
            Event::WeakNew { .. } | Event::WeakClone { .. } | Event::WeakUpgrade { .. }
        )
    }

    /// Check if this is a Box event
    pub fn is_box(&self) -> bool {
        matches!(
            self,
            Event::BoxNew { .. } | Event::BoxIntoRaw { .. } | Event::BoxFromRaw { .. }
        )
    }

    /// Check if this is a lock guard event
    pub fn is_lock_guard(&self) -> bool {
        matches!(
            self,
            Event::LockGuardAcquire { .. } | Event::LockGuardDrop { .. }
        )
    }

    /// Check if this is a Pin event
    pub fn is_pin(&self) -> bool {
        matches!(self, Event::PinNew { .. } | Event::PinIntoInner { .. })
    }

    /// Check if this is a Cow event
    pub fn is_cow(&self) -> bool {
        matches!(
            self,
            Event::CowBorrowed { .. } | Event::CowOwned { .. } | Event::CowToMut { .. }
        )
    }

    /// Check if this is a thread event
    pub fn is_thread(&self) -> bool {
        matches!(self, Event::ThreadSpawn { .. } | Event::ThreadJoin { .. })
    }

    /// Check if this is a channel event
    pub fn is_channel(&self) -> bool {
        matches!(
            self,
            Event::ChannelSenderNew { .. }
                | Event::ChannelReceiverNew { .. }
                | Event::ChannelSend { .. }
                | Event::ChannelRecv { .. }
        )
    }

    /// Check if this is a OnceCell/OnceLock event
    pub fn is_once_cell(&self) -> bool {
        matches!(
            self,
            Event::OnceCellNew { .. }
                | Event::OnceCellSet { .. }
                | Event::OnceCellGet { .. }
                | Event::OnceCellGetOrInit { .. }
        )
    }

    /// Check if this is a MaybeUninit event
    pub fn is_maybe_uninit(&self) -> bool {
        matches!(
            self,
            Event::MaybeUninitNew { .. }
                | Event::MaybeUninitWrite { .. }
                | Event::MaybeUninitAssumeInit { .. }
                | Event::MaybeUninitAssumeInitRead { .. }
                | Event::MaybeUninitAssumeInitDrop { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_new() {
        let event = Event::New {
            timestamp: 1,
            var_name: "x".to_string(),
            var_id: "x_0".to_string(),
            type_name: "i32".to_string(),
        };

        assert_eq!(event.timestamp(), 1);
        assert_eq!(event.var_name(), Some("x"));
        assert!(event.is_new());
        assert!(!event.is_borrow());
        assert!(!event.is_move());
        assert!(!event.is_drop());
    }

    #[test]
    fn test_event_borrow() {
        let event = Event::Borrow {
            timestamp: 2,
            borrower_name: "r".to_string(),
            borrower_id: "r_1".to_string(),
            owner_id: "x_0".to_string(),
            mutable: false,
        };

        assert_eq!(event.timestamp(), 2);
        assert_eq!(event.var_name(), Some("r"));
        assert!(event.is_borrow());
        assert!(!event.is_new());
    }

    #[test]
    fn test_event_move() {
        let event = Event::Move {
            timestamp: 3,
            from_id: "x_0".to_string(),
            to_name: "y".to_string(),
            to_id: "y_1".to_string(),
        };

        assert_eq!(event.timestamp(), 3);
        assert_eq!(event.var_name(), Some("y"));
        assert!(event.is_move());
    }

    #[test]
    fn test_event_drop() {
        let event = Event::Drop {
            timestamp: 4,
            var_id: "x_0".to_string(),
            location: None,
        };

        assert_eq!(event.timestamp(), 4);
        assert!(event.is_drop());
    }

    #[test]
    fn test_event_serialization() {
        let event = Event::New {
            timestamp: 1,
            var_name: "x".to_string(),
            var_id: "x_0".to_string(),
            type_name: "i32".to_string(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: Event = serde_json::from_str(&json).unwrap();

        assert_eq!(event, deserialized);
    }

    #[test]
    fn test_borrow_mutable_flag() {
        let immut = Event::Borrow {
            timestamp: 1,
            borrower_name: "r".to_string(),
            borrower_id: "r_0".to_string(),
            owner_id: "x_0".to_string(),
            mutable: false,
        };

        let mut_borrow = Event::Borrow {
            timestamp: 2,
            borrower_name: "r".to_string(),
            borrower_id: "r_1".to_string(),
            owner_id: "x_0".to_string(),
            mutable: true,
        };

        assert_ne!(immut, mut_borrow);
    }
}
