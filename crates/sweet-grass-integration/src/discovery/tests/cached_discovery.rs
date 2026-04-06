// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use super::*;

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
async fn test_cached_discovery_invalidate() {
    let inner = Arc::new(LocalDiscovery::new());
    let signer = make_test_primal("signer", vec![Capability::Signing]);
    inner.register(signer).await;

    let cached = CachedDiscovery::new(inner, Duration::from_secs(60));

    // Populate cache
    let _ = cached
        .find_by_capability(&Capability::Signing)
        .await
        .expect("find");

    // Invalidate and re-query - should still work (refreshes from inner)
    cached.invalidate(&Capability::Signing).await;
    let result = cached
        .find_by_capability(&Capability::Signing)
        .await
        .expect("find");
    assert_eq!(result.len(), 1);
}

#[tokio::test]
async fn test_cached_discovery_invalidate_all() {
    let inner = Arc::new(LocalDiscovery::new());
    let signer = make_test_primal("signer", vec![Capability::Signing]);
    inner.register(signer).await;

    let cached = CachedDiscovery::new(inner, Duration::from_secs(60));
    let _ = cached
        .find_by_capability(&Capability::Signing)
        .await
        .expect("find");

    cached.invalidate_all().await;
    let result = cached
        .find_by_capability(&Capability::Signing)
        .await
        .expect("find");
    assert_eq!(result.len(), 1);
}

#[tokio::test]
async fn test_cached_discovery_expired_entries() {
    let inner = Arc::new(LocalDiscovery::new());
    let signer = make_test_primal("signer", vec![Capability::Signing]);
    inner.register(signer).await;

    let cached = CachedDiscovery::new(inner, Duration::from_millis(1));

    let _ = cached
        .find_by_capability(&Capability::Signing)
        .await
        .expect("populate cache");

    tokio::time::sleep(Duration::from_millis(10)).await;

    let result = cached
        .find_by_capability(&Capability::Signing)
        .await
        .expect("should refresh after TTL expires");
    assert_eq!(result.len(), 1);
}

#[tokio::test]
async fn test_cached_discovery_announce() {
    let inner = Arc::new(LocalDiscovery::new());
    let cached = CachedDiscovery::new(inner, Duration::from_secs(60));

    let primal = make_test_primal("announced", vec![Capability::Signing]);
    cached.announce(&primal).await.expect("announce");

    let found = cached
        .find_by_capability(&Capability::Signing)
        .await
        .expect("find");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].name, "announced");
}

#[tokio::test]
async fn test_cached_discovery_invalidate_forces_refresh() {
    let inner = Arc::new(LocalDiscovery::new());
    let signer = make_test_primal("signer-original", vec![Capability::Signing]);
    inner.register(signer).await;

    let inner_dyn: Arc<dyn PrimalDiscovery> = Arc::clone(&inner) as Arc<dyn PrimalDiscovery>;
    let cached = CachedDiscovery::new(inner_dyn, Duration::from_secs(300));
    let _ = cached
        .find_by_capability(&Capability::Signing)
        .await
        .unwrap();

    inner.unregister("signer-original-instance").await;
    let updated = make_test_primal("signer-updated", vec![Capability::Signing]);
    inner.register(updated).await;

    cached.invalidate(&Capability::Signing).await;
    let result = cached
        .find_by_capability(&Capability::Signing)
        .await
        .unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "signer-updated");
}

#[tokio::test]
async fn test_cached_discovery_different_capabilities_separate_cache() {
    let inner = Arc::new(LocalDiscovery::new());
    inner
        .register(make_test_primal("signer", vec![Capability::Signing]))
        .await;
    inner
        .register(make_test_primal("anchor", vec![Capability::Anchoring]))
        .await;

    let cached = CachedDiscovery::new(inner, Duration::from_secs(60));

    let signers = cached
        .find_by_capability(&Capability::Signing)
        .await
        .expect("find");
    let anchors = cached
        .find_by_capability(&Capability::Anchoring)
        .await
        .expect("find");

    assert_eq!(signers.len(), 1);
    assert_eq!(anchors.len(), 1);
    assert_eq!(signers[0].name, "signer");
    assert_eq!(anchors[0].name, "anchor");
}

#[tokio::test]
async fn test_cached_discovery_find_one_no_healthy_uses_cache() {
    let inner = Arc::new(LocalDiscovery::new());
    let mut unhealthy = make_test_primal("unhealthy", vec![Capability::Signing]);
    unhealthy.healthy = false;
    inner.register(unhealthy).await;

    let cached = CachedDiscovery::new(inner, Duration::from_secs(60));

    let result = cached.find_one(&Capability::Signing).await;
    assert!(result.is_err());
}
