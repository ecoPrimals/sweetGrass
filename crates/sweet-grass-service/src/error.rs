// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Error types for the service layer.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

/// Service error types.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ServiceError {
    /// Entity not found.
    #[error("not found: {0}")]
    NotFound(String),

    /// Invalid request input.
    #[error("bad request: {0}")]
    BadRequest(String),

    /// Internal server error.
    #[error("internal error: {0}")]
    Internal(String),

    /// Store error.
    #[error("store error: {0}")]
    Store(#[from] sweet_grass_store::StoreError),

    /// Query error.
    #[error("query error: {0}")]
    Query(#[from] sweet_grass_query::QueryError),

    /// Factory error.
    #[error("factory error: {0}")]
    Factory(#[from] sweet_grass_factory::FactoryError),

    /// Compression error.
    #[error("compression error: {0}")]
    Compression(#[from] sweet_grass_compression::CompressionError),

    /// Core error.
    #[error("core error: {0}")]
    Core(#[from] sweet_grass_core::SweetGrassError),

    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// IO error (UDS, TCP, file operations).
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// IPC transport error (UDS/tarpc communication failure).
    #[error("transport error: {0}")]
    Transport(String),

    /// Capability discovery failed for a required service.
    #[error("discovery error ({capability}): {message}")]
    Discovery {
        /// The capability that could not be discovered.
        capability: String,
        /// Error detail.
        message: String,
    },
}

/// Error response body.
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        let (status, error_type) = match &self {
            Self::NotFound(_) => (StatusCode::NOT_FOUND, "not_found"),
            Self::BadRequest(_) => (StatusCode::BAD_REQUEST, "bad_request"),
            Self::Serialization(_) => (StatusCode::BAD_REQUEST, "serialization_error"),
            Self::Store(_) | Self::Query(_) | Self::Factory(_) | Self::Compression(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "processing_error")
            },
            Self::Io(_) | Self::Core(_) | Self::Internal(_) | Self::Transport(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error")
            },
            Self::Discovery { .. } => (StatusCode::SERVICE_UNAVAILABLE, "discovery_error"),
        };

        let body = ErrorResponse {
            error: error_type.to_string(),
            message: self.to_string(),
        };

        (status, Json(body)).into_response()
    }
}

impl From<serde_json::Error> for ServiceError {
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
    use axum::response::IntoResponse;

    #[test]
    fn test_service_error_display() {
        let err = ServiceError::NotFound("braid-xyz".to_string());
        assert!(err.to_string().contains("not found"));

        let err = ServiceError::BadRequest("missing field".to_string());
        assert!(err.to_string().contains("bad request"));

        let err = ServiceError::Internal("oops".to_string());
        assert!(err.to_string().contains("internal"));

        let err = ServiceError::Serialization("json parse failed".to_string());
        assert!(err.to_string().contains("serialization"));
    }

    #[test]
    fn test_into_response_not_found() {
        let err = ServiceError::NotFound("missing".to_string());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_into_response_bad_request() {
        let err = ServiceError::BadRequest("invalid".to_string());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_into_response_internal() {
        let err = ServiceError::Internal("boom".to_string());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_from_serde_json_error() {
        let json_err: Result<serde_json::Value, _> = serde_json::from_str("not json");
        let err: ServiceError = json_err.unwrap_err().into();
        assert!(matches!(err, ServiceError::Serialization(_)));
    }

    #[test]
    fn test_into_response_serialization() {
        let err = ServiceError::Serialization("bad json".to_string());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_into_response_store_error() {
        let store_err = sweet_grass_store::StoreError::Internal("db down".to_string());
        let err = ServiceError::from(store_err);
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_into_response_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let err = ServiceError::from(io_err);
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_into_response_transport() {
        let err = ServiceError::Transport("connection reset".to_string());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_into_response_discovery() {
        let err = ServiceError::Discovery {
            capability: "signing".to_string(),
            message: "no endpoint".to_string(),
        };
        assert!(err.to_string().contains("signing"));
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn test_error_display_variants() {
        let err = ServiceError::Transport("timeout".to_string());
        assert!(err.to_string().contains("transport"));

        let err = ServiceError::Discovery {
            capability: "compute".to_string(),
            message: "unreachable".to_string(),
        };
        assert!(err.to_string().contains("compute"));
        assert!(err.to_string().contains("unreachable"));
    }
}
