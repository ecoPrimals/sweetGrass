// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Unit tests for BTSP server-side handshake.

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test module")]

use base64::Engine;

use super::server::*;

#[test]
fn resolve_security_socket_default() {
    temp_env::with_vars(
        [
            ("SECURITY_PROVIDER_SOCKET", None::<&str>),
            ("BEARDOG_SOCKET", None::<&str>),
            ("BIOMEOS_SOCKET_DIR", None::<&str>),
        ],
        || {
            let path = resolve_security_socket_from_env();
            let path_str = path.to_string_lossy();
            assert!(
                path_str.contains("security"),
                "should resolve to security.sock: {path_str}"
            );
        },
    );
}

#[tokio::test]
async fn handshake_rejects_bad_version() {
    let hello = super::protocol::ClientHello {
        version: 99,
        client_ephemeral_pub: "dGVzdA==".to_string(),
    };

    let mut buf = Vec::new();
    super::protocol::write_message(&mut buf, &hello)
        .await
        .expect("write hello");

    let mut cursor = std::io::Cursor::new(buf);
    let result = perform_server_handshake(&mut cursor).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("unsupported version"),
        "expected version error, got: {err}"
    );
}

#[test]
fn extract_str_missing_field() {
    let value = serde_json::json!({"other": "val"});
    let err = super::server::extract_str(&value, "missing").unwrap_err();
    assert!(
        err.to_string().contains("missing"),
        "should mention field name: {err}"
    );
}

#[test]
fn extract_str_success() {
    let value = serde_json::json!({"session_id": "abc123"});
    let result = super::server::extract_str(&value, "session_id").expect("should extract");
    assert_eq!(result, "abc123");
}

#[test]
fn extract_str_non_string_field() {
    let value = serde_json::json!({"count": 42});
    let err = super::server::extract_str(&value, "count").unwrap_err();
    assert!(err.to_string().contains("missing"));
}

#[test]
fn resolve_security_socket_explicit_env() {
    temp_env::with_vars(
        [
            ("SECURITY_PROVIDER_SOCKET", Some("/custom/path.sock")),
            ("BIOMEOS_SOCKET_DIR", None::<&str>),
            ("XDG_RUNTIME_DIR", None::<&str>),
        ],
        || {
            assert_eq!(
                resolve_security_socket_from_env(),
                std::path::PathBuf::from("/custom/path.sock")
            );
        },
    );
}

#[test]
fn resolve_security_socket_beardog_env() {
    temp_env::with_vars(
        [
            ("SECURITY_PROVIDER_SOCKET", None::<&str>),
            ("BEARDOG_SOCKET", Some("/run/biomeos/beardog.sock")),
            ("BIOMEOS_SOCKET_DIR", Some("/run/biomeos")),
            ("XDG_RUNTIME_DIR", None::<&str>),
        ],
        || {
            assert_eq!(
                resolve_security_socket_from_env(),
                std::path::PathBuf::from("/run/biomeos/beardog.sock"),
                "BEARDOG_SOCKET should take precedence over BIOMEOS_SOCKET_DIR"
            );
        },
    );
}

#[test]
fn resolve_security_socket_biomeos_dir() {
    temp_env::with_vars(
        [
            ("SECURITY_PROVIDER_SOCKET", None::<&str>),
            ("BIOMEOS_SOCKET_DIR", Some("/run/biomeos")),
            ("XDG_RUNTIME_DIR", None::<&str>),
        ],
        || {
            assert_eq!(
                resolve_security_socket_from_env(),
                std::path::PathBuf::from("/run/biomeos/security.sock")
            );
        },
    );
}

#[test]
fn resolve_security_socket_xdg_runtime() {
    temp_env::with_vars(
        [
            ("SECURITY_PROVIDER_SOCKET", None::<&str>),
            ("BIOMEOS_SOCKET_DIR", None::<&str>),
            ("XDG_RUNTIME_DIR", Some("/run/user/1000")),
        ],
        || {
            assert_eq!(
                resolve_security_socket_from_env(),
                std::path::PathBuf::from("/run/user/1000/biomeos/security.sock")
            );
        },
    );
}

