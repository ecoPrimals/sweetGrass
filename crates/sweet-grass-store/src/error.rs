// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Storage error types.

use thiserror::Error;

/// Errors that can occur during store operations.
#[derive(Debug, Error)]
pub enum StoreError {
    /// Braid not found.
    #[error("Braid not found: {0}")]
    NotFound(String),

    /// Duplicate Braid ID.
    #[error("Braid already exists: {0}")]
    Duplicate(String),

    /// Invalid query.
    #[error("Invalid query: {0}")]
    InvalidQuery(String),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Index error.
    #[error("Index error: {0}")]
    Index(String),

    /// Connection error.
    #[error("Connection error: {0}")]
    Connection(String),

    /// Transaction error.
    #[error("Transaction error: {0}")]
    Transaction(String),

    /// Internal storage error.
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<serde_json::Error> for StoreError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serialization(e.to_string())
    }
}

impl StoreError {
    /// Check if this is a not-found error.
    #[must_use]
    pub const fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound(_))
    }

    /// Check if this is a connection error.
    #[must_use]
    pub const fn is_connection(&self) -> bool {
        matches!(self, Self::Connection(_))
    }

    /// Check if this is a retriable error.
    #[must_use]
    pub const fn is_retriable(&self) -> bool {
        matches!(self, Self::Connection(_) | Self::Transaction(_))
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test module: unwrap is standard in tests"
)]
mod tests {
    use super::*;

    #[test]
    fn test_store_error_display() {
        let err = StoreError::NotFound("braid-123".to_string());
        assert_eq!(err.to_string(), "Braid not found: braid-123");

        let err = StoreError::Duplicate("braid-456".to_string());
        assert_eq!(err.to_string(), "Braid already exists: braid-456");

        let err = StoreError::InvalidQuery("bad filter".to_string());
        assert_eq!(err.to_string(), "Invalid query: bad filter");

        let err = StoreError::Serialization("json parse error".to_string());
        assert_eq!(err.to_string(), "Serialization error: json parse error");

        let err = StoreError::Index("index corruption".to_string());
        assert_eq!(err.to_string(), "Index error: index corruption");

        let err = StoreError::Connection("timeout".to_string());
        assert_eq!(err.to_string(), "Connection error: timeout");

        let err = StoreError::Transaction("deadlock".to_string());
        assert_eq!(err.to_string(), "Transaction error: deadlock");

        let err = StoreError::Internal("unexpected state".to_string());
        assert_eq!(err.to_string(), "Internal error: unexpected state");
    }

    #[test]
    fn test_store_error_predicates() {
        assert!(StoreError::NotFound("x".to_string()).is_not_found());
        assert!(!StoreError::Connection("x".to_string()).is_not_found());

        assert!(StoreError::Connection("x".to_string()).is_connection());
        assert!(!StoreError::NotFound("x".to_string()).is_connection());

        assert!(StoreError::Connection("x".to_string()).is_retriable());
        assert!(StoreError::Transaction("x".to_string()).is_retriable());
        assert!(!StoreError::NotFound("x".to_string()).is_retriable());
    }

    #[test]
    fn test_from_json_error() {
        let json_err = serde_json::from_str::<i32>("invalid").unwrap_err();
        let err: StoreError = json_err.into();
        assert!(matches!(err, StoreError::Serialization(_)));
    }
}
