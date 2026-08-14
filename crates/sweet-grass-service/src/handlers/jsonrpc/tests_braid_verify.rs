// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Comprehensive tests for the `braid.verify` JSON-RPC handler.

#![expect(clippy::unwrap_used, reason = "test module")]

use std::path::{Path, PathBuf};

use super::*;
use crate::crypto_delegate::CryptoDelegate;
use crate::ledger_client::LedgerClient;
use crate::state::AppState;
use sweet_grass_core::agent::Did;
use sweet_grass_core::braid::ContentHash;
use sweet_grass_core::dehydration::Witness;
use sweet_grass_store::BraidStore;

fn test_state() -> AppState {
    AppState::new_memory(Did::new("did:key:z6MkTest"))
}

fn find_check<'a>(checks: &'a [serde_json::Value], name: &str) -> &'a serde_json::Value {
    checks
        .iter()
        .find(|c| c["check"].as_str() == Some(name))
        .unwrap_or_else(|| panic!("check {name} not found in {checks:?}"))
}

async fn create_braid(state: &AppState, data_hash: &str) -> String {
    let result = dispatch(
        state,
        "braid.create",
        serde_json::json!({
            "data_hash": data_hash,
            "mime_type": "text/plain",
            "size": 64
        }),
    )
    .await
    .unwrap();
    result["@id"].as_str().unwrap().to_string()
}

async fn create_signed_braid(
    state: &AppState,
    data_hash: &str,
    agent: &Did,
    signature_bytes: &[u8],
) -> String {
    let witness = Witness::from_ed25519(agent, signature_bytes);
    let result = dispatch(
        state,
        "braid.create",
        serde_json::json!({
            "data_hash": data_hash,
            "mime_type": "text/plain",
            "size": 64,
            "witness": serde_json::to_value(&witness).unwrap()
        }),
    )
    .await
    .unwrap();
    result["@id"].as_str().unwrap().to_string()
}

async fn verify_braid(state: &AppState, braid_id: &str) -> serde_json::Value {
    dispatch(
        state,
        "braid.verify",
        serde_json::json!({ "braid_id": braid_id }),
    )
    .await
    .unwrap()
}

async fn store_braid_with_corrupted_data_hash(state: &AppState, data_hash: &str) -> String {
    let mut braid = state
        .factory
        .from_hash(
            ContentHash::new(
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            ),
            "text/plain",
            64,
            None,
        )
        .unwrap();
    braid.data_hash = ContentHash::new(data_hash);
    let braid_id = braid.id.as_str().to_string();
    state.store.put(&braid).await.unwrap();
    braid_id
}

#[cfg(unix)]
fn start_mock_crypto_verify(valid: bool) -> (PathBuf, tokio::task::JoinHandle<()>) {
    use std::os::unix::net::UnixListener as StdUnixListener;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let dir = tempfile::tempdir().unwrap();
    let sock = dir.path().join("crypto-verify.sock");
    let sock_path = sock.clone();

    let std_listener = StdUnixListener::bind(&sock).unwrap();
    std_listener.set_nonblocking(true).unwrap();
    let listener = tokio::net::UnixListener::from_std(std_listener).unwrap();

    let handle = tokio::spawn(async move {
        let _dir = dir;
        loop {
            let Ok((stream, _)) = listener.accept().await else {
                break;
            };
            tokio::spawn(async move {
                let (reader, mut writer) = stream.into_split();
                let mut lines = BufReader::new(reader).lines();
                if let Ok(Some(line)) = lines.next_line().await {
                    let req: serde_json::Value = serde_json::from_str(&line).unwrap_or_default();
                    let response = serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": req["id"],
                        "result": { "valid": valid }
                    });
                    let mut resp = serde_json::to_string(&response).unwrap();
                    resp.push('\n');
                    let _ = writer.write_all(resp.as_bytes()).await;
                }
            });
        }
    });

    (sock_path, handle)
}

