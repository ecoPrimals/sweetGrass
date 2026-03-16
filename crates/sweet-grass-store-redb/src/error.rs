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
