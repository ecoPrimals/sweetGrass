// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Error types for the `NestGate` store backend.

use sweet_grass_store::StoreError;

/// Errors specific to the `NestGate` store backend.
#[derive(Debug, thiserror::Error)]
pub enum NestGateStoreError {
    /// `NestGate` socket not found via discovery.
    #[error("NestGate socket not found: {0}")]
    SocketNotFound(String),

    /// Failed to connect to `NestGate` socket.
    #[error("NestGate connection failed: {0}")]
    ConnectionFailed(String),

    /// `JSON-RPC` request/response error.
    #[error("NestGate RPC error: {0}")]
    Rpc(String),

    /// Serialization or deserialization failure.
    #[error("NestGate serde error: {0}")]
    Serde(#[from] serde_json::Error),

    /// `NestGate` returned a `JSON-RPC` error response.
    #[error("NestGate returned error (code {code}): {message}")]
    JsonRpcError {
        /// JSON-RPC error code.
        code: i64,
        /// JSON-RPC error message.
        message: String,
    },

    /// I/O error communicating with `NestGate`.
    #[error("NestGate I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<NestGateStoreError> for StoreError {
    fn from(err: NestGateStoreError) -> Self {
        match err {
            NestGateStoreError::SocketNotFound(msg) | NestGateStoreError::ConnectionFailed(msg) => {
                Self::Connection(msg)
            },
            NestGateStoreError::Serde(e) => Self::Serialization(e.to_string()),
            NestGateStoreError::Io(e) => Self::Connection(e.to_string()),
            NestGateStoreError::Rpc(msg)
            | NestGateStoreError::JsonRpcError { message: msg, .. } => Self::Internal(msg),
        }
    }
}
