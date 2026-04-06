// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use super::*;

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
