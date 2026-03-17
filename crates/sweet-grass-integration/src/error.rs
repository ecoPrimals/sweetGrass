// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Error types for integration operations.
//!
//! Structured IPC error phases align with ecosystem partners (rhizoCrypt,
//! healthSpring V28) for consistent observability across the provenance trio.

use std::fmt;

use thiserror::Error;

/// Phase of an IPC call that failed.
///
/// Enables targeted retries and diagnostics without a logging dependency.
/// Aligned with `rhizoCrypt::IpcErrorPhase` and healthSpring V28 standard.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IpcErrorPhase {
    /// Socket/TCP connection failed (primal unreachable or socket missing).
    Connect,
    /// Request write to socket failed (broken pipe, timeout).
    Write,
    /// Response read from socket failed (timeout, truncated).
    Read,
    /// Response is not valid JSON.
    InvalidJson,
    /// HTTP response status was not 2xx.
    HttpStatus(u16),
    /// Response lacks a `result` field (JSON-RPC protocol violation).
    NoResult,
    /// JSON-RPC error object returned by the remote primal.
    JsonRpcError(i64),
}

impl IpcErrorPhase {
    /// Whether this phase represents a transport-level failure that may
    /// succeed on retry (network flake, temporary overload).
    ///
    /// Aligned with loamSpine `IpcPhase::is_recoverable()` and rhizoCrypt
    /// `should_retry()` gating.
    #[must_use]
    pub const fn is_retriable(&self) -> bool {
        matches!(
            self,
            Self::Connect | Self::Write | Self::Read | Self::HttpStatus(502..=504)
        )
    }

    /// Whether this phase likely indicates a timeout rather than a hard error.
    #[must_use]
    pub const fn is_timeout_likely(&self) -> bool {
        matches!(self, Self::Read | Self::HttpStatus(504))
    }

    /// Whether the remote primal reported "method not found" (-32601).
    #[must_use]
    pub const fn is_method_not_found(&self) -> bool {
        matches!(self, Self::JsonRpcError(-32601))
    }

    /// Whether this is an application-level error (remote primal understood
    /// the request but returned an error). Not retriable — the request itself
    /// is the problem.
    #[must_use]
    pub const fn is_application_error(&self) -> bool {
        matches!(
            self,
            Self::JsonRpcError(_) | Self::HttpStatus(400 | 404 | 422)
        )
    }
}

impl fmt::Display for IpcErrorPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connect => write!(f, "connect"),
            Self::Write => write!(f, "write"),
            Self::Read => write!(f, "read"),
            Self::InvalidJson => write!(f, "invalid_json"),
            Self::HttpStatus(code) => write!(f, "http_{code}"),
            Self::NoResult => write!(f, "no_result"),
            Self::JsonRpcError(code) => write!(f, "jsonrpc_{code}"),
        }
    }
}

