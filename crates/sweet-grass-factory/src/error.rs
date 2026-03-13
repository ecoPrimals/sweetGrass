// SPDX-License-Identifier: AGPL-3.0-only
//! Factory error types.

use thiserror::Error;

/// Errors that can occur during Braid factory operations.
#[derive(Debug, Error)]
pub enum FactoryError {
    /// Missing required data.
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// Invalid input data.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Hash computation error.
    #[error("Hash computation failed: {0}")]
    HashError(String),

    /// Signature error.
    #[error("Signature error: {0}")]
    SignatureError(String),

    /// Core error.
    #[error("Core error: {0}")]
    Core(#[from] sweet_grass_core::SweetGrassError),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Attribution calculation error.
    #[error("Attribution error: {0}")]
    Attribution(String),
}

impl From<serde_json::Error> for FactoryError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serialization(e.to_string())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_factory_error_display() {
        let err = FactoryError::MissingField("data_hash".to_string());
        assert!(err.to_string().contains("Missing required"));

        let err = FactoryError::InvalidInput("empty data".to_string());
        assert!(err.to_string().contains("Invalid input"));

        let err = FactoryError::HashError("computation failed".to_string());
        assert!(err.to_string().contains("Hash computation"));

        let err = FactoryError::SignatureError("invalid sig".to_string());
        assert!(err.to_string().contains("Signature"));

        let err = FactoryError::Attribution("calc failed".to_string());
        assert!(err.to_string().contains("Attribution"));
    }

    #[test]
    fn test_from_serde_json_error() {
        let json_err: Result<serde_json::Value, _> = serde_json::from_str("{invalid}");
        let err: FactoryError = json_err.unwrap_err().into();
        assert!(matches!(err, FactoryError::Serialization(_)));
    }
}