#[cfg(unix)]
fn start_mock_crypto_error() -> (PathBuf, tokio::task::JoinHandle<()>) {
    use std::os::unix::net::UnixListener as StdUnixListener;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let dir = tempfile::tempdir().unwrap();
    let sock = dir.path().join("crypto-error.sock");
    let sock_path = sock.clone();

    let std_listener = StdUnixListener::bind(&sock).unwrap();
    std_listener.set_nonblocking(true).unwrap();
    let listener = tokio::net::UnixListener::from_std(std_listener).unwrap();

    let handle = tokio::spawn(async move {
        let _dir = dir;
        loop {
            let Ok((stream, _)) = listener.accept().await else {
                break;
            };
            tokio::spawn(async move {
                let (reader, mut writer) = stream.into_split();
                let mut lines = BufReader::new(reader).lines();
                if let Ok(Some(line)) = lines.next_line().await {
                    let req: serde_json::Value = serde_json::from_str(&line).unwrap_or_default();
                    let response = serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": req["id"],
                        "error": { "code": -32000, "message": "verification service down" }
                    });
                    let mut resp = serde_json::to_string(&response).unwrap();
                    resp.push('\n');
                    let _ = writer.write_all(resp.as_bytes()).await;
                }
            });
        }
    });

    (sock_path, handle)
}

fn start_mock_loamspine(valid: bool) -> (PathBuf, tokio::task::JoinHandle<()>) {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let dir = tempfile::tempdir().unwrap();
    let sock = dir.path().join("loamspine-verify.sock");
    let sock_path = sock.clone();

    let handle = tokio::spawn(async move {
        let listener = tokio::net::UnixListener::bind(&sock).unwrap();
        let _dir = dir;
        loop {
            let Ok((stream, _)) = listener.accept().await else {
                break;
            };
            tokio::spawn(async move {
                let (reader, mut writer) = stream.into_split();
                let mut lines = BufReader::new(reader).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let request: serde_json::Value =
                        serde_json::from_str(&line).unwrap_or_default();
                    let id = request
                        .get("id")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!(0));
                    let response = serde_json::json!({
                        "jsonrpc": "2.0",
                        "result": {
                            "valid": valid,
                            "detail": if valid {
                                "sealed in ledger"
                            } else {
                                "certificate not found in ledger"
                            }
                        },
                        "id": id
                    });
                    let mut buf = serde_json::to_string(&response).unwrap();
                    buf.push('\n');
                    if writer.write_all(buf.as_bytes()).await.is_err() {
                        break;
                    }
                }
            });
        }
    });

    (sock_path, handle)
}

fn start_unreachable_loamspine() -> LedgerClient {
    LedgerClient::from_socket_path(Path::new("/nonexistent/loamspine.sock"))
}

async fn state_with_ledger(valid: bool) -> (AppState, tokio::task::JoinHandle<()>) {
    let (sock, handle) = start_mock_loamspine(valid);
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let client = LedgerClient::from_socket_path(&sock);
    let state = test_state().with_ledger_client(client);
    (state, handle)
}

#[cfg(unix)]
async fn state_with_crypto(valid: bool) -> (AppState, tokio::task::JoinHandle<()>) {
    let (sock, handle) = start_mock_crypto_verify(valid);
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let crypto = CryptoDelegate::with_socket(sock);
    let state = test_state().with_crypto(crypto);
    (state, handle)
}

// ==================== content integrity ====================

#[tokio::test]
async fn test_braid_verify_content_integrity_pass() {
    let state = test_state();
    let braid_id = create_braid(&state, "sha256:verify-content-pass").await;

    let result = verify_braid(&state, &braid_id).await;
    let checks = result["checks"].as_array().unwrap();
    let integrity = find_check(checks, "content_integrity");

    assert_eq!(integrity["status"], "pass");
    assert!(
        integrity["signing_hash"]
            .as_str()
            .unwrap()
            .starts_with("sha256:")
    );
    assert_eq!(integrity["data_hash"], "sha256:verify-content-pass");
}

