// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "test module: expect/unwrap are standard in tests"
)]

use super::*;
use sweet_grass_core::agent::Did;
use sweet_grass_core::braid::BraidBuilder;
use sweet_grass_store::{BraidStore, MemoryStore};

fn create_test_braid() -> Braid {
    BraidBuilder::default()
        .data_hash("sha256:test123")
        .mime_type("text/plain")
        .size(100)
        .attributed_to(Did::new("did:key:z6MkTest"))
        .build()
        .expect("build braid")
}

fn create_test_braid_with_hash(hash: &str) -> Braid {
    BraidBuilder::default()
        .data_hash(hash)
        .mime_type("text/plain")
        .size(100)
        .attributed_to(Did::new("did:key:z6MkTest"))
        .build()
        .expect("build braid")
}

#[tokio::test]
async fn test_mock_client_anchor() {
    let client = MockAnchoringClient::new();
    let braid = create_test_braid();

    let receipt = client.anchor(&braid, "spine-1").await.expect("anchor");
    assert_eq!(receipt.anchor.spine_id, "spine-1");
    assert!(receipt.transaction_id.is_some());
}

#[tokio::test]
async fn test_mock_client_verify() {
    let client = MockAnchoringClient::new();
    let braid = create_test_braid();

    client.anchor(&braid, "spine-1").await.expect("anchor");

    let info = client.verify(&braid.id).await.expect("verify");
    assert!(info.is_some());
    assert_eq!(info.unwrap().braid_id, braid.id);
}

#[tokio::test]
async fn test_mock_client_health() {
    let client = MockAnchoringClient::new();
    assert!(client.health().await.expect("health"));

    let unhealthy = MockAnchoringClient::new().with_health(false);
    assert!(!unhealthy.health().await.expect("health"));
}

#[tokio::test]
async fn test_mock_client_get_anchors() {
    let client = MockAnchoringClient::new();
    let braid = create_test_braid();

    let anchors = client.get_anchors(&braid.id).await.expect("get");
    assert!(anchors.is_empty());

    client.anchor(&braid, "spine-1").await.expect("anchor");

    let anchors = client.get_anchors(&braid.id).await.expect("get");
    assert_eq!(anchors.len(), 1);
    assert_eq!(anchors[0].spine_id, "spine-1");
}

#[tokio::test]
async fn test_mock_client_verify_not_found() {
    let client = MockAnchoringClient::new();
    let braid = create_test_braid();

    let info = client.verify(&braid.id).await.expect("verify");
    assert!(info.is_none());
}

#[tokio::test]
async fn test_anchor_manager_with_client() {
    let client: Arc<AnchoringBackend> =
        Arc::new(AnchoringBackend::Mock(MockAnchoringClient::new()));
    let store = Arc::new(MemoryStore::new());

    let manager = AnchorManager::with_client(client, store);
    let client_ref = manager.client();
    assert!(client_ref.health().await.expect("health"));
}

#[tokio::test]
async fn test_anchor_manager_anchor_by_id() {
    let client: Arc<AnchoringBackend> =
        Arc::new(AnchoringBackend::Mock(MockAnchoringClient::new()));
    let store = Arc::new(MemoryStore::new());
    let braid = create_test_braid_with_hash("sha256:anchor_test");

    store.put(&braid).await.expect("store");

    let manager = AnchorManager::with_client(client, store);
    let receipt = manager
        .anchor_by_id(&braid.id, "test-spine")
        .await
        .expect("anchor");
    assert_eq!(receipt.anchor.spine_id, "test-spine");
}

