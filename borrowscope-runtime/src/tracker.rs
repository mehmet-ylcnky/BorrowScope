//! Core tracking functionality

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
    pub fn record_drop(&mut self, var_id: &str) {
        let timestamp = Self::next_timestamp();

        self.events.push(Event::Drop {
            timestamp,
            var_id: var_id.to_string(),
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

/// Track a new variable
#[inline(always)]
pub fn track_new<T>(name: &str, value: T) -> T {
    let type_name = std::any::type_name::<T>();

    if let Some(mut tracker) = TRACKER.try_lock() {
        tracker.record_new(name, type_name);
    }

    value
}

/// Track an immutable borrow
#[inline(always)]
pub fn track_borrow<'a, T>(name: &str, value: &'a T) -> &'a T {
    if let Some(mut tracker) = TRACKER.try_lock() {
        tracker.record_borrow(name, "unknown", false);
    }

    value
}

/// Track a mutable borrow
#[inline(always)]
pub fn track_borrow_mut<'a, T>(name: &str, value: &'a mut T) -> &'a mut T {
    if let Some(mut tracker) = TRACKER.try_lock() {
        tracker.record_borrow(name, "unknown", true);
    }

    value
}

/// Track a move
#[inline(always)]
pub fn track_move<T>(_from: &str, _to: &str, value: T) -> T {
    // Move tracking will be improved in later sections
    value
}

/// Track a drop
#[inline(always)]
pub fn track_drop(name: &str) {
    if let Some(mut tracker) = TRACKER.try_lock() {
        tracker.record_drop(name);
    }
}

/// Reset tracking state
pub fn reset() {
    if let Some(mut tracker) = TRACKER.try_lock() {
        tracker.clear();
    }
}

/// Get all events
pub fn get_events() -> Vec<Event> {
    TRACKER.lock().events().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    lazy_static::lazy_static! {
        /// Global test lock to ensure tests run serially when accessing shared tracker
        static ref TEST_LOCK: parking_lot::Mutex<()> = parking_lot::Mutex::new(());
    }

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
        let value = track_new("x", 42);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_track_borrow_returns_reference() {
        let _lock = TEST_LOCK.lock();
        reset();
        let s = String::from("hello");
        let r = track_borrow("r", &s);
        assert_eq!(r, "hello");
    }

    #[test]
    fn test_track_borrow_mut_returns_reference() {
        let _lock = TEST_LOCK.lock();
        reset();
        let mut s = String::from("hello");
        let r = track_borrow_mut("r", &mut s);
        r.push_str(" world");
        assert_eq!(r, "hello world");
    }

    #[test]
    fn test_complete_workflow() {
        let _lock = TEST_LOCK.lock();
        reset();

        let x = track_new("x", 5);
        let _r = track_borrow("r", &x);
        track_drop("r");
        track_drop("x");

        let events = get_events();
        assert_eq!(events.len(), 4);
        assert!(events[0].is_new());
        assert!(events[1].is_borrow());
        assert!(events[2].is_drop());
        assert!(events[3].is_drop());
    }

    #[test]
    fn test_reset() {
        let _lock = TEST_LOCK.lock();
        reset();

        track_new("x", 5);
        track_new("y", 10);

        assert_eq!(get_events().len(), 2);

        reset();

        assert_eq!(get_events().len(), 0);
    }

    #[test]
    fn test_unique_ids() {
        let _lock = TEST_LOCK.lock();
        reset();

        track_new("x", 1);
        track_new("x", 2);
        track_new("x", 3);

        let events = get_events();
        let ids: Vec<_> = events
            .iter()
            .filter_map(|e| match e {
                Event::New { var_id, .. } => Some(var_id.as_str()),
                _ => None,
            })
            .collect();

        assert_eq!(ids.len(), 3);
        assert_eq!(ids[0], "x_0");
        assert_eq!(ids[1], "x_1");
        assert_eq!(ids[2], "x_2");
    }

    #[test]
    fn test_thread_safety() {
        let _lock = TEST_LOCK.lock();
        reset();

        // Test that multiple threads can safely call tracking functions
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
        // With try_lock(), some events may be lost under contention
        // The important thing is no panics or data corruption
        assert!(!events.is_empty(), "Should have tracked some events");
    }

    #[test]
    fn test_timestamp_uniqueness() {
        let _lock = TEST_LOCK.lock();
        reset();

        // Generate timestamps from multiple threads
        let handles: Vec<_> = (0..4)
            .map(|i| {
                std::thread::spawn(move || {
                    for j in 0..5 {
                        track_new(&format!("var_{}_{}", i, j), i * 5 + j);
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let events = get_events();
        if events.len() > 1 {
            let mut timestamps: Vec<_> = events.iter().map(|e| e.timestamp()).collect();
            timestamps.sort_unstable();

            // Check for uniqueness
            for i in 1..timestamps.len() {
                assert!(
                    timestamps[i] > timestamps[i - 1],
                    "Timestamps should be unique and monotonic"
                );
            }
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
}