#[tokio::test]
async fn test_braid_verify_content_integrity_fail_corrupted_hash() {
    let state = test_state();
    let braid_id = store_braid_with_corrupted_data_hash(&state, "md5:corrupted-not-sha256").await;

    let result = verify_braid(&state, &braid_id).await;
    let checks = result["checks"].as_array().unwrap();
    let integrity = find_check(checks, "content_integrity");

    assert_eq!(integrity["status"], "fail");
    assert_eq!(integrity["data_hash"], "md5:corrupted-not-sha256");
    assert_eq!(result["verified"], false);
}

#[test]
fn test_content_integrity_valid_helper() {
    let signing = ContentHash::new("sha256:abc123");
    let data = ContentHash::new("sha256:def456");
    assert!(super::braid_verify::content_integrity_valid(
        &signing, &data
    ));

    let bad_data = ContentHash::new("md5:bad");
    assert!(!super::braid_verify::content_integrity_valid(
        &signing, &bad_data
    ));

    let bad_signing = ContentHash::new("not-sha256");
    assert!(!super::braid_verify::content_integrity_valid(
        &bad_signing,
        &data
    ));
}

// ==================== signature checks ====================

#[tokio::test]
async fn test_braid_verify_unsigned_braid_reports_unsigned() {
    let state = test_state();
    let braid_id = create_braid(&state, "sha256:verify-unsigned-status").await;

    let result = verify_braid(&state, &braid_id).await;
    let checks = result["checks"].as_array().unwrap();
    let signature = find_check(checks, "signature");

    assert_eq!(signature["status"], "unsigned");
    assert_eq!(signature["detail"], "no witness signature present");
    assert_eq!(result["verified"], false);
}

#[tokio::test]
async fn test_braid_verify_signed_without_crypto_reports_present() {
    let state = test_state();
    let agent = Did::from_public_key_bytes(b"test-public-key-bytes-32-chars!!");
    let braid_id =
        create_signed_braid(&state, "sha256:signed-no-crypto", &agent, b"fake-sig-bytes").await;

    let result = verify_braid(&state, &braid_id).await;
    let checks = result["checks"].as_array().unwrap();
    let signature = find_check(checks, "signature");

    assert_eq!(signature["status"], "present");
    assert!(
        signature["detail"]
            .as_str()
            .unwrap()
            .contains("crypto provider unavailable")
    );
    assert_eq!(signature["agent"], agent.as_str());
}

#[tokio::test]
async fn test_braid_verify_signed_empty_evidence_fails() {
    let state = test_state();
    let agent = Did::from_public_key_bytes(b"empty-evidence-key-bytes-32-chars!");
    let witness = serde_json::json!({
        "agent": agent.as_str(),
        "kind": "signature",
        "evidence": "",
        "encoding": "base64",
        "algorithm": "ed25519"
    });
    let create = dispatch(
        &state,
        "braid.create",
        serde_json::json!({
            "data_hash": "sha256:empty-evidence",
            "mime_type": "text/plain",
            "size": 64,
            "witness": witness
        }),
    )
    .await
    .unwrap();
    let braid_id = create["@id"].as_str().unwrap();

    let result = verify_braid(&state, braid_id).await;
    let checks = result["checks"].as_array().unwrap();
    let signature = find_check(checks, "signature");

    assert_eq!(signature["status"], "unsigned");
}

#[tokio::test]
async fn test_braid_verify_signed_invalid_base64_fails() {
    let state = test_state();
    let agent = Did::from_public_key_bytes(b"invalid-b64-key-bytes-32-chars!!");
    let witness = serde_json::json!({
        "agent": agent.as_str(),
        "kind": "signature",
        "evidence": "not!!!valid-base64",
        "encoding": "base64",
        "algorithm": "ed25519"
    });
    let create = dispatch(
        &state,
        "braid.create",
        serde_json::json!({
            "data_hash": "sha256:invalid-b64-evidence",
            "mime_type": "text/plain",
            "size": 64,
            "witness": witness
        }),
    )
    .await
    .unwrap();
    let braid_id = create["@id"].as_str().unwrap();

    let result = verify_braid(&state, braid_id).await;
    let checks = result["checks"].as_array().unwrap();
    let signature = find_check(checks, "signature");

    assert_eq!(signature["status"], "fail");
    assert_eq!(signature["detail"], "witness evidence is not valid base64");
    assert_eq!(result["verified"], false);
}

