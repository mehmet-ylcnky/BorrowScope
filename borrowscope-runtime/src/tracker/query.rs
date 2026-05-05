//! Query functions for retrieving and summarizing events

use super::TRACKER;
use crate::event::Event;

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
    TRACKER
        .lock()
        .events()
        .iter()
        .filter(|e| predicate(e))
        .cloned()
        .collect()
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
