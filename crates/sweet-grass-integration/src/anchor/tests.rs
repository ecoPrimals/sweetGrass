// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "test module: expect/unwrap are standard in tests"
)]

use super::*;
use sweet_grass_core::agent::Did;
use sweet_grass_core::braid::BraidBuilder;
use sweet_grass_store::MemoryStore;

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
    let client: Arc<dyn AnchoringClient> = Arc::new(MockAnchoringClient::new());
    let store: Arc<dyn sweet_grass_store::BraidStore> = Arc::new(MemoryStore::new());

    let manager = AnchorManager::with_client(client, store);
    let client_ref = manager.client();
    assert!(client_ref.health().await.expect("health"));
}

#[tokio::test]
async fn test_anchor_manager_anchor_by_id() {
    let client: Arc<dyn AnchoringClient> = Arc::new(MockAnchoringClient::new());
    let store: Arc<dyn sweet_grass_store::BraidStore> = Arc::new(MemoryStore::new());
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
    let client: Arc<dyn AnchoringClient> = Arc::new(MockAnchoringClient::new());
    let store: Arc<dyn sweet_grass_store::BraidStore> = Arc::new(MemoryStore::new());
    let braid = create_test_braid();

    let manager = AnchorManager::with_client(client, store);
    let result = manager.anchor_by_id(&braid.id, "test-spine").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn test_anchor_manager_verify() {
    let client: Arc<dyn AnchoringClient> = Arc::new(MockAnchoringClient::new());
    let store: Arc<dyn sweet_grass_store::BraidStore> = Arc::new(MemoryStore::new());
    let braid = create_test_braid_with_hash("sha256:verify_test");

    store.put(&braid).await.expect("store");

    let manager = AnchorManager::with_client(Arc::clone(&client), store);

    client.anchor(&braid, "spine-1").await.expect("anchor");

    let info = manager.verify(&braid.id).await.expect("verify");
    assert!(info.is_some());
}

#[tokio::test]
async fn test_anchor_manager_get_anchors() {
    let client: Arc<dyn AnchoringClient> = Arc::new(MockAnchoringClient::new());
    let store: Arc<dyn sweet_grass_store::BraidStore> = Arc::new(MemoryStore::new());
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
async fn test_create_anchoring_client_async() {
    use crate::discovery::DiscoveredPrimal;
    use sweet_grass_core::config::Capability;

    let test_address = std::env::var("TEST_ANCHORING_ADDR")
        .unwrap_or_else(|_| format!("localhost:{}", crate::testing::allocate_test_port()));

    let primal = DiscoveredPrimal {
        instance_id: "anchor-1".to_string(),
        name: "TestAnchoringService".to_string(),
        capabilities: vec![Capability::Anchoring],
        tarpc_address: Some(test_address),
        rest_address: None,
        last_seen: std::time::SystemTime::now(),
        healthy: true,
    };

    let client = create_anchoring_client_async(&primal)
        .await
        .expect("create client");
    assert!(client.health().await.expect("health"));
}

#[tokio::test]
async fn test_anchor_manager_new_discovery_failure() {
    use crate::discovery::{LocalDiscovery, PrimalDiscovery};

    let discovery: Arc<dyn PrimalDiscovery> = Arc::new(LocalDiscovery::new());
    let store: Arc<dyn sweet_grass_store::BraidStore> = Arc::new(MemoryStore::new());

    let result = AnchorManager::new(discovery, store, |_| {
        Arc::new(MockAnchoringClient::new()) as Arc<dyn AnchoringClient>
    })
    .await;

    let err = result.err().expect("should fail");
    assert!(
        err.to_string().to_lowercase().contains("capability"),
        "error should mention capability: {err}"
    );
}
