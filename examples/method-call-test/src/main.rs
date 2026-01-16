//! Test file for method call tracking (Phase 1 + Phase 1.5)

use std::cell::Cell;
use std::borrow::Cow;
use std::cell::OnceCell;
use std::sync::mpsc;
use std::thread;
use std::rc::Rc;
use std::sync::Arc;
use std::cell::RefCell;
use std::sync::{Mutex, RwLock};

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

// Phase 1.5: Smart pointer methods
fn test_rc_methods() {
    let rc = Rc::new(42);
    let _ = rc.clone();  // Should track: rc_clone
    let weak = Rc::downgrade(&rc);  // downgrade is associated fn, not method
    let _ = weak.upgrade();  // Should track: weak_upgrade
}

fn test_arc_methods() {
    let arc = Arc::new(42);
    let _ = arc.clone();  // Should track: arc_clone
    let weak = Arc::downgrade(&arc);  // downgrade is associated fn
    let _ = weak.upgrade();  // Should track: weak_upgrade
}

fn test_refcell_methods() {
    let refcell = RefCell::new(42);
    let _ = refcell.borrow();  // Should track: refcell_borrow
    let _ = refcell.borrow_mut();  // Should track: refcell_borrow_mut
}

fn test_mutex_methods() {
    let mutex = Mutex::new(42);
    let _ = mutex.lock();  // Should track: mutex_lock
    let _ = mutex.try_lock();  // Should track: mutex_try_lock
}

fn test_rwlock_methods() {
    let rwlock = RwLock::new(42);
    let _ = rwlock.read();  // Should track: rwlock_read
    let _ = rwlock.write();  // Should track: rwlock_write
}

// Phase 2: Standalone expressions
fn test_drop_forget() {
    let x = Box::new(42);
    let y = Box::new(43);
    drop(x);  // Should track: drop
    std::mem::forget(y);  // Should track: forget
}

fn test_thread_spawn() {
    let _ = std::thread::spawn(|| 42);  // Should track: thread_spawn
}

fn main() {
    test_cell_methods();
    test_cow_methods();
    test_once_cell_methods();
    test_channel_methods();
    test_thread_join();
    test_rc_methods();
    test_arc_methods();
    test_refcell_methods();
    test_mutex_methods();
    test_rwlock_methods();
    test_drop_forget();
    test_thread_spawn();
}
