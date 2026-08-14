// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! `lifecycle.status` dispatch tests.

use super::super::*;
use super::helpers::test_state;

#[tokio::test]
async fn test_lifecycle_status_returns_running() {
    let state = test_state();
    let result = dispatch(&state, "lifecycle.status", serde_json::json!({}))
        .await
        .unwrap();
    assert_eq!(result["status"], "running");
    assert!(result["version"].is_string());
    assert!(result["gate_mode"].is_string());
    assert!(result["uptime_secs"].is_number());
    assert!(result["method_count"].is_number());
    assert!(result["capabilities_count"].is_number());
    assert_eq!(result["store_backend"], "memory");
    assert_eq!(result["method_count"], METHODS.len());
    assert_eq!(
        result["capabilities_count"],
        sweet_grass_core::niche::CAPABILITIES.len()
    );
}
