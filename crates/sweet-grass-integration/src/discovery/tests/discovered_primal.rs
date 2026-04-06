// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use super::*;

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
async fn test_primal_tarpc_address() {
    let primal = make_test_primal("test", vec![Capability::Signing]);
    assert!(primal.tarpc_address.is_some());
    assert!(primal.rest_address.is_some());
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
