#![feature(mapped_lock_guards)]

use std::sync::{Mutex, RwLock, MutexGuard, RwLockReadGuard, RwLockWriteGuard};

fn main() {
    let mutex = Mutex::new(vec![1, 2, 3]);
    let guard = mutex.lock().unwrap();
    let mapped = MutexGuard::map(guard, |v| &mut v[0]);

    let rwlock = RwLock::new(vec![4, 5, 6]);
    let read_guard = rwlock.read().unwrap();
    let mapped_read = RwLockReadGuard::map(read_guard, |v| &v[1]);

    drop(mapped);
    drop(mapped_read);

    let write_guard = rwlock.write().unwrap();
    let mapped_write = RwLockWriteGuard::map(write_guard, |v| &mut v[2]);
    drop(mapped_write);
}
