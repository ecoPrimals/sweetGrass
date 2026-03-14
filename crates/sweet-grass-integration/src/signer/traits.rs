// SPDX-License-Identifier: AGPL-3.0-only
//! Core signing traits.
//!
//! These traits abstract the signing interface allowing for different
//! backend implementations (tarpc, REST, mocks for testing).
//!
//! ## Zero-Knowledge Architecture
//!
//! This module uses capability-based naming:
//! - `SigningClient` - connects to any primal offering `Capability::Signing`
//! - No hardcoded primal names, ports, or addresses
//! - Runtime discovery via the universal adapter

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use sweet_grass_core::agent::Did;
use sweet_grass_core::braid::{BraidSignature, Timestamp};
use sweet_grass_core::Braid;

use crate::Result;

/// Default signature algorithm used by signing clients.
pub const SIGNING_ALGORITHM: &str = "Ed25519Signature2020";

/// Information about a signature.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignatureInfo {
    /// The signer's DID.
    pub signer: Did,

    /// Signature algorithm.
    pub algorithm: String,

    /// When the signature was created.
    pub signed_at: Timestamp,

    /// Whether the signature is valid.
    pub valid: bool,
}

/// Trait for signing capability client connections.
///
/// This trait abstracts the signing implementation, allowing connection
/// to any primal that offers the `Capability::Signing` capability.
///
/// ## Discovery Pattern
///
/// ```rust,ignore
/// let discovery = create_discovery().await;
/// let primal = discovery.find_one(&Capability::Signing).await?;
/// let client = create_signing_client(&primal).await?;
/// ```
#[async_trait]
pub trait SigningClient: Send + Sync {
    /// Sign a Braid.
    async fn sign(&self, braid: &Braid) -> Result<BraidSignature>;

    /// Verify a Braid's signature.
    async fn verify(&self, braid: &Braid) -> Result<SignatureInfo>;

    /// Get the current agent's DID.
    async fn current_did(&self) -> Result<Did>;

    /// Resolve a DID to its document.
    async fn resolve_did(&self, did: &Did) -> Result<Option<serde_json::Value>>;

    /// Check connection health.
    async fn health(&self) -> Result<bool>;
}

/// Trait for signing Braids.
#[async_trait]
pub trait Signer: Send + Sync {
    /// Sign a Braid, returning a signed copy.
    async fn sign_braid(&self, braid: &Braid) -> Result<Braid>;

    /// Verify a Braid's signature.
    async fn verify_braid(&self, braid: &Braid) -> Result<bool>;

    /// Get the signer's DID.
    fn signer_did(&self) -> &Did;
}
