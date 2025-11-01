//! Shared test utilities
//!
//! This module provides shared resources for tests to prevent race conditions.

#[cfg(test)]
lazy_static::lazy_static! {
    /// Global test lock to ensure tests run serially when accessing shared tracker
    /// This is shared across all test modules to prevent race conditions
    pub static ref TEST_LOCK: parking_lot::Mutex<()> = parking_lot::Mutex::new(());
}
