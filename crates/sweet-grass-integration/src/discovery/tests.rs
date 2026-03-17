// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "test file: expect/unwrap are standard in tests"
)]

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
async fn test_preferred_address_tarpc_first() {
    let primal = make_test_primal("test", vec![Capability::Signing]);
    assert_eq!(primal.preferred_address(), primal.tarpc_address.as_deref());
}

#[tokio::test]
async fn test_preferred_address_rest_fallback() {
    let primal = DiscoveredPrimal {
        instance_id: "fallback-instance".to_string(),
        name: "fallback".to_string(),
        capabilities: vec![Capability::Signing],
        tarpc_address: None,
        rest_address: Some(crate::testing::TEST_REST_URL.to_string()),
        last_seen: std::time::SystemTime::now(),
        healthy: true,
    };
    assert_eq!(
        primal.preferred_address(),
        Some(crate::testing::TEST_REST_URL)
    );
}

#[tokio::test]
async fn test_preferred_address_none() {
    let primal = DiscoveredPrimal {
        instance_id: "no-addr-instance".to_string(),
        name: "no-addr".to_string(),
        capabilities: vec![Capability::Signing],
        tarpc_address: None,
        rest_address: None,
        last_seen: std::time::SystemTime::now(),
        healthy: true,
    };
    assert!(primal.preferred_address().is_none());
}

#[tokio::test]
async fn test_service_info_to_primal() {
    use super::ServiceInfo;

    let info = ServiceInfo {
        id: "svc-1".to_string(),
        name: "test-service".to_string(),
        version: "1.0.0".to_string(),
        tarpc_address: Some(crate::testing::TEST_TARPC_URI.to_string()),
        rest_address: Some(crate::testing::TEST_REST_URL.to_string()),
        capabilities: vec!["signing".to_string(), "anchoring".to_string()],
        last_seen: 1_700_000_000,
        healthy: true,
    };

    let primal = info.to_primal();
    assert_eq!(primal.instance_id, "svc-1");
    assert_eq!(primal.name, "test-service");
    assert!(primal.offers(&Capability::Signing));
    assert!(primal.offers(&Capability::Anchoring));
    assert!(primal.healthy);
}

#[tokio::test]
async fn test_discovery_error_display() {
    let err = DiscoveryError::CapabilityNotFound(Capability::Signing);
    assert!(err.to_string().contains("Signing"));

    let err = DiscoveryError::ConnectionFailed {
        address: crate::testing::TEST_TARPC_ADDR.to_string(),
        reason: "connection refused".to_string(),
    };
    assert!(err.to_string().contains(crate::testing::TEST_TARPC_ADDR));
    assert!(err.to_string().contains("connection refused"));

    let err = DiscoveryError::ServiceUnavailable("down".to_string());
    assert!(err.to_string().contains("down"));

    let err = DiscoveryError::Timeout(Duration::from_secs(5));
    assert!(err.to_string().contains('5'));
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
async fn test_create_discovery_no_env() {
    let discovery = super::create_discovery_with_reader(|_| None).await;
    assert!(discovery.health().await);
}

#[tokio::test]
async fn test_registry_from_reader_missing() {
    let result = super::RegistryDiscovery::from_reader(|_| None).await;
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(err.to_string().contains("No discovery address found"));
    }
}

#[tokio::test]
async fn test_create_discovery_with_invalid_addr_fallback() {
    let discovery = super::create_discovery_with_reader(|key| {
        (key == "DISCOVERY_ADDRESS").then(|| crate::testing::TEST_INVALID_ADDR.to_string())
    })
    .await;

    assert!(discovery.health().await);
}

#[tokio::test]
async fn test_create_discovery_prefers_discovery_address() {
    let discovery = super::create_discovery_with_reader(|key| match key {
        "DISCOVERY_ADDRESS" => Some(crate::testing::TEST_INVALID_ADDR.to_string()),
        "UNIVERSAL_ADAPTER_ADDRESS" => Some("127.0.0.1:2".to_string()),
        "DISCOVERY_BOOTSTRAP" => Some("127.0.0.1:3".to_string()),
        _ => None,
    })
    .await;

    assert!(discovery.health().await);
}

#[tokio::test]
async fn test_service_info_to_primal_empty_capabilities() {
    use super::ServiceInfo;

    let info = ServiceInfo {
        id: "empty-svc".to_string(),
        name: "empty".to_string(),
        version: "1.0.0".to_string(),
        tarpc_address: None,
        rest_address: None,
        capabilities: vec![],
        last_seen: 0,
        healthy: true,
    };

    let primal = info.to_primal();
    assert_eq!(primal.instance_id, "empty-svc");
    assert!(primal.capabilities.is_empty());
}

