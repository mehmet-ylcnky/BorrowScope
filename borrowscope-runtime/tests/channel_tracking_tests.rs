//! Comprehensive tests for channel (mpsc) tracking
//!
//! Tests cover:
//! - Channel creation
//! - Sender::send tracking
//! - Receiver::recv tracking
//! - Receiver::try_recv tracking
//! - Multi-producer patterns
//! - Channel lifecycle

use borrowscope_runtime::*;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

mod test_utils {
    use parking_lot::Mutex;
    pub static TEST_LOCK: Mutex<()> = Mutex::new(());
}

// =============================================================================
// Channel Creation Tests
// =============================================================================

#[test]
fn test_channel_creation() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let (tx, rx) = mpsc::channel::<i32>();
    let (_tx, _rx) = track_channel("chan", "test:1", tx, rx);

    let events = get_events();
    assert_eq!(events.len(), 2); // sender + receiver
    assert!(events.iter().all(|e| e.is_channel()));
}

#[test]
fn test_channel_creation_with_type() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let (tx, rx) = mpsc::channel::<String>();
    let (_tx, _rx) = track_channel("string_chan", "test:1", tx, rx);

    let events = get_events();
    assert_eq!(events.len(), 2);
}

// =============================================================================
// Send Tests
// =============================================================================

#[test]
fn test_channel_send_success() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let (tx, rx) = mpsc::channel();
    let result = track_channel_send("tx", "test:1", tx.send(42));

    assert!(result.is_ok());
    assert_eq!(rx.recv().unwrap(), 42);

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_channel());
}

#[test]
fn test_channel_send_multiple() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let (tx, rx) = mpsc::channel();

    let _ = track_channel_send("tx", "test:1", tx.send(1));
    let _ = track_channel_send("tx", "test:2", tx.send(2));
    let _ = track_channel_send("tx", "test:3", tx.send(3));

    assert_eq!(rx.recv().unwrap(), 1);
    assert_eq!(rx.recv().unwrap(), 2);
    assert_eq!(rx.recv().unwrap(), 3);

    let events = get_events();
    assert_eq!(events.len(), 3);
}

#[test]
fn test_channel_send_after_receiver_drop() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let (tx, rx) = mpsc::channel::<i32>();
    drop(rx);

    let result = track_channel_send("tx", "test:1", tx.send(42));
    assert!(result.is_err());

    let events = get_events();
    assert_eq!(events.len(), 1);
}

// =============================================================================
// Recv Tests
// =============================================================================

#[test]
fn test_channel_recv_success() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let (tx, rx) = mpsc::channel();
    tx.send(100).unwrap();

    let result = track_channel_recv("rx", "test:1", rx.recv());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 100);

    let events = get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].is_channel());
}

#[test]
fn test_channel_recv_after_sender_drop() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let (tx, rx) = mpsc::channel::<i32>();
    drop(tx);

    let result = track_channel_recv("rx", "test:1", rx.recv());
    assert!(result.is_err());

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
fn test_channel_recv_multiple() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let (tx, rx) = mpsc::channel();
    tx.send("a").unwrap();
    tx.send("b").unwrap();
    tx.send("c").unwrap();

    let r1 = track_channel_recv("rx", "test:1", rx.recv());
    let r2 = track_channel_recv("rx", "test:2", rx.recv());
    let r3 = track_channel_recv("rx", "test:3", rx.recv());

    assert_eq!(r1.unwrap(), "a");
    assert_eq!(r2.unwrap(), "b");
    assert_eq!(r3.unwrap(), "c");

    let events = get_events();
    assert_eq!(events.len(), 3);
}

// =============================================================================
// Try_recv Tests
// =============================================================================

#[test]
fn test_channel_try_recv_success() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let (tx, rx) = mpsc::channel();
    tx.send(42).unwrap();

    let result = track_channel_try_recv("rx", "test:1", rx.try_recv());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
fn test_channel_try_recv_empty() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let (_tx, rx) = mpsc::channel::<i32>();

    let result = track_channel_try_recv("rx", "test:1", rx.try_recv());
    assert!(result.is_err());

    let events = get_events();
    assert_eq!(events.len(), 1);
}

#[test]
fn test_channel_try_recv_disconnected() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let (tx, rx) = mpsc::channel::<i32>();
    drop(tx);

    let result = track_channel_try_recv("rx", "test:1", rx.try_recv());
    assert!(result.is_err());

    let events = get_events();
    assert_eq!(events.len(), 1);
}

// =============================================================================
// Full Lifecycle Tests
// =============================================================================

