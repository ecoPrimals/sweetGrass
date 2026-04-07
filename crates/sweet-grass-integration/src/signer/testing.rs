// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Test-only mock implementations for signing.
//!
//! These are isolated from production code and only available in test builds.
//!
//! ## Zero-Knowledge Architecture
//!
//! Mock implementations use capability-based naming (`MockSigningClient`)
//! rather than primal-specific names.

#![cfg(any(test, feature = "test"))]

use async_trait::async_trait;

use sweet_grass_core::Braid;
use sweet_grass_core::agent::Did;
use sweet_grass_core::dehydration::Witness;

use crate::Result;

use super::traits::{SignatureInfo, SigningClient};

/// Mock signing client for testing.
///
/// This provides a fully functional mock that can be configured
/// for different test scenarios.
pub struct MockSigningClient {
    did: Did,
    sign_result: Option<Witness>,
    verify_result: Option<bool>,
    healthy: bool,
}

impl MockSigningClient {
    /// Create a mock client with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            did: Did::new("did:key:z6MkTestSigner"),
            sign_result: None,
            verify_result: Some(true),
            healthy: true,
        }
    }

    /// Set the DID this mock will return.
    #[must_use]
    pub fn with_did(mut self, did: Did) -> Self {
        self.did = did;
        self
    }

    /// Configure what `sign()` returns.
    #[must_use]
    pub fn with_sign_result(mut self, witness: Witness) -> Self {
        self.sign_result = Some(witness);
        self
    }

    /// Configure what `verify()` returns.
    #[must_use]
    pub const fn with_verify_result(mut self, valid: bool) -> Self {
        self.verify_result = Some(valid);
        self
    }

    /// Set health status.
    #[must_use]
    pub const fn with_health(mut self, healthy: bool) -> Self {
        self.healthy = healthy;
        self
    }
}

impl Default for MockSigningClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SigningClient for MockSigningClient {
    async fn sign(&self, _braid: &Braid) -> Result<Witness> {
        self.sign_result.as_ref().map_or_else(
            || Ok(Witness::from_ed25519(&self.did, b"mock-signature-value")),
            |w| Ok(w.clone()),
        )
    }

    async fn verify(&self, _braid: &Braid) -> Result<SignatureInfo> {
        let valid = self.verify_result.unwrap_or(true);
        let now = chrono::Utc::now();

        Ok(SignatureInfo {
            signer: self.did.clone(),
            algorithm: "ed25519".to_string(),
            signed_at: u64::try_from(now.timestamp()).unwrap_or(0),
            valid,
        })
    }

    async fn current_did(&self) -> Result<Did> {
        Ok(self.did.clone())
    }

    async fn resolve_did(&self, did: &Did) -> Result<Option<serde_json::Value>> {
        // Return a mock DID document
        Ok(Some(serde_json::json!({
            "@context": ["https://www.w3.org/ns/did/v1"],
            "id": did.as_str(),
            "verificationMethod": [{
                "id": format!("{}#keys-1", did.as_str()),
                "type": "Ed25519VerificationKey2020",
                "controller": did.as_str(),
                "publicKeyMultibase": "z6MkTestKey"
            }]
        })))
    }

    async fn health(&self) -> Result<bool> {
        Ok(self.healthy)
    }
}
