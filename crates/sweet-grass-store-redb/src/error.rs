// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! redb store error types.

use thiserror::Error;

/// Result type for redb operations.
pub type Result<T> = std::result::Result<T, RedbError>;

/// redb store error types.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum RedbError {
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

    /// Transaction error.
    #[error("Transaction error: {0}")]
    Transaction(String),
}

impl From<redb::Error> for RedbError {
    fn from(err: redb::Error) -> Self {
        Self::Read(err.to_string())
    }
}

impl From<redb::DatabaseError> for RedbError {
    fn from(err: redb::DatabaseError) -> Self {
        Self::Open(err.to_string())
    }
}

impl From<redb::TableError> for RedbError {
    fn from(err: redb::TableError) -> Self {
        Self::Read(err.to_string())
    }
}

impl From<redb::TransactionError> for RedbError {
    fn from(err: redb::TransactionError) -> Self {
        Self::Transaction(err.to_string())
    }
}

impl From<redb::StorageError> for RedbError {
    fn from(err: redb::StorageError) -> Self {
        Self::Read(err.to_string())
    }
}

impl From<redb::CommitError> for RedbError {
    fn from(err: redb::CommitError) -> Self {
        Self::Transaction(err.to_string())
    }
}

impl From<serde_json::Error> for RedbError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

impl From<RedbError> for sweet_grass_store::StoreError {
    fn from(err: RedbError) -> Self {
        match err {
            RedbError::NotFound(msg) => Self::NotFound(msg),
            _ => Self::Internal(err.to_string()),
        }
    }
}

impl RedbError {
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
#[expect(
    clippy::unwrap_used,
    reason = "test module: unwrap is standard in tests"
)]
mod tests {
    use super::*;
    use sweet_grass_store::StoreError;

    #[test]
    fn test_error_variants_display() {
        assert_eq!(
            RedbError::Open("path error".to_string()).to_string(),
            "Database open error: path error"
        );
        assert_eq!(
            RedbError::Read("read failed".to_string()).to_string(),
            "Read error: read failed"
        );
        assert_eq!(
            RedbError::Write("write failed".to_string()).to_string(),
            "Write error: write failed"
        );
        assert_eq!(
            RedbError::Delete("delete failed".to_string()).to_string(),
            "Delete error: delete failed"
        );
        assert_eq!(
            RedbError::Serialization("json parse error".to_string()).to_string(),
            "Serialization error: json parse error"
        );
        assert_eq!(
            RedbError::NotFound("key-123".to_string()).to_string(),
            "Record not found: key-123"
        );
        assert_eq!(
            RedbError::Transaction("commit failed".to_string()).to_string(),
            "Transaction error: commit failed"
        );
    }

    #[test]
    fn test_is_not_found() {
        assert!(RedbError::NotFound("x".to_string()).is_not_found());
        assert!(!RedbError::Open("x".to_string()).is_not_found());
        assert!(!RedbError::Read("x".to_string()).is_not_found());
        assert!(!RedbError::Write("x".to_string()).is_not_found());
        assert!(!RedbError::Delete("x".to_string()).is_not_found());
        assert!(!RedbError::Serialization("x".to_string()).is_not_found());
        assert!(!RedbError::Transaction("x".to_string()).is_not_found());
    }

    #[test]
    fn test_is_retriable() {
        assert!(RedbError::Transaction("x".to_string()).is_retriable());
        assert!(!RedbError::NotFound("x".to_string()).is_retriable());
        assert!(!RedbError::Open("x".to_string()).is_retriable());
        assert!(!RedbError::Read("x".to_string()).is_retriable());
        assert!(!RedbError::Write("x".to_string()).is_retriable());
        assert!(!RedbError::Delete("x".to_string()).is_retriable());
        assert!(!RedbError::Serialization("x".to_string()).is_retriable());
    }

    #[test]
    fn test_from_redb_error() {
        let err: redb::Error =
            redb::TableError::TableDoesNotExist("missing_table".to_string()).into();
        let redb_err: RedbError = err.into();
        assert!(matches!(redb_err, RedbError::Read(_)));
        assert!(redb_err.to_string().contains("missing_table"));
    }

    #[test]
    fn test_from_database_error() {
        let err: redb::DatabaseError = redb::DatabaseError::DatabaseAlreadyOpen;
        let redb_err: RedbError = err.into();
        assert!(matches!(redb_err, RedbError::Open(_)));
    }

    #[test]
    fn test_from_table_error() {
        let err: redb::TableError = redb::TableError::TableDoesNotExist("test_table".to_string());
        let redb_err: RedbError = err.into();
        assert!(matches!(redb_err, RedbError::Read(_)));
        assert!(redb_err.to_string().contains("test_table"));
    }

    #[test]
    fn test_from_transaction_error() {
        let storage_err = redb::StorageError::Corrupted("corrupt data".to_string());
        let err: redb::TransactionError = redb::TransactionError::Storage(storage_err);
        let redb_err: RedbError = err.into();
        assert!(matches!(redb_err, RedbError::Transaction(_)));
    }

    #[test]
    fn test_from_storage_error() {
        let err: redb::StorageError = redb::StorageError::Corrupted("storage corrupt".to_string());
        let redb_err: RedbError = err.into();
        assert!(matches!(redb_err, RedbError::Read(_)));
    }

    #[test]
    fn test_from_commit_error() {
        let storage_err = redb::StorageError::Corrupted("commit failed".to_string());
        let err: redb::CommitError = redb::CommitError::Storage(storage_err);
        let redb_err: RedbError = err.into();
        assert!(matches!(redb_err, RedbError::Transaction(_)));
    }

    #[test]
    fn test_from_serde_json_error() {
        let err: serde_json::Error = serde_json::from_str::<i32>("not a number").unwrap_err();
        let redb_err: RedbError = err.into();
        assert!(matches!(redb_err, RedbError::Serialization(_)));
    }

    #[test]
    fn test_from_redb_error_to_store_error_not_found() {
        let redb_err = RedbError::NotFound("braid-xyz".to_string());
        let store_err: StoreError = redb_err.into();
        assert!(matches!(store_err, StoreError::NotFound(s) if s == "braid-xyz"));
    }

    #[test]
    fn test_from_redb_error_to_store_error_internal() {
        let redb_err = RedbError::Read("read failed".to_string());
        let store_err: StoreError = redb_err.into();
        assert!(matches!(store_err, StoreError::Internal(_)));
        assert!(store_err.to_string().contains("read failed"));
    }
}
