//! Error types for BorrowScope runtime

use std::fmt;
use std::io;

/// Runtime errors
#[derive(Debug)]
pub enum Error {
    /// JSON serialization failed
    SerializationError(serde_json::Error),

    /// File I/O error
    IoError(io::Error),

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
            Error::IoError(e) => write!(f, "I/O error: {}", e),
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
            Error::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::SerializationError(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IoError(err)
    }
}

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::LockError;
        assert_eq!(err.to_string(), "Failed to acquire lock");

        let err = Error::ExportError("test".to_string());
        assert_eq!(err.to_string(), "Export error: test");

        let err = Error::InvalidEventSequence("invalid".to_string());
        assert_eq!(err.to_string(), "Invalid event sequence: invalid");
    }

    #[test]
    fn test_error_from_serde() {
        let json_err = serde_json::from_str::<i32>("invalid").unwrap_err();
        let err: Error = json_err.into();
        assert!(matches!(err, Error::SerializationError(_)));
    }

    #[test]
    fn test_error_source_trait() {
        use std::error::Error as StdError;

        let json_err = serde_json::from_str::<i32>("invalid").unwrap_err();
        let err = Error::SerializationError(json_err);
        assert!(StdError::source(&err).is_some());

        let err = Error::LockError;
        assert!(StdError::source(&err).is_none());
    }
}
