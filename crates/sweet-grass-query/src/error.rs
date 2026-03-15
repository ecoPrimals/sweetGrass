// SPDX-License-Identifier: AGPL-3.0-only
//! Query engine error types.

use thiserror::Error;

/// Errors that can occur during query operations.
#[derive(Debug, Error)]
pub enum QueryError {
    /// Braid not found.
    #[error("Braid not found: {0}")]
    NotFound(String),

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
        let err = QueryError::NotFound("braid-123".to_string());
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
}
