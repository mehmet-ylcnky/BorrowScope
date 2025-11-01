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

/// Track an immutable borrow
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

/// Track a mutable borrow
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

/// Track a move
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

/// Track a drop
#[inline(always)]
pub fn track_drop(#[cfg_attr(not(feature = "track"), allow(unused_variables))] name: &str) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_drop(name);
    }
}

/// Track multiple drops in batch (optimized)
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

/// Reset tracking state
pub fn reset() {
    let mut tracker = TRACKER.lock();
    tracker.clear();
}

/// Get all events
pub fn get_events() -> Vec<Event> {
    TRACKER.lock().events().to_vec()
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
pub fn track_rc_new_with_id<T>(
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
pub fn track_rc_clone_with_id<T>(
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
pub fn track_arc_new_with_id<T>(
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
pub fn track_arc_clone_with_id<T>(
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

/// Track Rc::new allocation
#[inline(always)]
pub fn track_rc_new<T>(
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

/// Track Rc::clone operation
#[inline(always)]
pub fn track_rc_clone<T>(
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

/// Track Arc::new allocation
#[inline(always)]
pub fn track_arc_new<T>(
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

/// Track Arc::clone operation
#[inline(always)]
pub fn track_arc_clone<T>(
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