#[tokio::test]
async fn test_service_info_to_primal_mixed_capabilities() {
    use super::ServiceInfo;

    let info = ServiceInfo {
        id: "mixed-svc".to_string(),
        name: "mixed".to_string(),
        version: "1.0.0".to_string(),
        tarpc_address: None,
        rest_address: None,
        capabilities: vec!["signing".to_string(), "unknown_xyz".to_string()],
        last_seen: 0,
        healthy: true,
    };

    let primal = info.to_primal();
    assert!(primal.offers(&Capability::Signing));
    assert!(primal.offers(&Capability::Custom("unknown_xyz".to_string())));
}

#[tokio::test]
async fn test_service_info_to_primal_filters_invalid_capability_strings() {
    use super::ServiceInfo;

    let info = ServiceInfo {
        id: "filter-svc".to_string(),
        name: "filter".to_string(),
        version: "1.0.0".to_string(),
        tarpc_address: None,
        rest_address: None,
        capabilities: vec!["signing".to_string(), String::new()],
        last_seen: 0,
        healthy: true,
    };

    let primal = info.to_primal();
    assert_eq!(primal.capabilities.len(), 1);
    assert!(primal.offers(&Capability::Signing));
}

#[tokio::test]
async fn test_service_info_to_primal_custom_capability() {
    use super::ServiceInfo;

    let info = ServiceInfo {
        id: "custom-svc".to_string(),
        name: "custom".to_string(),
        version: "1.0.0".to_string(),
        tarpc_address: None,
        rest_address: None,
        capabilities: vec!["custom:my_feature".to_string()],
        last_seen: 1_700_000_000,
        healthy: false,
    };

    let primal = info.to_primal();
    assert!(primal.offers(&Capability::Custom("my_feature".to_string())));
    assert!(!primal.healthy);
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

#[tokio::test]
async fn test_registry_discovery_connect_invalid_addr() {
    let result = super::RegistryDiscovery::connect(crate::testing::TEST_INVALID_ADDR).await;
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(
            err.to_string().contains(crate::testing::TEST_INVALID_ADDR)
                || err.to_string().contains("connection")
        );
    }
}

// ── extract_capabilities tests ────────────────────────────────────────────

#[test]
fn extract_capabilities_flat_array() {
    let resp = serde_json::json!({"methods": ["braid.create", "health.check", "braid.get"]});
    let caps = super::extract_capabilities(&resp);
    assert_eq!(caps, vec!["braid.create", "braid.get", "health.check"]);
}

#[test]
fn extract_capabilities_capabilities_alias() {
    let resp = serde_json::json!({"capabilities": ["store.put", "store.get"]});
    let caps = super::extract_capabilities(&resp);
    assert_eq!(caps, vec!["store.get", "store.put"]);
}

#[test]
fn extract_capabilities_structured_domains() {
    let resp = serde_json::json!({
        "domains": {
            "braid": ["create", "get"],
            "health": ["check", "liveness"]
        }
    });
    let caps = super::extract_capabilities(&resp);
    assert_eq!(
        caps,
        vec![
            "braid.create",
            "braid.get",
            "health.check",
            "health.liveness"
        ]
    );
}

#[test]
fn extract_capabilities_result_wrapper() {
    let resp = serde_json::json!({
        "jsonrpc": "2.0",
        "result": {"methods": ["a.b", "c.d"]},
        "id": 1
    });
    assert_eq!(super::extract_capabilities(&resp), vec!["a.b", "c.d"]);
}

#[test]
fn extract_capabilities_empty() {
    let resp = serde_json::json!({});
    assert!(super::extract_capabilities(&resp).is_empty());
}

#[test]
fn extract_capabilities_deduplicates() {
    let resp = serde_json::json!({"methods": ["a.b", "a.b", "c.d"]});
    assert_eq!(super::extract_capabilities(&resp), vec!["a.b", "c.d"]);
}

// ── extract_capabilities proptest ─────────────────────────────────────────

proptest::proptest! {
    #[test]
    fn extract_capabilities_flat_roundtrip(
        methods in proptest::collection::vec("[a-z]{1,8}\\.[a-z]{1,8}", 0..20)
    ) {
        let resp = serde_json::json!({"methods": methods});
        let caps = super::extract_capabilities(&resp);
        for cap in &caps {
            proptest::prop_assert!(cap.contains('.'));
        }
        let mut expected: Vec<String> = methods;
        expected.sort();
        expected.dedup();
        proptest::prop_assert_eq!(caps, expected);
    }

    #[test]
    fn extract_capabilities_never_panics(s in "\\PC{0,200}") {
        let val: std::result::Result<serde_json::Value, _> = serde_json::from_str(&s);
        if let Ok(v) = val {
            let _ = super::extract_capabilities(&v);
        }
    }
}