#[cfg(unix)]
#[tokio::test]
async fn test_braid_verify_signed_unextractable_did_reports_present() {
    let (state, crypto_handle) = state_with_crypto(true).await;
    let agent = Did::new("did:web:example.com");
    let braid_id =
        create_signed_braid(&state, "sha256:bad-did-agent", &agent, b"fake-sig-bytes").await;

    let result = verify_braid(&state, &braid_id).await;
    let checks = result["checks"].as_array().unwrap();
    let signature = find_check(checks, "signature");

    assert_eq!(signature["status"], "present");
    assert!(
        signature["detail"]
            .as_str()
            .unwrap()
            .contains("cannot extract public key from agent DID")
    );

    crypto_handle.abort();
}

#[cfg(unix)]
#[tokio::test]
async fn test_braid_verify_signed_crypto_invalid_signature_fails() {
    let (state, crypto_handle) = state_with_crypto(false).await;
    let agent = Did::from_public_key_bytes(b"invalid-signature-key-bytes-32!");
    let braid_id = create_signed_braid(
        &state,
        "sha256:crypto-invalid-sig",
        &agent,
        b"bad-signature",
    )
    .await;

    let result = verify_braid(&state, &braid_id).await;
    let checks = result["checks"].as_array().unwrap();
    let signature = find_check(checks, "signature");

    assert_eq!(signature["status"], "fail");
    assert_eq!(signature["detail"], "Ed25519 signature invalid");
    assert_eq!(result["verified"], false);

    crypto_handle.abort();
}

#[cfg(unix)]
#[tokio::test]
async fn test_braid_verify_signed_crypto_provider_error_reports_present() {
    let (sock, crypto_handle) = start_mock_crypto_error();
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let state = test_state().with_crypto(CryptoDelegate::with_socket(sock));
    let agent = Did::from_public_key_bytes(b"crypto-error-key-bytes-32-chars!");
    let braid_id =
        create_signed_braid(&state, "sha256:crypto-provider-error", &agent, b"sig-bytes").await;

    let result = verify_braid(&state, &braid_id).await;
    let checks = result["checks"].as_array().unwrap();
    let signature = find_check(checks, "signature");

    assert_eq!(signature["status"], "present");
    assert!(
        signature["detail"]
            .as_str()
            .unwrap()
            .contains("crypto provider error")
    );

    crypto_handle.abort();
}

// ==================== braid not found ====================

#[tokio::test]
async fn test_braid_verify_not_found() {
    let state = test_state();
    let result = dispatch(
        &state,
        "braid.verify",
        serde_json::json!({ "braid_id": "urn:braid:uuid:missing-braid" }),
    )
    .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, error_code::NOT_FOUND);
    assert!(err.message.contains("Braid not found"));
}

// ==================== ledger checks ====================

#[tokio::test]
async fn test_braid_verify_no_ledger_client_skipped() {
    let state = test_state();
    let braid_id = create_braid(&state, "sha256:ledger-skipped").await;

    let result = verify_braid(&state, &braid_id).await;
    let checks = result["checks"].as_array().unwrap();
    let ledger = find_check(checks, "ledger");

    assert_eq!(ledger["status"], "skipped");
    assert_eq!(ledger["detail"], "no ledger client configured");
}

#[tokio::test]
async fn test_braid_verify_ledger_pass() {
    let (state, ledger_handle) = state_with_ledger(true).await;
    let braid_id = create_braid(&state, "sha256:ledger-pass").await;

    let result = verify_braid(&state, &braid_id).await;
    let checks = result["checks"].as_array().unwrap();
    let ledger = find_check(checks, "ledger");

    assert_eq!(ledger["status"], "pass");
    assert_eq!(ledger["detail"], "sealed in ledger");

    ledger_handle.abort();
}

