//! `SweetGrass` error types.
//!
//! Comprehensive error handling for all `SweetGrass` operations.

use thiserror::Error;

/// Errors specific to `SweetGrass`.
#[derive(Debug, Error)]
pub enum SweetGrassError {
    // ==================== Braid Errors ====================
    /// Braid not found.
    #[error("braid not found: {0}")]
    BraidNotFound(String),

    /// Invalid Braid structure.
    #[error("invalid braid: {0}")]
    InvalidBraid(String),

    /// Braid signature verification failed.
    #[error("signature verification failed: {0}")]
    SignatureVerification(String),

    /// Braid already exists.
    #[error("braid already exists: {0}")]
    BraidExists(String),

    // ==================== Validation Errors ====================
    /// Validation error.
    #[error("validation error: {0}")]
    Validation(String),

    /// Missing required field.
    #[error("missing required field: {0}")]
    MissingField(String),

    /// Invalid content hash.
    #[error("invalid content hash: {0}")]
    InvalidHash(String),

    // ==================== Storage Errors ====================
    /// Storage error.
    #[error("storage error: {0}")]
    Storage(String),

    /// Storage connection failed.
    #[error("storage connection failed: {0}")]
    StorageConnection(String),

    // ==================== Query Errors ====================
    /// Query error.
    #[error("query error: {0}")]
    Query(String),

    /// Query timeout.
    #[error("query timeout after {0:?}")]
    QueryTimeout(std::time::Duration),

    /// Query depth exceeded.
    #[error("query depth exceeded: {depth} > {max}")]
    QueryDepthExceeded {
        /// Actual depth.
        depth: u32,
        /// Maximum allowed depth.
        max: u32,
    },

    // ==================== Attribution Errors ====================
    /// Attribution calculation error.
    #[error("attribution error: {0}")]
    Attribution(String),

    /// Circular dependency in provenance graph.
    #[error("circular dependency detected: {0}")]
    CircularDependency(String),

    // ==================== Compression Errors ====================
    /// Compression error.
    #[error("compression error: {0}")]
    Compression(String),

    /// Session not found.
    #[error("session not found: {0}")]
    SessionNotFound(String),

    // ==================== Integration Errors (Capability-based) ====================
    /// Session events integration error.
    #[error("session events error: {0}")]
    SessionEvents(String),

    /// Anchoring integration error.
    #[error("anchoring error: {0}")]
    Anchoring(String),

    /// Signing integration error.
    #[error("signing error: {0}")]
    Signing(String),

    /// Compute integration error.
    #[error("compute error: {0}")]
    Compute(String),

    /// Discovery integration error.
    #[error("discovery error: {0}")]
    Discovery(String),

    // ==================== Configuration Errors ====================
    /// Configuration error.
    #[error("configuration error: {0}")]
    Config(String),

    /// Invalid configuration value.
    #[error("invalid configuration: {field} = {value}")]
    InvalidConfig {
        /// Field name.
        field: String,
        /// Invalid value.
        value: String,
    },

    // ==================== Primal Lifecycle Errors ====================
    /// Primal not running.
    #[error("primal not running, current state: {0}")]
    NotRunning(String),

    /// Primal already running.
    #[error("primal already running")]
    AlreadyRunning,

    /// Startup failed.
    #[error("startup failed: {0}")]
    StartupFailed(String),

    /// Shutdown failed.
    #[error("shutdown failed: {0}")]
    ShutdownFailed(String),

    // ==================== Serialization Errors ====================
    /// JSON serialization error.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    // ==================== Generic Errors ====================
    /// Internal error.
    #[error("internal error: {0}")]
    Internal(String),

    /// Not implemented.
    #[error("not implemented: {0}")]
    NotImplemented(String),

    /// IO error.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl SweetGrassError {
    /// Check if this is a retriable error.
    ///
    /// # Errors
    ///
    /// Returns `true` for transient errors that may succeed on retry.
    #[must_use]
    pub const fn is_retriable(&self) -> bool {
        matches!(
            self,
            Self::Storage(_)
                | Self::StorageConnection(_)
                | Self::QueryTimeout(_)
                | Self::SessionEvents(_)
                | Self::Anchoring(_)
                | Self::Signing(_)
                | Self::Compute(_)
                | Self::Discovery(_)
        )
    }

    /// Check if this is a validation error.
    #[must_use]
    pub const fn is_validation(&self) -> bool {
        matches!(
            self,
            Self::Validation(_)
                | Self::MissingField(_)
                | Self::InvalidHash(_)
                | Self::InvalidBraid(_)
        )
    }

    /// Check if this is a not-found error.
    #[must_use]
    pub const fn is_not_found(&self) -> bool {
        matches!(self, Self::BraidNotFound(_) | Self::SessionNotFound(_))
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = SweetGrassError::BraidNotFound("test-id".to_string());
        assert_eq!(err.to_string(), "braid not found: test-id");
    }

    #[test]
    fn test_error_retriable() {
        assert!(SweetGrassError::Storage("timeout".to_string()).is_retriable());
        assert!(!SweetGrassError::Validation("bad data".to_string()).is_retriable());
    }

    #[test]
    fn test_error_validation() {
        assert!(SweetGrassError::Validation("bad".to_string()).is_validation());
        assert!(SweetGrassError::MissingField("field".to_string()).is_validation());
        assert!(!SweetGrassError::Storage("err".to_string()).is_validation());
    }

    #[test]
    fn test_error_not_found() {
        assert!(SweetGrassError::BraidNotFound("id".to_string()).is_not_found());
        assert!(SweetGrassError::SessionNotFound("id".to_string()).is_not_found());
        assert!(!SweetGrassError::Validation("err".to_string()).is_not_found());
    }

    #[test]
    fn test_query_depth_exceeded() {
        let err = SweetGrassError::QueryDepthExceeded { depth: 15, max: 10 };
        assert!(err.to_string().contains("15"));
        assert!(err.to_string().contains("10"));
    }

    #[test]
    fn test_from_json_error() {
        let json_err = serde_json::from_str::<i32>("invalid").unwrap_err();
        let err: SweetGrassError = json_err.into();
        assert!(matches!(err, SweetGrassError::Json(_)));
    }
}
