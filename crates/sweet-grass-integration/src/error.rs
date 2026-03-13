// SPDX-License-Identifier: AGPL-3.0-only
//! Error types for integration operations.

use thiserror::Error;

/// Integration error types.
#[derive(Debug, Error)]
pub enum IntegrationError {
    /// Discovery failed - no primal found offering required capability.
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
