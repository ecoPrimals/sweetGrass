// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

#[expect(
    unused_imports,
    reason = "discovery APIs are invoked via super:: paths matching the original tests.rs"
)]
use super::*;

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
