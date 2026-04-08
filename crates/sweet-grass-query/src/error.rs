// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Query engine error types.

use sweet_grass_core::ContentHash;
use thiserror::Error;

/// Errors that can occur during query operations.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum QueryError {
    /// Braid not found.
    #[error("Braid not found: {0}")]
    NotFound(ContentHash),

    /// Invalid query.
    #[error("Invalid query: {0}")]
    InvalidQuery(String),

    /// Maximum depth exceeded.
    #[error("Maximum traversal depth ({0}) exceeded")]
    MaxDepthExceeded(u32),

    /// Cycle detected in provenance graph.
    #[error("Cycle detected in provenance graph")]
    CycleDetected,

    /// Store error.
    #[error("Store error: {0}")]
    Store(#[from] sweet_grass_store::StoreError),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Internal error.
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<serde_json::Error> for QueryError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serialization(e.to_string())
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
    fn test_query_error_display() {
        let err = QueryError::NotFound(ContentHash::new("braid-123"));
        assert!(err.to_string().contains("not found"));

        let err = QueryError::InvalidQuery("bad filter".to_string());
        assert!(err.to_string().contains("Invalid query"));

        let err = QueryError::MaxDepthExceeded(10);
        assert!(err.to_string().contains("Maximum traversal depth"));
        assert!(err.to_string().contains("10"));

        let err = QueryError::CycleDetected;
        assert!(err.to_string().contains("Cycle detected"));

        let err = QueryError::Internal("unexpected".to_string());
        assert!(err.to_string().contains("Internal"));
    }

    #[test]
    fn test_from_serde_json_error() {
        let json_err: Result<serde_json::Value, _> = serde_json::from_str("{{");
        let err: QueryError = json_err.unwrap_err().into();
        assert!(matches!(err, QueryError::Serialization(_)));
    }

    #[test]
    fn test_store_variant_display() {
        let se = sweet_grass_store::StoreError::Internal("db corruption".to_string());
        let err = QueryError::from(se);
        assert!(err.to_string().contains("Store error"));
    }
}
