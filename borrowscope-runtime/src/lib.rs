//! BorrowScope Runtime
//!
//! This crate provides the runtime tracking system that records ownership
//! and borrowing events during program execution.
//!
//! # Design Principles
//!
//! - **Zero-cost abstractions**: Tracking functions are inlined and return values unchanged
//! - **Type safety**: Generic functions work with any type without boxing
//! - **Thread safety**: All operations are thread-safe using efficient synchronization
//! - **Simplicity**: Clean, minimal API that's easy to use
//! - **Reliability**: Tracking never panics or breaks user code
//!
//! # Architecture
//!
//! The runtime uses an event sourcing pattern:
//! 1. Track operations as events (New, Borrow, Move, Drop)
//! 2. Store events in a thread-safe global tracker
//! 3. Build ownership graphs from event streams on demand
//! 4. Export data to JSON for visualization
//!
//! # Example
//!
//! ```rust
//! use borrowscope_runtime::*;
//!
//! // Track variable creation
//! let x = track_new("x", 5);
//!
//! // Track borrowing
//! let r = track_borrow("r", &x);
//!
//! // Track drop (called automatically by macro)
//! track_drop("r");
//! track_drop("x");
//! ```

mod event;
mod graph;
mod tracker;

pub use event::Event;
pub use graph::{build_graph, GraphStats, OwnershipGraph, Relationship, Variable};
pub use tracker::{
    get_events, reset, track_borrow, track_borrow_mut, track_drop, track_move, track_new,
};

/// Get the ownership graph built from current events
pub fn get_graph() -> OwnershipGraph {
    let events = get_events();
    build_graph(&events)
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_simple_tracking() {
        reset();

        let x = track_new("x", 5);
        assert_eq!(x, 5);

        let events = get_events();
        assert_eq!(events.len(), 1);
        assert!(events[0].is_new());
    }

    #[test]
    fn test_borrow_tracking() {
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
}
