// SPDX-License-Identifier: AGPL-3.0-only
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

    /// Capability provider error (structured, vendor-agnostic).
    ///
    /// Matches rhizoCrypt's and LoamSpine's `CapabilityProvider` for ecosystem consistency.
    #[error("capability provider error ({capability}): {message}")]
    CapabilityProvider {
        /// The capability that failed.
        capability: String,
        /// Error detail.
        message: String,
    },

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
                | Self::CapabilityProvider { .. }
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

    /// Create a capability provider error.
    #[must_use]
    pub fn capability_provider(capability: impl Into<String>, message: impl Into<String>) -> Self {
        Self::CapabilityProvider {
            capability: capability.into(),
            message: message.into(),
        }
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

    #[test]
    fn test_all_error_variants_display() {
        // Test all error variants have proper display messages
        let errors = vec![
            SweetGrassError::InvalidBraid("test".to_string()),
            SweetGrassError::SignatureVerification("bad sig".to_string()),
            SweetGrassError::BraidExists("exists".to_string()),
            SweetGrassError::InvalidHash("bad hash".to_string()),
            SweetGrassError::StorageConnection("conn failed".to_string()),
            SweetGrassError::Query("query failed".to_string()),
            SweetGrassError::QueryTimeout(std::time::Duration::from_secs(30)),
            SweetGrassError::Attribution("calc failed".to_string()),
            SweetGrassError::CircularDependency("loop detected".to_string()),
            SweetGrassError::Compression("compress failed".to_string()),
            SweetGrassError::SessionEvents("event error".to_string()),
            SweetGrassError::Anchoring("anchor error".to_string()),
            SweetGrassError::Signing("sign error".to_string()),
            SweetGrassError::Compute("compute error".to_string()),
            SweetGrassError::Discovery("discovery error".to_string()),
            SweetGrassError::CapabilityProvider {
                capability: "signing".to_string(),
                message: "HSM unavailable".to_string(),
            },
            SweetGrassError::Config("config error".to_string()),
            SweetGrassError::InvalidConfig {
                field: "port".to_string(),
                value: "invalid".to_string(),
            },
            SweetGrassError::NotRunning("stopped".to_string()),
            SweetGrassError::AlreadyRunning,
            SweetGrassError::StartupFailed("init failed".to_string()),
            SweetGrassError::ShutdownFailed("stop failed".to_string()),
            SweetGrassError::Internal("internal".to_string()),
            SweetGrassError::NotImplemented("feature".to_string()),
        ];

        for err in errors {
            // All errors should have non-empty display messages
            assert!(!err.to_string().is_empty());
        }
    }

    #[test]
    fn test_retriable_errors_comprehensive() {
        // Test all retriable errors
        assert!(SweetGrassError::Storage("err".to_string()).is_retriable());
        assert!(SweetGrassError::StorageConnection("err".to_string()).is_retriable());
        assert!(SweetGrassError::QueryTimeout(std::time::Duration::from_secs(1)).is_retriable());
        assert!(SweetGrassError::SessionEvents("err".to_string()).is_retriable());
        assert!(SweetGrassError::Anchoring("err".to_string()).is_retriable());
        assert!(SweetGrassError::Signing("err".to_string()).is_retriable());
        assert!(SweetGrassError::Compute("err".to_string()).is_retriable());
        assert!(SweetGrassError::Discovery("err".to_string()).is_retriable());

        // Test all non-retriable errors
        assert!(!SweetGrassError::InvalidBraid("err".to_string()).is_retriable());
        assert!(!SweetGrassError::Validation("err".to_string()).is_retriable());
        assert!(!SweetGrassError::CircularDependency("err".to_string()).is_retriable());
        assert!(!SweetGrassError::AlreadyRunning.is_retriable());
    }

    #[test]
    fn test_validation_errors_comprehensive() {
        // Test all validation errors
        assert!(SweetGrassError::Validation("bad".to_string()).is_validation());
        assert!(SweetGrassError::MissingField("field".to_string()).is_validation());
        assert!(SweetGrassError::InvalidHash("hash".to_string()).is_validation());
        assert!(SweetGrassError::InvalidBraid("braid".to_string()).is_validation());

        // Test non-validation errors
        assert!(!SweetGrassError::BraidNotFound("id".to_string()).is_validation());
        assert!(!SweetGrassError::Internal("err".to_string()).is_validation());
    }

    #[test]
    fn test_not_found_errors_comprehensive() {
        // Test all not-found errors
        assert!(SweetGrassError::BraidNotFound("id".to_string()).is_not_found());
        assert!(SweetGrassError::SessionNotFound("id".to_string()).is_not_found());

        // Test non-not-found errors
        assert!(!SweetGrassError::BraidExists("id".to_string()).is_not_found());
        assert!(!SweetGrassError::Validation("err".to_string()).is_not_found());
    }

    #[test]
    fn test_query_timeout_duration() {
        let duration = std::time::Duration::from_secs(42);
        let err = SweetGrassError::QueryTimeout(duration);
        let msg = err.to_string();
        assert!(msg.contains("query timeout"));
        assert!(msg.contains("42"));
    }

    #[test]
    fn test_invalid_config_fields() {
        let err = SweetGrassError::InvalidConfig {
            field: "max_depth".to_string(),
            value: "-1".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("max_depth"));
        assert!(msg.contains("-1"));
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: SweetGrassError = io_err.into();
        assert!(matches!(err, SweetGrassError::Io(_)));
    }

    #[test]
    fn test_capability_provider_error() {
        let err = SweetGrassError::capability_provider("signing", "HSM unavailable");
        assert!(err.to_string().contains("capability provider error"));
        assert!(err.to_string().contains("signing"));
        assert!(err.to_string().contains("HSM unavailable"));
        assert!(err.is_retriable());
    }

    #[test]
    fn test_error_debug_impl() {
        // Ensure Debug impl works for all error variants
        let err = SweetGrassError::BraidNotFound("test".to_string());
        let debug_str = format!("{err:?}");
        assert!(!debug_str.is_empty());
    }
}