#[test]
fn test_channel_full_lifecycle() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let (tx, rx) = mpsc::channel();
    let (tx, rx) = track_channel("data_chan", "test:1", tx, rx);

    let _ = track_channel_send("data_chan_tx", "test:2", tx.send(1));
    let _ = track_channel_send("data_chan_tx", "test:3", tx.send(2));

    let r1 = track_channel_recv("data_chan_rx", "test:4", rx.recv());
    let r2 = track_channel_recv("data_chan_rx", "test:5", rx.recv());

    assert_eq!(r1.unwrap(), 1);
    assert_eq!(r2.unwrap(), 2);

    let events = get_events();
    assert_eq!(events.len(), 6); // 2 creation + 2 send + 2 recv
}

// =============================================================================
// Cross-Thread Tests
// =============================================================================

#[test]
fn test_channel_cross_thread() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let (tx, rx) = mpsc::channel();

    let sender = thread::spawn(move || {
        for i in 0..5 {
            tx.send(i).unwrap();
        }
    });

    let receiver = thread::spawn(move || {
        let mut sum = 0;
        for _ in 0..5 {
            sum += rx.recv().unwrap();
        }
        sum
    });

    sender.join().unwrap();
    let result = receiver.join().unwrap();
    assert_eq!(result, 10); // 0+1+2+3+4
}

#[test]
fn test_channel_producer_consumer() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let (tx, rx) = mpsc::channel();
    let (tx, rx) = track_channel("work", "test:1", tx, rx);

    let producer = thread::spawn(move || {
        for i in 1..=3 {
            let _ = track_channel_send("work_tx", &format!("send:{}", i), tx.send(i * 10));
            thread::sleep(Duration::from_millis(10));
        }
    });

    let consumer = thread::spawn(move || {
        let mut results = Vec::new();
        for i in 1..=3 {
            if let Ok(val) = track_channel_recv("work_rx", &format!("recv:{}", i), rx.recv()) {
                results.push(val);
            }
        }
        results
    });

    producer.join().unwrap();
    let results = consumer.join().unwrap();
    assert_eq!(results, vec![10, 20, 30]);
}

// =============================================================================
// Multi-Producer Tests
// =============================================================================

#[test]
fn test_channel_multi_producer() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let (tx, rx) = mpsc::channel();
    let tx2 = tx.clone();
    let tx3 = tx.clone();

    let _ = track_channel_send("tx1", "test:1", tx.send(1));
    let _ = track_channel_send("tx2", "test:2", tx2.send(2));
    let _ = track_channel_send("tx3", "test:3", tx3.send(3));

    let mut values = vec![rx.recv().unwrap(), rx.recv().unwrap(), rx.recv().unwrap()];
    values.sort();

    assert_eq!(values, vec![1, 2, 3]);

    let events = get_events();
    assert_eq!(events.len(), 3);
}

#[test]
fn test_channel_multi_producer_threads() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let (tx, rx) = mpsc::channel();

    let handles: Vec<_> = (0..3)
        .map(|i| {
            let tx = tx.clone();
            thread::spawn(move || {
                tx.send(i).unwrap();
            })
        })
        .collect();

    drop(tx); // Drop original sender

    for h in handles {
        h.join().unwrap();
    }

    let mut values: Vec<_> = (0..3).map(|_| rx.recv().unwrap()).collect();
    values.sort();
    assert_eq!(values, vec![0, 1, 2]);
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_channel_large_messages() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    let (tx, rx) = mpsc::channel();
    let large_data = vec![0u8; 1024 * 1024]; // 1MB

    let _ = track_channel_send("tx", "test:1", tx.send(large_data));
    let result = track_channel_recv("rx", "test:2", rx.recv());

    assert_eq!(result.unwrap().len(), 1024 * 1024);

    let events = get_events();
    assert_eq!(events.len(), 2);
}

#[test]
fn test_channel_complex_type() {
    let _lock = test_utils::TEST_LOCK.lock();
    reset();

    #[derive(Debug, PartialEq)]
    struct Message {
        id: u64,
        payload: String,
    }

    let (tx, rx) = mpsc::channel();
    let msg = Message {
        id: 42,
        payload: String::from("hello"),
    };

    let _ = track_channel_send("tx", "test:1", tx.send(msg));
    let result = track_channel_recv("rx", "test:2", rx.recv()).unwrap();

    assert_eq!(result.id, 42);
    assert_eq!(result.payload, "hello");

    let events = get_events();
    assert_eq!(events.len(), 2);
}