/// Integration error types.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum IntegrationError {
    /// Structured IPC error with phase context.
    ///
    /// Provides observability into which phase of the IPC call failed,
    /// enabling targeted retries and diagnostics.
    #[error("IPC error ({phase}): {message}")]
    Ipc {
        /// Phase of the IPC call that failed.
        phase: IpcErrorPhase,
        /// Human-readable error detail.
        message: String,
    },

    /// Discovery failed — no primal found offering required capability.
    #[error("Discovery error: {0}")]
    Discovery(String),

    /// Connection to primal failed.
    #[error("Connection error: {0}")]
    Connection(String),

    /// Session events service connection failed.
    #[error("Session events connection error: {0}")]
    SessionEventsConnection(String),

    /// Anchoring service connection failed.
    #[error("Anchoring connection error: {0}")]
    AnchoringConnection(String),

    /// Signing service connection failed.
    #[error("Signing connection error: {0}")]
    SigningConnection(String),

    /// Event processing failed.
    #[error("Event processing error: {0}")]
    EventProcessing(String),

    /// Subscription to events failed.
    #[error("Subscription error: {0}")]
    Subscription(String),

    /// Anchoring failed.
    #[error("Anchoring error: {0}")]
    Anchoring(String),

    /// Signing failed.
    #[error("Signing error: {0}")]
    Signing(String),

    /// Verification failed.
    #[error("Verification error: {0}")]
    Verification(String),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// RPC communication error.
    #[error("RPC error: {0}")]
    Rpc(String),

    /// Compression error.
    #[error("Compression error: {0}")]
    Compression(#[from] sweet_grass_compression::CompressionError),

    /// Store error.
    #[error("Store error: {0}")]
    Store(#[from] sweet_grass_store::StoreError),

    /// Core error.
    #[error("Core error: {0}")]
    Core(#[from] sweet_grass_core::SweetGrassError),

    /// Operation timeout.
    #[error("Operation timeout: {0}")]
    Timeout(String),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Feature not yet implemented.
    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

impl IntegrationError {
    /// Create a structured IPC error with phase context.
    #[must_use]
    pub fn ipc(phase: IpcErrorPhase, msg: impl Into<String>) -> Self {
        Self::Ipc {
            phase,
            message: msg.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipc_error_phase_display() {
        assert_eq!(IpcErrorPhase::Connect.to_string(), "connect");
        assert_eq!(IpcErrorPhase::Write.to_string(), "write");
        assert_eq!(IpcErrorPhase::Read.to_string(), "read");
        assert_eq!(IpcErrorPhase::InvalidJson.to_string(), "invalid_json");
        assert_eq!(IpcErrorPhase::HttpStatus(503).to_string(), "http_503");
        assert_eq!(IpcErrorPhase::NoResult.to_string(), "no_result");
        assert_eq!(
            IpcErrorPhase::JsonRpcError(-32600).to_string(),
            "jsonrpc_-32600"
        );
    }

    #[test]
    fn structured_ipc_error() {
        let err = IntegrationError::ipc(IpcErrorPhase::Connect, "connection refused");
        assert!(err.to_string().contains("connect"));
        assert!(err.to_string().contains("connection refused"));
    }

    #[test]
    fn ipc_error_phase_equality() {
        assert_eq!(IpcErrorPhase::Connect, IpcErrorPhase::Connect);
        assert_ne!(IpcErrorPhase::Connect, IpcErrorPhase::Read);
        assert_eq!(
            IpcErrorPhase::HttpStatus(404),
            IpcErrorPhase::HttpStatus(404)
        );
        assert_ne!(
            IpcErrorPhase::HttpStatus(404),
            IpcErrorPhase::HttpStatus(503)
        );
    }

    #[test]
    fn is_retriable_transport_errors() {
        assert!(IpcErrorPhase::Connect.is_retriable());
        assert!(IpcErrorPhase::Write.is_retriable());
        assert!(IpcErrorPhase::Read.is_retriable());
        assert!(IpcErrorPhase::HttpStatus(503).is_retriable());
        assert!(IpcErrorPhase::HttpStatus(502).is_retriable());
        assert!(IpcErrorPhase::HttpStatus(504).is_retriable());

        assert!(!IpcErrorPhase::InvalidJson.is_retriable());
        assert!(!IpcErrorPhase::NoResult.is_retriable());
        assert!(!IpcErrorPhase::JsonRpcError(-32600).is_retriable());
        assert!(!IpcErrorPhase::HttpStatus(400).is_retriable());
        assert!(!IpcErrorPhase::HttpStatus(404).is_retriable());
    }

    #[test]
    fn is_timeout_likely() {
        assert!(IpcErrorPhase::Read.is_timeout_likely());
        assert!(IpcErrorPhase::HttpStatus(504).is_timeout_likely());
        assert!(!IpcErrorPhase::Connect.is_timeout_likely());
        assert!(!IpcErrorPhase::Write.is_timeout_likely());
    }

    #[test]
    fn is_method_not_found() {
        assert!(IpcErrorPhase::JsonRpcError(-32601).is_method_not_found());
        assert!(!IpcErrorPhase::JsonRpcError(-32600).is_method_not_found());
        assert!(!IpcErrorPhase::Connect.is_method_not_found());
    }

    #[test]
    fn is_application_error() {
        assert!(IpcErrorPhase::JsonRpcError(-32600).is_application_error());
        assert!(IpcErrorPhase::JsonRpcError(-32601).is_application_error());
        assert!(IpcErrorPhase::HttpStatus(400).is_application_error());
        assert!(IpcErrorPhase::HttpStatus(404).is_application_error());
        assert!(IpcErrorPhase::HttpStatus(422).is_application_error());

        assert!(!IpcErrorPhase::Connect.is_application_error());
        assert!(!IpcErrorPhase::Read.is_application_error());
        assert!(!IpcErrorPhase::HttpStatus(503).is_application_error());
    }
}
