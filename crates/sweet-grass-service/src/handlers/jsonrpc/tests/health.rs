// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! `health.*` dispatch tests.

use super::super::*;
use super::helpers::test_state;

#[tokio::test]
async fn test_health_method() {
    let state = test_state();
    let result = dispatch(&state, "health.check", serde_json::json!({})).await;
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value["status"], "healthy");
    assert_eq!(value["braid_count"], 0);
    assert!(
        value["primal"].is_string(),
        "health.check must include primal"
    );
    assert!(
        value["uptime_secs"].is_number(),
        "health.check must include uptime_secs"
    );
    assert!(
        value["version"].is_string(),
        "health.check must include version"
    );
}

#[tokio::test]
async fn test_bare_health_alias() {
    let state = test_state();
    let result = dispatch(&state, "health", serde_json::json!({})).await;
    assert!(result.is_ok(), "bare 'health' should resolve via alias");
    let value = result.unwrap();
    assert_eq!(value["status"], "healthy");
    assert!(value["primal"].is_string());
}

#[tokio::test]
async fn test_health_liveness() {
    let state = test_state();
    let result = dispatch(&state, "health.liveness", serde_json::json!({})).await;
    assert!(result.is_ok());
    let val = result.unwrap();
    assert_eq!(val["alive"], true);
}

#[tokio::test]
async fn test_health_readiness() {
    let state = test_state();
    let result = dispatch(&state, "health.readiness", serde_json::json!({})).await;
    assert!(result.is_ok());
    let val = result.unwrap();
    assert_eq!(val["ready"], true);
}