#[tokio::test]
async fn test_anchor_manager_anchor_by_id_not_found() {
    let client: Arc<AnchoringBackend> =
        Arc::new(AnchoringBackend::Mock(MockAnchoringClient::new()));
    let store = Arc::new(MemoryStore::new());
    let braid = create_test_braid();

    let manager = AnchorManager::with_client(client, store);
    let result = manager.anchor_by_id(&braid.id, "test-spine").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn test_anchor_manager_verify() {
    let client: Arc<AnchoringBackend> =
        Arc::new(AnchoringBackend::Mock(MockAnchoringClient::new()));
    let store = Arc::new(MemoryStore::new());
    let braid = create_test_braid_with_hash("sha256:verify_test");

    store.put(&braid).await.expect("store");

    let manager = AnchorManager::with_client(Arc::clone(&client), store);

    client.anchor(&braid, "spine-1").await.expect("anchor");

    let info = manager.verify(&braid.id).await.expect("verify");
    assert!(info.is_some());
}

#[tokio::test]
async fn test_anchor_manager_get_anchors() {
    let client: Arc<AnchoringBackend> =
        Arc::new(AnchoringBackend::Mock(MockAnchoringClient::new()));
    let store = Arc::new(MemoryStore::new());
    let braid = create_test_braid_with_hash("sha256:get_anchors_test");

    store.put(&braid).await.expect("store");

    let manager = AnchorManager::with_client(Arc::clone(&client), store);

    client.anchor(&braid, "spine-1").await.expect("anchor");
    client.anchor(&braid, "spine-2").await.expect("anchor");

    let anchors = manager.get_anchors(&braid.id).await.expect("get");
    assert_eq!(anchors.len(), 2);
}

#[tokio::test]
async fn test_anchor_receipt_structure() {
    let client = MockAnchoringClient::new();
    let braid = create_test_braid_with_hash("sha256:receipt_test");

    let receipt = client.anchor(&braid, "test-spine").await.expect("anchor");

    assert_eq!(receipt.anchor.braid_id, braid.id);
    assert_eq!(receipt.anchor.spine_id, "test-spine");
    assert!(receipt.anchor.entry_hash.starts_with("entry:"));
    assert!(!receipt.anchor.verified);
    assert!(receipt.confirmations >= 1);
}

#[tokio::test]
async fn test_anchor_info_structure() {
    let client = MockAnchoringClient::new();
    let braid = create_test_braid_with_hash("sha256:info_test");

    client.anchor(&braid, "spine-test").await.expect("anchor");

    let info = client.verify(&braid.id).await.expect("verify").unwrap();

    assert_eq!(info.braid_id, braid.id);
    assert_eq!(info.spine_id, "spine-test");
    assert!(info.anchored_at > 0);
}

#[tokio::test]
async fn test_create_anchoring_client_requires_real_server() {
    use crate::discovery::DiscoveredPrimal;
    use sweet_grass_core::config::Capability;

    let primal = DiscoveredPrimal {
        instance_id: "anchor-1".to_string(),
        name: "TestAnchoringService".to_string(),
        capabilities: vec![Capability::Anchoring],
        tarpc_address: Some("127.0.0.1:1".to_string()),
        rest_address: None,
        last_seen: std::time::SystemTime::now(),
        healthy: true,
    };

    let result = create_anchoring_client_async(&primal).await;
    assert!(result.is_err(), "should fail without a real tarpc server");
}

#[tokio::test]
async fn test_mock_anchoring_client_directly() {
    let client: Arc<AnchoringBackend> =
        Arc::new(AnchoringBackend::Mock(MockAnchoringClient::new()));
    assert!(client.health().await.expect("health"));
}

#[tokio::test]
async fn test_anchor_manager_new_discovery_failure() {
    use crate::discovery::{DiscoveryBackend, LocalDiscovery};

    let discovery: Arc<DiscoveryBackend> = Arc::new(DiscoveryBackend::Local(LocalDiscovery::new()));
    let store = Arc::new(MemoryStore::new());

    let result = AnchorManager::new(discovery, store, |_| {
        Arc::new(AnchoringBackend::Mock(MockAnchoringClient::new()))
    })
    .await;

    let err = result.err().expect("should fail");
    assert!(
        err.to_string().to_lowercase().contains("capability"),
        "error should mention capability: {err}"
    );
}

#[tokio::test]
async fn test_anchor_manager_new_with_discovery_success() {
    use crate::discovery::{DiscoveredPrimal, DiscoveryBackend, LocalDiscovery};
    use sweet_grass_core::config::Capability;

    let local = LocalDiscovery::new();
    local
        .register(DiscoveredPrimal {
            instance_id: "test-anchor-primal-1".to_owned(),
            name: "test-anchoring".to_string(),
            capabilities: vec![Capability::Anchoring],
            tarpc_address: Some("localhost:0".to_string()),
            rest_address: None,
            last_seen: std::time::SystemTime::now(),
            healthy: true,
        })
        .await;

    let discovery = Arc::new(DiscoveryBackend::Local(local));

    let store = Arc::new(MemoryStore::new());

    let manager = AnchorManager::new(discovery, store, |primal| {
        assert_eq!(primal.name, "test-anchoring");
        Arc::new(AnchoringBackend::Mock(MockAnchoringClient::new()))
    })
    .await
    .expect("discovery should succeed");

    assert!(manager.client().health().await.expect("health"));
}

#[tokio::test]
async fn test_anchor_manager_reconnect_success() {
    use crate::discovery::{DiscoveredPrimal, DiscoveryBackend, LocalDiscovery};
    use sweet_grass_core::config::Capability;

    let local = LocalDiscovery::new();
    local
        .register(DiscoveredPrimal {
            instance_id: "anchor-primary".to_string(),
            name: "primary-anchor".to_string(),
            capabilities: vec![Capability::Anchoring],
            tarpc_address: Some("localhost:0".to_string()),
            rest_address: None,
            last_seen: std::time::SystemTime::now(),
            healthy: true,
        })
        .await;

    let discovery = Arc::new(DiscoveryBackend::Local(local));

    let store = Arc::new(MemoryStore::new());

    let manager = AnchorManager::new(Arc::clone(&discovery), store, |_| {
        Arc::new(AnchoringBackend::Mock(MockAnchoringClient::new()))
    })
    .await
    .expect("initial");

    let result = manager
        .reconnect(|primal| {
            assert_eq!(primal.name, "primary-anchor");
            Arc::new(AnchoringBackend::Mock(
                MockAnchoringClient::new().with_health(true),
            ))
        })
        .await;
    assert!(result.is_ok());
    assert!(manager.client().health().await.expect("health"));
}

#[tokio::test]
async fn test_anchor_manager_reconnect_failure_no_primal() {
    let client: Arc<AnchoringBackend> =
        Arc::new(AnchoringBackend::Mock(MockAnchoringClient::new()));
    let store = Arc::new(MemoryStore::new());

    let manager = AnchorManager::with_client(client, store);

    let result = manager
        .reconnect(|_| Arc::new(AnchoringBackend::Mock(MockAnchoringClient::new())))
        .await;

    assert!(result.is_err(), "empty discovery should fail reconnect");
}

#[tokio::test]
async fn test_anchor_manager_multiple_operations() {
    let client: Arc<AnchoringBackend> =
        Arc::new(AnchoringBackend::Mock(MockAnchoringClient::new()));
    let store = Arc::new(MemoryStore::new());

    let braid1 = create_test_braid_with_hash("sha256:multi_op_1");
    let braid2 = create_test_braid_with_hash("sha256:multi_op_2");
    store.put(&braid1).await.expect("put 1");
    store.put(&braid2).await.expect("put 2");

    let manager = AnchorManager::with_client(Arc::clone(&client), store);

    let r1 = manager
        .anchor_by_id(&braid1.id, "spine-a")
        .await
        .expect("anchor 1");
    let r2 = manager
        .anchor_by_id(&braid2.id, "spine-b")
        .await
        .expect("anchor 2");

    assert_eq!(r1.anchor.spine_id, "spine-a");
    assert_eq!(r2.anchor.spine_id, "spine-b");

    let anchors1 = manager.get_anchors(&braid1.id).await.expect("get 1");
    assert_eq!(anchors1.len(), 1);

    let verify1 = manager.verify(&braid1.id).await.expect("verify 1");
    assert!(verify1.is_some());

    let verify_none = manager
        .verify(&create_test_braid().id)
        .await
        .expect("verify none");
    assert!(verify_none.is_none());
}

#[tokio::test]
async fn test_anchor_info_serialization() {
    let info = AnchorInfo {
        braid_id: BraidId::from_string("urn:braid:test-123"),
        spine_id: "spine-1".to_string(),
        entry_hash: "entry:sha256:abc".to_string(),
        index: 42,
        anchored_at: 1_710_000_000,
        verified: true,
    };

    let json = serde_json::to_string(&info).expect("serialize");
    let parsed: AnchorInfo = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(parsed.spine_id, "spine-1");
    assert_eq!(parsed.index, 42);
    assert!(parsed.verified);
}

#[tokio::test]
async fn test_anchor_receipt_serialization() {
    let receipt = AnchorReceipt {
        anchor: AnchorInfo {
            braid_id: BraidId::from_string("urn:braid:test-ser"),
            spine_id: "spine-ser".to_string(),
            entry_hash: "entry:test".to_string(),
            index: 0,
            anchored_at: 1_710_000_000,
            verified: false,
        },
        transaction_id: Some("tx-001".to_string()),
        confirmations: 3,
    };

    let json = serde_json::to_string(&receipt).expect("serialize");
    let parsed: AnchorReceipt = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(parsed.confirmations, 3);
    assert_eq!(parsed.transaction_id, Some("tx-001".to_string()));
}

#[tokio::test]
async fn test_from_primal_missing_tarpc_address() {
    use crate::discovery::DiscoveredPrimal;
    use sweet_grass_core::config::Capability;

    let primal = DiscoveredPrimal {
        instance_id: "no-addr".to_string(),
        name: "no-address-anchor".to_string(),
        capabilities: vec![Capability::Anchoring],
        tarpc_address: None,
        rest_address: None,
        last_seen: std::time::SystemTime::now(),
        healthy: true,
    };

    let result = super::TarpcAnchoringClient::from_primal(&primal).await;
    assert!(result.is_err());
    let err = result.err().expect("should be error");
    assert!(
        err.to_string().contains("no tarpc address"),
        "should return MissingTarpcAddress, got: {err}"
    );
}
