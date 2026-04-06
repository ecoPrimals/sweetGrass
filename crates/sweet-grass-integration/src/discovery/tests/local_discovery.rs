// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use super::*;

#[tokio::test]
async fn test_local_discovery_register_and_find() {
    let discovery = LocalDiscovery::new();

    let signer = make_test_primal("signer", vec![Capability::Signing]);
    let anchor = make_test_primal("anchor", vec![Capability::Anchoring]);

    discovery.register(signer).await;
    discovery.register(anchor).await;

    let signers = discovery
        .find_by_capability(&Capability::Signing)
        .await
        .expect("find");
    assert_eq!(signers.len(), 1);
    assert_eq!(signers[0].name, "signer");

    let anchors = discovery
        .find_by_capability(&Capability::Anchoring)
        .await
        .expect("find");
    assert_eq!(anchors.len(), 1);
    assert_eq!(anchors[0].name, "anchor");
}

#[tokio::test]
async fn test_local_discovery_find_one() {
    let discovery = LocalDiscovery::new();
    let signer = make_test_primal("signer", vec![Capability::Signing]);
    discovery.register(signer).await;

    let found = discovery
        .find_one(&Capability::Signing)
        .await
        .expect("find");
    assert_eq!(found.name, "signer");

    let not_found = discovery.find_one(&Capability::Compute).await;
    assert!(not_found.is_err());
}

#[tokio::test]
async fn test_multiple_primals_same_capability() {
    let discovery = LocalDiscovery::new();
    discovery
        .register(make_test_primal("signer1", vec![Capability::Signing]))
        .await;
    discovery
        .register(make_test_primal("signer2", vec![Capability::Signing]))
        .await;

    let signers = discovery
        .find_by_capability(&Capability::Signing)
        .await
        .expect("find");
    assert_eq!(signers.len(), 2);
}

#[tokio::test]
async fn test_unregister() {
    let discovery = LocalDiscovery::new();
    let primal = make_test_primal("signer", vec![Capability::Signing]);
    let id = primal.instance_id.clone();

    discovery.register(primal).await;
    assert_eq!(
        discovery
            .find_by_capability(&Capability::Signing)
            .await
            .expect("find")
            .len(),
        1
    );

    discovery.unregister(&id).await;
    assert_eq!(
        discovery
            .find_by_capability(&Capability::Signing)
            .await
            .expect("find")
            .len(),
        0
    );
}

#[tokio::test]
async fn test_local_discovery_health() {
    let discovery = LocalDiscovery::new();
    assert!(discovery.health().await);
}

#[tokio::test]
async fn test_discovery_update_primal() {
    let discovery = LocalDiscovery::new();

    // Register initial version
    let mut primal = make_test_primal("updatable", vec![Capability::Signing]);
    discovery.register(primal.clone()).await;

    // Re-register with additional capability
    primal.capabilities.push(Capability::Anchoring);
    discovery.register(primal).await;

    // Should have updated capabilities (still only one primal)
    let signers = discovery
        .find_by_capability(&Capability::Signing)
        .await
        .expect("find");
    assert_eq!(signers.len(), 1);

    let anchors = discovery
        .find_by_capability(&Capability::Anchoring)
        .await
        .expect("find");
    assert_eq!(anchors.len(), 1);
}

#[tokio::test]
async fn test_discovery_custom_capability() {
    let discovery = LocalDiscovery::new();
    let custom_cap = Capability::custom("my-custom-feature");
    let primal = make_test_primal("custom", vec![custom_cap.clone()]);
    discovery.register(primal).await;

    let found = discovery
        .find_by_capability(&custom_cap)
        .await
        .expect("find");
    assert_eq!(found.len(), 1);
    assert!(found[0].offers(&custom_cap));
}

#[tokio::test]
async fn test_concurrent_discovery_operations() {
    let discovery = Arc::new(LocalDiscovery::new());
    let mut handles = vec![];

    // Spawn multiple concurrent registrations
    for i in 0..10 {
        let d = Arc::clone(&discovery);
        let handle = tokio::spawn(async move {
            let primal = DiscoveredPrimal {
                instance_id: format!("primal-{i}"),
                name: format!("primal-{i}"),
                capabilities: vec![Capability::Signing],
                tarpc_address: Some(format!(
                    "localhost:{}",
                    crate::testing::allocate_test_port()
                )),
                rest_address: None,
                last_seen: std::time::SystemTime::now(),
                healthy: true,
            };
            d.register(primal).await;
        });
        handles.push(handle);
    }

    // Wait for all registrations
    for handle in handles {
        handle.await.expect("join");
    }

    // All should be registered
    let found = discovery
        .find_by_capability(&Capability::Signing)
        .await
        .expect("find");
    assert_eq!(found.len(), 10);
}

#[tokio::test]
async fn test_find_by_capability_none_exist() {
    let discovery = LocalDiscovery::new();

    let result = discovery
        .find_by_capability(&Capability::Signing)
        .await
        .expect("find");
    assert!(result.is_empty());
}

#[tokio::test]
async fn test_find_one_health_filtering() {
    let discovery = LocalDiscovery::new();

    // Register only unhealthy primal
    let mut unhealthy = make_test_primal("unhealthy-signer", vec![Capability::Signing]);
    unhealthy.healthy = false;
    discovery.register(unhealthy).await;

    // find_one should fail - no healthy primals
    let result = discovery.find_one(&Capability::Signing).await;
    assert!(result.is_err());

    // Add healthy primal
    let healthy = make_test_primal("healthy-signer", vec![Capability::Signing]);
    discovery.register(healthy).await;

    // find_one should now succeed with healthy one
    let found = discovery
        .find_one(&Capability::Signing)
        .await
        .expect("find");
    assert!(found.healthy);
    assert_eq!(found.name, "healthy-signer");
}

#[tokio::test]
async fn test_announce() {
    let discovery = LocalDiscovery::new();
    let primal = make_test_primal("announced", vec![Capability::Signing]);

    discovery.announce(&primal).await.expect("announce");

    let found = discovery
        .find_by_capability(&Capability::Signing)
        .await
        .expect("find");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].name, "announced");
}

#[tokio::test]
async fn test_local_discovery_all() {
    let discovery = LocalDiscovery::new();
    let signer = make_test_primal("signer", vec![Capability::Signing]);
    let anchor = make_test_primal("anchor", vec![Capability::Anchoring]);

    discovery.register(signer).await;
    discovery.register(anchor).await;

    let all = discovery.all().await;
    assert_eq!(all.len(), 2);
}

#[tokio::test]
async fn test_local_discovery_default() {
    let discovery = LocalDiscovery::default();
    assert!(discovery.health().await);
}