#[tokio::test]
async fn test_braid_verify_ledger_fail() {
    let (state, ledger_handle) = state_with_ledger(false).await;
    let braid_id = create_braid(&state, "sha256:ledger-fail").await;

    let result = verify_braid(&state, &braid_id).await;
    let checks = result["checks"].as_array().unwrap();
    let ledger = find_check(checks, "ledger");

    assert_eq!(ledger["status"], "fail");
    assert_eq!(ledger["detail"], "certificate not found in ledger");
    assert_eq!(result["verified"], false);

    ledger_handle.abort();
}

#[tokio::test]
async fn test_braid_verify_ledger_unavailable_skipped() {
    let state = test_state().with_ledger_client(start_unreachable_loamspine());
    let braid_id = create_braid(&state, "sha256:ledger-unavailable").await;

    let result = verify_braid(&state, &braid_id).await;
    let checks = result["checks"].as_array().unwrap();
    let ledger = find_check(checks, "ledger");

    assert_eq!(ledger["status"], "skipped");
    assert!(
        ledger["detail"]
            .as_str()
            .unwrap()
            .contains("loamSpine unavailable")
    );
}

// ==================== combined outcomes ====================

#[cfg(unix)]
#[tokio::test]
async fn test_braid_verify_all_checks_pass() {
    let (sock, crypto_handle) = start_mock_crypto_verify(true);
    let (ledger_sock, ledger_handle) = start_mock_loamspine(true);
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let state = test_state()
        .with_crypto(CryptoDelegate::with_socket(sock))
        .with_ledger_client(LedgerClient::from_socket_path(&ledger_sock));

    let agent = Did::from_public_key_bytes(b"all-checks-pass-key-bytes-32-chars");
    let braid_id =
        create_signed_braid(&state, "sha256:all-checks-pass", &agent, b"valid-signature").await;

    let result = verify_braid(&state, &braid_id).await;
    let checks = result["checks"].as_array().unwrap();

    assert_eq!(find_check(checks, "content_integrity")["status"], "pass");
    assert_eq!(find_check(checks, "signature")["status"], "pass");
    assert_eq!(find_check(checks, "ledger")["status"], "pass");
    assert_eq!(result["verified"], true);

    crypto_handle.abort();
    ledger_handle.abort();
}

#[tokio::test]
async fn test_braid_verify_mixed_results_verified_false() {
    let (state, ledger_handle) = state_with_ledger(false).await;
    let braid_id = create_braid(&state, "sha256:mixed-results").await;

    let result = verify_braid(&state, &braid_id).await;
    let checks = result["checks"].as_array().unwrap();

    assert_eq!(find_check(checks, "content_integrity")["status"], "pass");
    assert_eq!(find_check(checks, "signature")["status"], "unsigned");
    assert_eq!(find_check(checks, "ledger")["status"], "fail");
    assert_eq!(result["verified"], false);

    ledger_handle.abort();
}

// ==================== extract_public_key_from_did ====================

#[test]
fn test_extract_public_key_from_did_valid_ed25519() {
    let public_key = b"0123456789abcdef0123456789abcdef";
    let did = Did::from_public_key_bytes(public_key);
    let extracted = super::braid_verify::extract_public_key_from_did(&did).unwrap();
    assert_eq!(extracted, public_key);
}

#[test]
fn test_extract_public_key_from_did_wrong_method() {
    let did = Did::new("did:web:example.com");
    assert!(super::braid_verify::extract_public_key_from_did(&did).is_none());
}

#[test]
fn test_extract_public_key_from_did_invalid_base64() {
    let did = Did::new("did:key:z6Mk!!!not-valid-base64url!!!");
    assert!(super::braid_verify::extract_public_key_from_did(&did).is_none());
}

#[test]
fn test_extract_public_key_from_did_missing_prefix() {
    let did = Did::new("did:key:zOtherEncoding");
    assert!(super::braid_verify::extract_public_key_from_did(&did).is_none());
}
