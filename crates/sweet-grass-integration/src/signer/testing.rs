//! Test-only mock implementations for signing.
//!
//! These are isolated from production code and only available in test builds.
//!
//! ## Zero-Knowledge Architecture
//!
//! Mock implementations use capability-based naming (`MockSigningClient`)
//! rather than primal-specific names.

#![cfg(any(test, feature = "test-support"))]
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::cast_sign_loss,
    clippy::option_if_let_else
)]

use async_trait::async_trait;

use sweet_grass_core::agent::Did;
use sweet_grass_core::braid::BraidSignature;
use sweet_grass_core::Braid;

use crate::Result;

use super::traits::{SignatureInfo, SigningClient};

/// Mock signing client for testing.
///
/// This provides a fully functional mock that can be configured
/// for different test scenarios.
pub struct MockSigningClient {
    did: Did,
    sign_result: Option<BraidSignature>,
    verify_result: Option<bool>,
    healthy: bool,
}

/// Backward compatibility alias.
#[deprecated(
    since = "0.3.0",
    note = "Use MockSigningClient - capability-based naming"
)]
pub type MockBearDogClient = MockSigningClient;

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

    /// Configure what sign() returns.
    #[must_use]
    pub fn with_sign_result(mut self, signature: BraidSignature) -> Self {
        self.sign_result = Some(signature);
        self
    }

    /// Configure what verify() returns.
    #[must_use]
    pub fn with_verify_result(mut self, valid: bool) -> Self {
        self.verify_result = Some(valid);
        self
    }

    /// Set health status.
    #[must_use]
    pub fn with_health(mut self, healthy: bool) -> Self {
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
    async fn sign(&self, _braid: &Braid) -> Result<BraidSignature> {
        if let Some(sig) = &self.sign_result {
            Ok(sig.clone())
        } else {
            // Generate a deterministic mock signature
            let now = chrono::Utc::now();
            Ok(BraidSignature {
                sig_type: "Ed25519Signature2020".to_string(),
                created: now.timestamp() as u64,
                verification_method: format!("{}#keys-1", self.did.as_str()),
                proof_purpose: "assertionMethod".to_string(),
                proof_value: "mock-signature-value".to_string(),
            })
        }
    }

    async fn verify(&self, _braid: &Braid) -> Result<SignatureInfo> {
        let valid = self.verify_result.unwrap_or(true);
        let now = chrono::Utc::now();

        Ok(SignatureInfo {
            signer: self.did.clone(),
            algorithm: "Ed25519Signature2020".to_string(),
            signed_at: now.timestamp() as u64,
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
