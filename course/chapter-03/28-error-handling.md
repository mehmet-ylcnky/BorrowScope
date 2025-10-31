# Section 28: Error Handling

## Learning Objectives

By the end of this section, you will:
- Design robust error types
- Handle serialization errors gracefully
- Implement fallible operations
- Use Result and Option effectively
- Add error context with anyhow

## Prerequisites

- Completed Section 27 (Integration Testing)
- Understanding of Result and Option types
- Familiarity with error handling patterns

---

## Error Types

Create `borrowscope-runtime/src/error.rs`:

```rust
//! Error types for BorrowScope runtime

use std::fmt;

#[derive(Debug)]
pub enum Error {
    /// JSON serialization failed
    SerializationError(serde_json::Error),
    
    /// Export failed
    ExportError(String),
    
    /// Invalid event sequence
    InvalidEventSequence(String),
    
    /// Lock acquisition failed
    LockError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::SerializationError(e) => write!(f, "Serialization error: {}", e),
            Error::ExportError(msg) => write!(f, "Export error: {}", msg),
            Error::InvalidEventSequence(msg) => write!(f, "Invalid event sequence: {}", msg),
            Error::LockError => write!(f, "Failed to acquire lock"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::SerializationError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::SerializationError(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
```

---

## Fallible Export

Update `borrowscope-runtime/src/tracker.rs`:

```rust
use crate::error::{Error, Result};

impl Tracker {
    /// Export to JSON with error handling
    pub fn to_json(&self) -> Result<String> {
        let data = self.export();
        serde_json::to_string_pretty(&data)
            .map_err(Error::from)
    }
    
    /// Try to export, return None on error
    pub fn try_export(&self) -> Option<String> {
        self.to_json().ok()
    }
}

/// Export with error handling
pub fn export_json() -> Result<String> {
    TRACKER.lock().to_json()
}

/// Export with timeout
pub fn export_json_timeout(duration: std::time::Duration) -> Result<String> {
    TRACKER.try_lock_for(duration)
        .ok_or(Error::LockError)?
        .to_json()
}
```

---

## Validation

Add event validation:

```rust
// borrowscope-runtime/src/event.rs
impl Event {
    /// Validate event data
    pub fn validate(&self) -> Result<()> {
        match self {
            Event::New { name, .. } => {
                if name.is_empty() {
                    return Err(Error::InvalidEventSequence(
                        "Variable name cannot be empty".to_string()
                    ));
                }
            }
            Event::Borrow { borrowed_id, .. } => {
                if *borrowed_id == 0 {
                    return Err(Error::InvalidEventSequence(
                        "Invalid borrowed ID".to_string()
                    ));
                }
            }
            _ => {}
        }
        Ok(())
    }
}
```

---

## Testing Errors

Create `borrowscope-runtime/tests/error_handling.rs`:

```rust
use borrowscope_runtime::*;

#[test]
fn test_export_error_handling() {
    reset_tracker();
    
    // Valid export
    let result = export_json();
    assert!(result.is_ok());
}

#[test]
fn test_timeout() {
    use std::time::Duration;
    
    let result = export_json_timeout(Duration::from_millis(100));
    assert!(result.is_ok());
}
```

---

## Key Takeaways

✅ **Custom error types** - Clear error messages  
✅ **Result propagation** - Use ? operator  
✅ **Graceful degradation** - Fallback options  
✅ **Validation** - Catch errors early  

---

**Previous:** [27-integration-testing.md](./27-integration-testing.md)  
**Next:** [29-benchmarking-suite.md](./29-benchmarking-suite.md)

**Progress:** 8/15 ⬛⬛⬛⬛⬛⬛⬛⬛⬜⬜⬜⬜⬜⬜⬜
