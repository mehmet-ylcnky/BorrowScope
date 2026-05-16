//! Memory layout tracking functions.
//! Records actual stack/heap addresses at runtime for visualization.

use super::{TRACKER, TIMESTAMP};
use crate::event::Event;
use std::sync::atomic::Ordering;

fn next_ts() -> u64 {
    TIMESTAMP.fetch_add(1, Ordering::Relaxed)
}

fn next_var_id(name: &str) -> String {
    format!("{}_{}", name, TIMESTAMP.load(Ordering::Relaxed))
}

/// Track a stack variable's address and size.
#[cfg_attr(not(feature = "track"), allow(unused_variables))]
pub fn track_stack_addr<T>(name: &str, value: &T) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.events.push(Event::StackAddr {
            timestamp: next_ts(),
            var_name: name.to_string(),
            var_id: next_var_id(name),
            addr: value as *const T as usize,
            size: std::mem::size_of::<T>(),
            type_name: std::any::type_name::<T>().to_string(),
            location: String::new(),
        });
    }
}

/// Track a stack variable with explicit ID.
#[cfg_attr(not(feature = "track"), allow(unused_variables))]
pub fn track_stack_addr_with_id<T>(name: &str, var_id: &str, value: &T) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.events.push(Event::StackAddr {
            timestamp: next_ts(),
            var_name: name.to_string(),
            var_id: var_id.to_string(),
            addr: value as *const T as usize,
            size: std::mem::size_of::<T>(),
            type_name: std::any::type_name::<T>().to_string(),
            location: String::new(),
        });
    }
}

/// Track a field within a stack variable.
#[cfg_attr(not(feature = "track"), allow(unused_variables))]
pub fn track_stack_field(var_id: &str, field_name: &str, field_value: &str, offset: usize) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.events.push(Event::StackField {
            timestamp: next_ts(),
            var_id: var_id.to_string(),
            field_name: field_name.to_string(),
            field_value: field_value.to_string(),
            offset,
        });
    }
}

/// Track a heap allocation.
#[cfg_attr(not(feature = "track"), allow(unused_variables))]
pub fn track_heap_addr(owner_name: &str, addr: usize, size: usize, capacity: usize, content: &str) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.events.push(Event::HeapAddr {
            timestamp: next_ts(),
            var_id: next_var_id(owner_name),
            owner_name: owner_name.to_string(),
            addr,
            size,
            capacity,
            content_preview: content.to_string(),
        });
    }
}

/// Track a heap reallocation.
#[cfg_attr(not(feature = "track"), allow(unused_variables))]
pub fn track_heap_realloc(var_id: &str, old_addr: usize, new_addr: usize, old_size: usize, new_size: usize) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.events.push(Event::HeapRealloc {
            timestamp: next_ts(),
            var_id: var_id.to_string(),
            old_addr, new_addr, old_size, new_size,
        });
    }
}

/// Track padding between stack variables.
#[cfg_attr(not(feature = "track"), allow(unused_variables))]
pub fn track_stack_padding(after_var: &str, addr: usize, bytes: usize) {
    #[cfg(feature = "track")]
    {
        let mut tracker = TRACKER.lock();
        tracker.events.push(Event::StackPadding {
            timestamp: next_ts(),
            after_var: after_var.to_string(),
            addr, bytes,
        });
    }
}

/// Convenience: track a String's full layout (stack + heap).
#[cfg_attr(not(feature = "track"), allow(unused_variables))]
pub fn track_string_layout(name: &str, value: &String) {
    #[cfg(feature = "track")]
    {
        let var_id = next_var_id(name);
        let addr = value as *const String as usize;
        let ptr_val = value.as_ptr() as usize;
        let mut tracker = TRACKER.lock();

        tracker.events.push(Event::StackAddr {
            timestamp: next_ts(), var_name: name.to_string(), var_id: var_id.clone(),
            addr, size: std::mem::size_of::<String>(), type_name: "String".to_string(), location: String::new(),
        });
        tracker.events.push(Event::StackField { timestamp: next_ts(), var_id: var_id.clone(), field_name: "ptr".to_string(), field_value: format!("0x{:x}", ptr_val), offset: 0 });
        tracker.events.push(Event::StackField { timestamp: next_ts(), var_id: var_id.clone(), field_name: "len".to_string(), field_value: value.len().to_string(), offset: 8 });
        tracker.events.push(Event::StackField { timestamp: next_ts(), var_id: var_id.clone(), field_name: "cap".to_string(), field_value: value.capacity().to_string(), offset: 16 });
        tracker.events.push(Event::HeapAddr {
            timestamp: next_ts(), var_id: format!("{}_data", var_id), owner_name: name.to_string(),
            addr: ptr_val, size: value.len(), capacity: value.capacity(),
            content_preview: if value.len() <= 20 { format!("\"{}\"", value) } else { format!("\"{}...\"", &value[..17]) },
        });
    }
}

/// Convenience: track a Vec's full layout (stack + heap).
#[cfg_attr(not(feature = "track"), allow(unused_variables))]
pub fn track_vec_layout<T: std::fmt::Debug>(name: &str, value: &Vec<T>) {
    #[cfg(feature = "track")]
    {
        let var_id = next_var_id(name);
        let addr = value as *const Vec<T> as usize;
        let ptr_val = value.as_ptr() as usize;
        let type_name = format!("Vec<{}>", std::any::type_name::<T>());
        let mut tracker = TRACKER.lock();

        tracker.events.push(Event::StackAddr {
            timestamp: next_ts(), var_name: name.to_string(), var_id: var_id.clone(),
            addr, size: std::mem::size_of::<Vec<T>>(), type_name, location: String::new(),
        });
        tracker.events.push(Event::StackField { timestamp: next_ts(), var_id: var_id.clone(), field_name: "ptr".to_string(), field_value: format!("0x{:x}", ptr_val), offset: 0 });
        tracker.events.push(Event::StackField { timestamp: next_ts(), var_id: var_id.clone(), field_name: "len".to_string(), field_value: value.len().to_string(), offset: 8 });
        tracker.events.push(Event::StackField { timestamp: next_ts(), var_id: var_id.clone(), field_name: "cap".to_string(), field_value: value.capacity().to_string(), offset: 16 });

        let content = if value.len() <= 5 { format!("{:?}", value) } else { format!("[{:?}, ... ({} items)]", value[0], value.len()) };
        tracker.events.push(Event::HeapAddr {
            timestamp: next_ts(), var_id: format!("{}_data", var_id), owner_name: name.to_string(),
            addr: ptr_val, size: value.len() * std::mem::size_of::<T>(), capacity: value.capacity() * std::mem::size_of::<T>(),
            content_preview: content,
        });
    }
}
