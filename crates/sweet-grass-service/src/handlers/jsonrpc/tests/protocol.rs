// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! JSON-RPC 2.0 protocol envelope and error code tests.

use super::super::*;

#[test]
fn test_parse_error_response() {
    let resp = JsonRpcResponse::error(
        serde_json::Value::Null,
        error_code::PARSE_ERROR,
        "test parse error",
    );
    assert_eq!(resp.jsonrpc, "2.0");
    assert!(resp.error.is_some());
    assert!(resp.result.is_none());
    assert_eq!(resp.error.unwrap().code, error_code::PARSE_ERROR);
}

#[test]
fn test_success_response() {
    let resp = JsonRpcResponse::success(serde_json::json!(1), serde_json::json!({"status": "ok"}));
    assert_eq!(resp.jsonrpc, "2.0");
    assert!(resp.result.is_some());
    assert!(resp.error.is_none());
}

#[test]
fn test_invalid_version_detection() {
    let request = serde_json::json!({
        "jsonrpc": "1.0",
        "method": "health.check",
        "params": {},
        "id": 1
    });
    let parsed: JsonRpcRequest = serde_json::from_value(request).unwrap();
    assert_ne!(parsed.jsonrpc, "2.0");
}

#[test]
fn test_all_error_codes() {
    assert_eq!(error_code::PARSE_ERROR, -32700);
    assert_eq!(error_code::INVALID_REQUEST, -32600);
    assert_eq!(error_code::METHOD_NOT_FOUND, -32601);
    assert_eq!(error_code::INVALID_PARAMS, -32602);
    assert_eq!(error_code::INTERNAL_ERROR, -32603);
    assert_eq!(error_code::NOT_FOUND, -32004);
    assert_eq!(error_code::PERMISSION_DENIED, -32001);
    assert_eq!(error_code::UNAUTHORIZED, -32000);
}
