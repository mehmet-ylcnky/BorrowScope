//! Concurrency tracking: threads, channels, lock guards

use super::TRACKER;

pub fn track_thread_spawn<T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] thread_id: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    handle: std::thread::JoinHandle<T>,
) -> std::thread::JoinHandle<T> {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_thread_spawn(thread_id, location);
    }
    handle
}

/// Track thread join
#[inline(always)]
pub fn track_thread_join<T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] thread_id: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    result: std::thread::Result<T>,
) -> std::thread::Result<T> {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_thread_join(thread_id, location);
    }
    result
}

/// Track channel creation (returns both sender and receiver)
#[inline(always)]
pub fn track_channel<T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] channel_id: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    sender: std::sync::mpsc::Sender<T>,
    receiver: std::sync::mpsc::Receiver<T>,
) -> (std::sync::mpsc::Sender<T>, std::sync::mpsc::Receiver<T>) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_channel_sender_new(&format!("{}_tx", channel_id), channel_id, location);
        tracker.record_channel_receiver_new(&format!("{}_rx", channel_id), channel_id, location);
    }
    (sender, receiver)
}

/// Track channel send
#[inline(always)]
pub fn track_channel_send<T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] sender_id: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    result: Result<(), std::sync::mpsc::SendError<T>>,
) -> Result<(), std::sync::mpsc::SendError<T>> {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.record_channel_send(sender_id, location);
    }
    result
}

/// Track channel receive
#[inline(always)]
pub fn track_channel_recv<T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] receiver_id: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    result: Result<T, std::sync::mpsc::RecvError>,
) -> Result<T, std::sync::mpsc::RecvError> {
    #[cfg(feature = "track")]
    {
        let success = result.is_ok();
        let mut tracker = TRACKER.lock();
        tracker.record_channel_recv(receiver_id, success, location);
    }
    result
}

/// Track channel try_recv
#[inline(always)]
pub fn track_channel_try_recv<T>(
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] receiver_id: &str,
    #[cfg_attr(not(feature = "track"), allow(unused_variables))] location: &str,
    result: Result<T, std::sync::mpsc::TryRecvError>,
) -> Result<T, std::sync::mpsc::TryRecvError> {
    #[cfg(feature = "track")]
    {
        let success = result.is_ok();
        let mut tracker = TRACKER.lock();
        tracker.record_channel_recv(receiver_id, success, location);
    }
    result
}

