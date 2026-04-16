// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
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

use std::future::Future;

use serde::{Deserialize, Serialize};

use sweet_grass_core::Braid;
use sweet_grass_core::agent::Did;
use sweet_grass_core::braid::Timestamp;
use sweet_grass_core::dehydration::Witness;

use crate::Result;

/// Default signature algorithm used by signing clients.
///
/// **Note:** This constant serves as a default only. The signing algorithm should
/// come from the signing primal at runtime (discovered, not assumed). This will
/// become config-driven in v0.8.0.
pub const SIGNING_ALGORITHM: &str = "ed25519";

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
/// Uses native `impl Future + Send` (Rust 2024). Runtime dispatch uses
/// [`crate::signer::SigningBackend`].
///
/// ## Discovery Pattern
///
/// ```rust,ignore
/// let discovery = create_discovery().await;
/// let primal = discovery.find_one(&Capability::Signing).await?;
/// let client = create_signing_client(&primal).await?;
/// ```
pub trait SigningClient: Send + Sync {
    /// Sign a Braid, returning a `Witness` (`WireWitnessRef`).
    fn sign(&self, braid: &Braid) -> impl Future<Output = Result<Witness>> + Send;

    /// Verify a Braid's signature.
    fn verify(&self, braid: &Braid) -> impl Future<Output = Result<SignatureInfo>> + Send;

    /// Get the current agent's DID.
    fn current_did(&self) -> impl Future<Output = Result<Did>> + Send;

    /// Resolve a DID to its document.
    fn resolve_did(
        &self,
        did: &Did,
    ) -> impl Future<Output = Result<Option<serde_json::Value>>> + Send;

    /// Check connection health.
    fn health(&self) -> impl Future<Output = Result<bool>> + Send;
}

/// Trait for signing Braids.
///
/// Uses native `impl Future + Send` (Rust 2024) instead of `#[async_trait]`
/// because this trait is never used as a trait object (`dyn Signer`).
pub trait Signer: Send + Sync {
    /// Sign a Braid, returning a signed copy.
    fn sign_braid(&self, braid: &Braid) -> impl Future<Output = Result<Braid>> + Send;

    /// Verify a Braid's signature.
    fn verify_braid(&self, braid: &Braid) -> impl Future<Output = Result<bool>> + Send;

    /// Get the signer's DID.
    fn signer_did(&self) -> &Did;
}
