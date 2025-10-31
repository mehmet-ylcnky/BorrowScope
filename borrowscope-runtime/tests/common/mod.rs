#![allow(dead_code)]

use borrowscope_runtime::*;

/// Test fixture that ensures clean state for each test
pub struct TestFixture;

impl TestFixture {
    pub fn new() -> Self {
        reset();
        Self
    }

    pub fn events(&self) -> Vec<Event> {
        get_events()
    }

    pub fn event_count(&self) -> usize {
        get_events().len()
    }

    pub fn assert_event_types(&self, expected: &[&str]) {
        let events = self.events();
        let actual: Vec<&str> = events
            .iter()
            .map(|e| {
                if e.is_new() {
                    "New"
                } else if e.is_borrow() {
                    "Borrow"
                } else if e.is_move() {
                    "Move"
                } else if e.is_drop() {
                    "Drop"
                } else {
                    "Unknown"
                }
            })
            .collect();

        assert_eq!(
            actual, expected,
            "Event types mismatch.\nExpected: {:?}\nActual: {:?}",
            expected, actual
        );
    }

    pub fn assert_has_event_type(&self, event_type: &str) {
        let events = self.events();
        let has_type = events.iter().any(|e| match event_type {
            "New" => e.is_new(),
            "Borrow" => e.is_borrow(),
            "Move" => e.is_move(),
            "Drop" => e.is_drop(),
            _ => false,
        });

        assert!(
            has_type,
            "Expected to find {} event, but none found",
            event_type
        );
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        reset();
    }
}
