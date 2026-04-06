// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Compression error types.

use thiserror::Error;

/// Errors that can occur during compression.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CompressionError {
    /// Session analysis failed.
    #[error("Analysis error: {0}")]
    Analysis(String),

    /// Invalid session state.
    #[error("Invalid session: {0}")]
    InvalidSession(String),

    /// Braid creation failed.
    #[error("Factory error: {0}")]
    Factory(#[from] sweet_grass_factory::FactoryError),

    /// Core type error.
    #[error("Core error: {0}")]
    Core(#[from] sweet_grass_core::SweetGrassError),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Session has no committed vertices to compress.
    #[error("No committed vertices in session")]
    NoCommittedVertices,

    /// Internal error.
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<serde_json::Error> for CompressionError {
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
    fn test_compression_error_display() {
        let err = CompressionError::Analysis("test".to_string());
        assert!(err.to_string().contains("Analysis"));

        let err = CompressionError::InvalidSession("bad state".to_string());
        assert!(err.to_string().contains("Invalid session"));

        let err = CompressionError::Serialization("json error".to_string());
        assert!(err.to_string().contains("Serialization"));

        let err = CompressionError::Config("bad config".to_string());
        assert!(err.to_string().contains("Configuration"));

        let err = CompressionError::Internal("oops".to_string());
        assert!(err.to_string().contains("Internal"));
    }

    #[test]
    fn test_from_serde_json_error() {
        let json_err: Result<serde_json::Value, _> = serde_json::from_str("not valid json");
        let err: CompressionError = json_err.unwrap_err().into();
        assert!(matches!(err, CompressionError::Serialization(_)));
    }
}