#[test]
fn resolve_family_seed_from_primary() {
    temp_env::with_vars(
        [
            ("FAMILY_SEED", Some("deadbeef01234567")),
            ("BEARDOG_FAMILY_SEED", None::<&str>),
        ],
        || {
            let b64 = resolve_family_seed_from_env().expect("should resolve");
            let decoded = base64::engine::general_purpose::STANDARD
                .decode(&b64)
                .expect("valid base64");
            assert_eq!(decoded, b"deadbeef01234567");
        },
    );
}

#[test]
fn resolve_family_seed_fallback_to_beardog() {
    temp_env::with_vars(
        [
            ("FAMILY_SEED", None::<&str>),
            ("BEARDOG_FAMILY_SEED", Some("fallback_seed_hex")),
        ],
        || {
            let b64 = resolve_family_seed_from_env().expect("should resolve");
            let decoded = base64::engine::general_purpose::STANDARD
                .decode(&b64)
                .expect("valid base64");
            assert_eq!(decoded, b"fallback_seed_hex");
        },
    );
}

#[test]
fn resolve_family_seed_primary_takes_precedence() {
    temp_env::with_vars(
        [
            ("FAMILY_SEED", Some("primary")),
            ("BEARDOG_FAMILY_SEED", Some("secondary")),
        ],
        || {
            let b64 = resolve_family_seed_from_env().expect("should resolve");
            let decoded = base64::engine::general_purpose::STANDARD
                .decode(&b64)
                .expect("valid base64");
            assert_eq!(decoded, b"primary");
        },
    );
}

#[test]
fn resolve_family_seed_missing_both() {
    temp_env::with_vars(
        [
            ("FAMILY_SEED", None::<&str>),
            ("BEARDOG_FAMILY_SEED", None::<&str>),
        ],
        || {
            let err = resolve_family_seed_from_env().unwrap_err();
            assert!(
                err.to_string().contains("FAMILY_SEED"),
                "error should mention variable: {err}"
            );
        },
    );
}

#[test]
fn resolve_family_seed_hex_roundtrip() {
    let hex_seed = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2";
    temp_env::with_vars(
        [
            ("FAMILY_SEED", Some(hex_seed)),
            ("BEARDOG_FAMILY_SEED", None::<&str>),
        ],
        || {
            let b64 = resolve_family_seed_from_env().expect("should resolve");
            let decoded = base64::engine::general_purpose::STANDARD
                .decode(&b64)
                .expect("valid base64");
            assert_eq!(
                std::str::from_utf8(&decoded).expect("utf8"),
                hex_seed,
                "BearDog receives the raw hex string bytes"
            );
        },
    );
}

#[test]
fn extract_handshake_key_valid() {
    let key_bytes = [0xABu8; 32];
    let b64 = base64::engine::general_purpose::STANDARD.encode(key_bytes);
    let verify_result = serde_json::json!({"verified": true, "session_key": b64});
    let key = super::server::extract_handshake_key(&verify_result).expect("should extract key");
    assert_eq!(key, key_bytes);
}

#[test]
fn extract_handshake_key_missing_field() {
    let verify_result = serde_json::json!({"verified": true});
    assert!(super::server::extract_handshake_key(&verify_result).is_none());
}

#[test]
fn extract_handshake_key_wrong_length() {
    let short = base64::engine::general_purpose::STANDARD.encode([1u8; 16]);
    let verify_result = serde_json::json!({"verified": true, "session_key": short});
    assert!(
        super::server::extract_handshake_key(&verify_result).is_none(),
        "16 bytes should be rejected (need 32)"
    );
}

#[test]
fn extract_handshake_key_invalid_base64() {
    let verify_result = serde_json::json!({"verified": true, "session_key": "not-valid-b64!!!"});
    assert!(super::server::extract_handshake_key(&verify_result).is_none());
}
