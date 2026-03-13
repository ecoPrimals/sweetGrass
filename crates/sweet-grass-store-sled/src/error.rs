// SPDX-License-Identifier: AGPL-3.0-only
//! Sled store error types.

use thiserror::Error;

/// Result type for Sled operations.
pub type Result<T> = std::result::Result<T, SledError>;

/// Sled store error types.
#[derive(Debug, Error)]
pub enum SledError {
    /// Database open error.
    #[error("Database open error: {0}")]
    Open(String),

    /// Read error.
    #[error("Read error: {0}")]
    Read(String),

    /// Write error.
    #[error("Write error: {0}")]
    Write(String),

    /// Delete error.
    #[error("Delete error: {0}")]
    Delete(String),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Record not found.
    #[error("Record not found: {0}")]
    NotFound(String),

    /// Tree error.
    #[error("Tree error: {0}")]
    Tree(String),

    /// Transaction error.
    #[error("Transaction error: {0}")]
    Transaction(String),
}

impl From<sled::Error> for SledError {
    fn from(err: sled::Error) -> Self {
        Self::Read(err.to_string())
    }
}

impl From<serde_json::Error> for SledError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

impl From<SledError> for sweet_grass_store::StoreError {
    fn from(err: SledError) -> Self {
        match err {
            SledError::NotFound(msg) => Self::NotFound(msg),
            _ => Self::Internal(err.to_string()),
        }
    }
}

impl SledError {
    /// Check if this is a not-found error.
    #[must_use]
    pub const fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound(_))
    }

    /// Check if this is a retriable error (transient failures).
    #[must_use]
    pub const fn is_retriable(&self) -> bool {
        matches!(self, Self::Transaction(_))
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_sled_error_display() {
        let err = SledError::Open("permission denied".to_string());
        assert_eq!(err.to_string(), "Database open error: permission denied");

        let err = SledError::Read("disk error".to_string());
        assert_eq!(err.to_string(), "Read error: disk error");

        let err = SledError::Write("disk full".to_string());
        assert_eq!(err.to_string(), "Write error: disk full");

        let err = SledError::Delete("key not found".to_string());
        assert_eq!(err.to_string(), "Delete error: key not found");

        let err = SledError::Serialization("invalid json".to_string());
        assert_eq!(err.to_string(), "Serialization error: invalid json");

        let err = SledError::NotFound("record-123".to_string());
        assert_eq!(err.to_string(), "Record not found: record-123");

        let err = SledError::Tree("tree corruption".to_string());
        assert_eq!(err.to_string(), "Tree error: tree corruption");

        let err = SledError::Transaction("conflict".to_string());
        assert_eq!(err.to_string(), "Transaction error: conflict");
    }

    #[test]
    fn test_sled_error_predicates() {
        assert!(SledError::NotFound("x".to_string()).is_not_found());
        assert!(!SledError::Read("x".to_string()).is_not_found());

        assert!(SledError::Transaction("x".to_string()).is_retriable());
        assert!(!SledError::Read("x".to_string()).is_retriable());
    }

    #[test]
    fn test_from_json_error() {
        let json_err = serde_json::from_str::<i32>("invalid").unwrap_err();
        let err: SledError = json_err.into();
        assert!(matches!(err, SledError::Serialization(_)));
    }

    #[test]
    fn test_into_store_error_not_found() {
        let err = SledError::NotFound("test-id".to_string());
        let store_err: sweet_grass_store::StoreError = err.into();
        assert!(matches!(store_err, sweet_grass_store::StoreError::NotFound(s) if s == "test-id"));
    }

    #[test]
    fn test_into_store_error_other() {
        let err = SledError::Read("disk error".to_string());
        let store_err: sweet_grass_store::StoreError = err.into();
        assert!(matches!(
            store_err,
            sweet_grass_store::StoreError::Internal(_)
        ));
    }
}
