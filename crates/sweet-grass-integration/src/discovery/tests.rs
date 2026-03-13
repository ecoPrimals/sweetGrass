// SPDX-License-Identifier: AGPL-3.0-only

#![allow(clippy::float_cmp, clippy::expect_used, clippy::unwrap_used)]

use super::*;

fn make_test_primal(name: &str, capabilities: Vec<Capability>) -> DiscoveredPrimal {
    // Use OS-allocated ports for test primals
    let [tarpc_port, rest_port] = crate::testing::allocate_test_ports::<2>();

    DiscoveredPrimal {
        instance_id: format!("{name}-instance"),
        name: name.to_string(),
        capabilities,
        tarpc_address: Some(format!("localhost:{tarpc_port}")),
        rest_address: Some(format!("localhost:{rest_port}")),
        last_seen: std::time::SystemTime::now(),
        healthy: true,
    }
}

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
async fn test_discovered_primal_offers() {
    let primal = make_test_primal(
        "multi",
        vec![Capability::Signing, Capability::custom("custom")],
    );

    assert!(primal.offers(&Capability::Signing));
    assert!(primal.offers(&Capability::Custom("custom".to_string())));
    assert!(!primal.offers(&Capability::Anchoring));
}

#[tokio::test]
async fn test_cached_discovery() {
    let inner = Arc::new(LocalDiscovery::new());
    let signer = make_test_primal("signer", vec![Capability::Signing]);
    inner.register(signer).await;

    let cached = CachedDiscovery::new(inner, Duration::from_secs(60));

    // First call populates cache
    let result1 = cached
        .find_by_capability(&Capability::Signing)
        .await
        .expect("find");
    assert_eq!(result1.len(), 1);

    // Second call uses cache
    let result2 = cached
        .find_by_capability(&Capability::Signing)
        .await
        .expect("find");
    assert_eq!(result2.len(), 1);
}

#[tokio::test]
async fn test_local_discovery_health() {
    let discovery = LocalDiscovery::new();
    assert!(discovery.health().await);
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
async fn test_cached_discovery_find_one() {
    let inner = Arc::new(LocalDiscovery::new());
    let signer = make_test_primal("cached-signer", vec![Capability::Signing]);
    inner.register(signer).await;

    let cached = CachedDiscovery::new(inner, Duration::from_secs(60));
    let found = cached.find_one(&Capability::Signing).await.expect("find");
    assert_eq!(found.name, "cached-signer");
}

#[tokio::test]
async fn test_cached_discovery_health() {
    let inner = Arc::new(LocalDiscovery::new());
    let cached = CachedDiscovery::new(inner, Duration::from_secs(60));
    assert!(cached.health().await);
}

#[tokio::test]
async fn test_primal_with_all_capabilities() {
    let primal = make_test_primal(
        "full",
        vec![
            Capability::Signing,
            Capability::Anchoring,
            Capability::SessionEvents,
        ],
    );
    assert!(primal.offers(&Capability::Signing));
    assert!(primal.offers(&Capability::Anchoring));
    assert!(primal.offers(&Capability::SessionEvents));
    assert!(!primal.offers(&Capability::Compute));
}

#[tokio::test]
async fn test_discovery_empty_capabilities() {
    let primal = make_test_primal("empty", vec![]);
    assert!(!primal.offers(&Capability::Signing));
    assert!(!primal.offers(&Capability::Anchoring));
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
async fn test_primal_tarpc_address() {
    let primal = make_test_primal("test", vec![Capability::Signing]);
    assert!(primal.tarpc_address.is_some());
    assert!(primal.rest_address.is_some());
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
                tarpc_address: Some(format!("localhost:{}", 8090 + i)),
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
