// SPDX-License-Identifier: AGPL-3.0-only
//! Signing capability integration.
//!
//! Provides traits for signing Braids via capability-based discovery.
//! SweetGrass discovers signing primals at runtime rather than using
//! hardcoded connections.
//!
//! ## Zero-Knowledge Architecture
//!
//! - Uses `Capability::Signing` for discovery
//! - No hardcoded primal names, ports, or addresses
//! - Runtime discovery via the universal adapter
//!
//! ## Usage
//!
//! ```rust,ignore
//! use sweet_grass_integration::{DiscoverySigner, create_discovery, Capability};
//!
//! let discovery = create_discovery().await;
//! let primal = discovery.find_one(&Capability::Signing).await?;
//! let signer = DiscoverySigner::with_client(
//!     create_signing_client_async(&primal).await?
//! ).await?;
//! let signed_braid = signer.sign_braid(&braid).await?;
//! ```

mod discovery;
mod tarpc_client;
mod traits;

#[cfg(any(test, feature = "test-support"))]
pub mod testing;

// Re-export core traits (capability-based naming)
pub use traits::{SignatureInfo, Signer, SigningClient, SIGNING_ALGORITHM};

// Re-export signers
pub use discovery::{DiscoverySigner, LegacySigner};

// Re-export tarpc client and factories (capability-based naming only - v0.5.0+)
pub use tarpc_client::{
    create_signing_client_async, SigningRpc, SigningRpcClient, TarpcSigningClient,
};

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::discovery::{DiscoveredPrimal, LocalDiscovery};
    use std::sync::Arc;
    use sweet_grass_core::config::Capability;

    #[tokio::test]
    async fn test_mock_client_sign() {
        let client = testing::MockSigningClient::new();
        let braid = sweet_grass_core::Braid::builder()
            .data_hash("sha256:test")
            .mime_type("text/plain")
            .size(100)
            .attributed_to(sweet_grass_core::agent::Did::new("did:key:z6MkTest"))
            .build()
            .expect("build braid");

        let signature = client.sign(&braid).await.expect("sign");
        assert!(signature.verification_method.contains("keys-1"));
    }

    #[tokio::test]
    async fn test_mock_client_verify() {
        let client = testing::MockSigningClient::new();
        let braid = sweet_grass_core::Braid::builder()
            .data_hash("sha256:test")
            .mime_type("text/plain")
            .size(100)
            .attributed_to(sweet_grass_core::agent::Did::new("did:key:z6MkTest"))
            .build()
            .expect("build braid");

        let info = client.verify(&braid).await.expect("verify");
        assert!(info.valid);
    }

    #[tokio::test]
    async fn test_mock_client_configurable_did() {
        let custom_did = sweet_grass_core::agent::Did::new("did:key:z6MkCustom");
        let client = testing::MockSigningClient::new().with_did(custom_did.clone());
        let did = client.current_did().await.expect("did");
        assert_eq!(did, custom_did);
    }

    #[tokio::test]
    async fn test_mock_client_resolve_did() {
        let client = testing::MockSigningClient::new();
        let did = sweet_grass_core::agent::Did::new("did:key:z6MkTest");
        let doc = client.resolve_did(&did).await.expect("resolve");
        assert!(doc.is_some());
        let doc = doc.unwrap();
        assert_eq!(doc["id"], did.as_str());
    }

    #[tokio::test]
    async fn test_discovery_signer_creation() {
        let discovery = Arc::new(LocalDiscovery::new());

        // Register a mock signing primal (no hardcoded name needed)
        let mock_primal = DiscoveredPrimal {
            instance_id: "signing-service-001".to_string(),
            name: "signing-service".to_string(),
            capabilities: vec![Capability::Signing],
            tarpc_address: Some("discovered-address:0".to_string()), // :0 = mock address
            rest_address: None,
            last_seen: std::time::SystemTime::now(),
            healthy: true,
        };
        discovery.register(mock_primal).await;

        // Create signer with mock client factory
        let signer = DiscoverySigner::new(discovery, |_primal| {
            Arc::new(testing::MockSigningClient::new()) as Arc<dyn SigningClient>
        })
        .await
        .expect("create signer");

        assert_eq!(signer.signer_did().as_str(), "did:key:z6MkTestSigner");
    }

    #[tokio::test]
    #[allow(clippy::similar_names)]
    async fn test_legacy_signer_sign() {
        let client = Arc::new(testing::MockSigningClient::new());
        let signer = LegacySigner::new(client).await.expect("create signer");

        let braid = sweet_grass_core::Braid::builder()
            .data_hash("sha256:test")
            .mime_type("text/plain")
            .size(100)
            .attributed_to(sweet_grass_core::agent::Did::new("did:key:z6MkTest"))
            .build()
            .expect("build braid");

        let signed_braid = signer.sign_braid(&braid).await.expect("sign");
        assert!(!signed_braid.signature.proof_value.is_empty());
    }

    #[tokio::test]
    async fn test_legacy_signer_verify() {
        let client = Arc::new(testing::MockSigningClient::new());
        let signer = LegacySigner::new(client).await.expect("create signer");

        let braid = sweet_grass_core::Braid::builder()
            .data_hash("sha256:test")
            .mime_type("text/plain")
            .size(100)
            .attributed_to(sweet_grass_core::agent::Did::new("did:key:z6MkTest"))
            .build()
            .expect("build braid");

        let valid = signer.verify_braid(&braid).await.expect("verify");
        assert!(valid);
    }

    #[tokio::test]
    async fn test_mock_client_invalid_verify() {
        let client = Arc::new(testing::MockSigningClient::new().with_verify_result(false));
        let signer = LegacySigner::new(client).await.expect("create signer");

        let braid = sweet_grass_core::Braid::builder()
            .data_hash("sha256:test")
            .mime_type("text/plain")
            .size(100)
            .attributed_to(sweet_grass_core::agent::Did::new("did:key:z6MkTest"))
            .build()
            .expect("build braid");

        let valid = signer.verify_braid(&braid).await.expect("verify");
        assert!(!valid);
    }

    #[tokio::test]
    async fn test_custom_did_signer() {
        let custom_did = sweet_grass_core::agent::Did::new("did:key:z6MkCustomSigner");
        let client = Arc::new(testing::MockSigningClient::new().with_did(custom_did.clone()));
        let signer = LegacySigner::new(client).await.expect("create signer");

        assert_eq!(signer.signer_did(), &custom_did);
    }

    #[tokio::test]
    async fn test_create_client_async() {
        let primal = DiscoveredPrimal {
            instance_id: "signing-001".to_string(),
            name: "signing-service".to_string(),
            capabilities: vec![Capability::Signing],
            tarpc_address: Some("discovered:0".to_string()), // :0 = mock address
            rest_address: None,
            last_seen: std::time::SystemTime::now(),
            healthy: true,
        };

        // In test mode, returns mock
        let client = create_signing_client_async(&primal)
            .await
            .expect("create client");
        assert!(client.health().await.expect("health"));
    }

    #[tokio::test]
    async fn test_create_client_sync_mock_directly() {
        // In test mode, we can use mock clients directly
        let client = testing::MockSigningClient::new();
        assert!(client.health().await.expect("health"));
    }

    #[tokio::test]
    async fn test_mock_client_health() {
        let client = testing::MockSigningClient::new();
        assert!(client.health().await.expect("health"));

        let unhealthy = testing::MockSigningClient::new().with_health(false);
        assert!(!unhealthy.health().await.expect("health"));
    }

    #[tokio::test]
    async fn test_mock_client_custom_signature() {
        let custom_sig = sweet_grass_core::braid::BraidSignature {
            sig_type: "CustomType".to_string(),
            created: 12345,
            verification_method: "did:key:test#custom".to_string(),
            proof_purpose: "assertionMethod".to_string(),
            proof_value: "custom-proof".to_string(),
        };

        let client = testing::MockSigningClient::new().with_sign_result(custom_sig.clone());

        let braid = sweet_grass_core::Braid::builder()
            .data_hash("sha256:test")
            .mime_type("text/plain")
            .size(100)
            .attributed_to(sweet_grass_core::agent::Did::new("did:key:z6MkTest"))
            .build()
            .expect("build braid");

        let sig = client.sign(&braid).await.expect("sign");
        assert_eq!(sig.sig_type, "CustomType");
        assert_eq!(sig.proof_value, "custom-proof");
    }

    #[tokio::test]
    async fn test_discovery_signer_no_signer_available() {
        let discovery = Arc::new(LocalDiscovery::new());
        // No primals registered - discovery will fail

        let result = DiscoverySigner::new(discovery, |_primal| {
            Arc::new(testing::MockSigningClient::new()) as Arc<dyn SigningClient>
        })
        .await;

        assert!(result.is_err());
        let err_msg = result.err().unwrap().to_string();
        assert!(
            err_msg.to_lowercase().contains("discovery"),
            "expected discovery error, got: {err_msg}"
        );
    }

    #[tokio::test]
    async fn test_discovery_signer_multiple_signers() {
        let discovery = Arc::new(LocalDiscovery::new());

        let primal1 = DiscoveredPrimal {
            instance_id: "signer-1".to_string(),
            name: "signer-one".to_string(),
            capabilities: vec![Capability::Signing],
            tarpc_address: Some("localhost:0".to_string()),
            rest_address: None,
            last_seen: std::time::SystemTime::now(),
            healthy: true,
        };
        let primal2 = DiscoveredPrimal {
            instance_id: "signer-2".to_string(),
            name: "signer-two".to_string(),
            capabilities: vec![Capability::Signing],
            tarpc_address: Some("localhost:0".to_string()),
            rest_address: None,
            last_seen: std::time::SystemTime::now(),
            healthy: true,
        };
        discovery.register(primal1).await;
        discovery.register(primal2).await;

        let signer = DiscoverySigner::new(discovery, |primal| {
            Arc::new(
                testing::MockSigningClient::new().with_did(sweet_grass_core::agent::Did::new(
                    format!("did:key:z6Mk{}", primal.name.replace('-', "")),
                )),
            ) as Arc<dyn SigningClient>
        })
        .await
        .expect("create signer");

        // Should get one of the signers (first healthy one)
        assert!(signer.signer_did().as_str().starts_with("did:key:z6Mk"));
    }

    #[tokio::test]
    async fn test_discovery_signer_with_client() {
        let client = Arc::new(testing::MockSigningClient::new());
        let signer = DiscoverySigner::with_client(client).await.expect("create");

        assert_eq!(signer.signer_did().as_str(), "did:key:z6MkTestSigner");
    }

    #[tokio::test]
    async fn test_discovery_signer_reconnect() {
        let discovery = Arc::new(LocalDiscovery::new());
        let primal = DiscoveredPrimal {
            instance_id: "signer-reconnect".to_string(),
            name: "reconnect-service".to_string(),
            capabilities: vec![Capability::Signing],
            tarpc_address: Some("localhost:0".to_string()),
            rest_address: None,
            last_seen: std::time::SystemTime::now(),
            healthy: true,
        };
        discovery.register(primal).await;

        let disc: Arc<dyn crate::PrimalDiscovery> = discovery;
        let mut signer = DiscoverySigner::new(Arc::clone(&disc), |_primal| {
            Arc::new(testing::MockSigningClient::new()) as Arc<dyn SigningClient>
        })
        .await
        .expect("create signer");

        let new_did = sweet_grass_core::agent::Did::new("did:key:z6MkReconnected");
        signer
            .reconnect(|_primal| {
                Arc::new(testing::MockSigningClient::new().with_did(new_did.clone()))
                    as Arc<dyn SigningClient>
            })
            .await
            .expect("reconnect");

        assert_eq!(signer.signer_did(), &new_did);
    }

    #[tokio::test]
    async fn test_discovery_signer_client_accessor() {
        let client = Arc::new(testing::MockSigningClient::new());
        let signer = DiscoverySigner::with_client(client).await.expect("create");

        let client_ref = signer.client();
        assert!(client_ref.health().await.expect("health"));
    }
}
