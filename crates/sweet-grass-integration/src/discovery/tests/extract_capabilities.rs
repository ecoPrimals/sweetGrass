// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use super::*;

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
