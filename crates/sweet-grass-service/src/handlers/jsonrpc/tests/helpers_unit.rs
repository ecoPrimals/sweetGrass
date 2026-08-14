// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Unit tests for JSON-RPC dispatch helper functions.

use super::super::*;

#[test]
fn test_parse_params_valid() {
    let val = serde_json::json!({"id": "test-id"});
    let result: Result<super::super::braid::GetBraidParams, _> = parse_params(val);
    assert!(result.is_ok());
}

#[test]
fn test_parse_params_invalid() {
    let val = serde_json::json!({"wrong_field": 123});
    let result: Result<super::super::braid::GetBraidParams, _> = parse_params(val);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, error_code::INVALID_PARAMS);
}

#[test]
fn test_to_value_success() {
    let data = serde_json::json!({"key": "value"});
    let result = to_value(&data);
    assert!(result.is_ok());
}

#[test]
fn test_internal_error() {
    let err = internal("something went wrong");
    assert_eq!(err.code, error_code::INTERNAL_ERROR);
    assert!(err.message.contains("something went wrong"));
    assert_eq!(err.source_detail.as_deref(), Some("something went wrong"));
}
