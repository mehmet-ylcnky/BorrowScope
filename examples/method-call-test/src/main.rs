//! Test file for method call tracking (Phase 1)

use std::cell::Cell;
use std::borrow::Cow;
use std::cell::OnceCell;
use std::sync::mpsc;
use std::thread;

fn test_cell_methods() {
    let cell = Cell::new(42);
    cell.set(100);  // Should track: cell_set, self_borrow: immutable
    let _ = cell.get();  // Should track: cell_get, self_borrow: immutable
}

fn test_cow_methods() {
    let mut cow: Cow<str> = Cow::Borrowed("hello");
    let _ = cow.to_mut();  // Should track: cow_to_mut, self_borrow: mutable
}

fn test_once_cell_methods() {
    let cell: OnceCell<i32> = OnceCell::new();
    let _ = cell.set(42);  // Should track: once_cell_set, self_borrow: immutable
    let _ = cell.get();  // Should track: once_cell_get, self_borrow: immutable
    let _ = cell.get_or_init(|| 42);  // Should track: once_cell_get_or_init
}

fn test_channel_methods() {
    let (tx, rx) = mpsc::channel();
    let _ = tx.send(42);  // Should track: channel_send
    let _ = rx.recv();  // Should track: channel_recv
    let _ = rx.try_recv();  // Should track: channel_try_recv
}

fn test_thread_join() {
    let handle = thread::spawn(|| 42);
    let _ = handle.join();  // Should track: thread_join, self_borrow: consuming
}

fn main() {
    test_cell_methods();
    test_cow_methods();
    test_once_cell_methods();
    test_channel_methods();
    test_thread_join();
}
