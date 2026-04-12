// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test module: expect/unwrap are standard in tests"
)]

use std::path::PathBuf;
use std::sync::Arc;

use sweet_grass_core::SelfKnowledge;
use sweet_grass_core::agent::Did;
use sweet_grass_core::primal_names::env_vars;
use sweet_grass_store::{BraidStore, MemoryStore};

use super::*;

// ==================== DI-based socket resolution tests ====================

#[test]
fn di_explicit_socket_override() {
    let config = SocketConfig {
        explicit_socket: Some("/custom/path.sock".to_string()),
        biomeos_socket_dir: Some("/run/biomeos".to_string()),
        ..Default::default()
    };
    assert_eq!(
        resolve_socket_path_with(&config),
        PathBuf::from("/custom/path.sock")
    );
}

#[test]
fn di_biomeos_dir() {
    let config = SocketConfig {
        biomeos_socket_dir: Some("/run/biomeos".to_string()),
        ..Default::default()
    };
    assert_eq!(
        resolve_socket_path_with(&config),
        PathBuf::from("/run/biomeos/sweetgrass.sock")
    );
}

#[test]
fn di_biomeos_dir_with_family() {
    let config = SocketConfig {
        biomeos_socket_dir: Some("/run/biomeos".to_string()),
        family_id: Some("alpha".to_string()),
        ..Default::default()
    };
    assert_eq!(
        resolve_socket_path_with(&config),
        PathBuf::from("/run/biomeos/sweetgrass-alpha.sock")
    );
}

#[test]
fn di_xdg_runtime() {
    let config = SocketConfig {
        xdg_runtime_dir: Some("/run/user/1000".to_string()),
        ..Default::default()
    };
    assert_eq!(
        resolve_socket_path_with(&config),
        PathBuf::from("/run/user/1000/biomeos/sweetgrass.sock")
    );
}

#[test]
fn di_user_fallback() {
    let config = SocketConfig {
        user: Some("testuser".to_string()),
        ..Default::default()
    };
    let expected = std::env::temp_dir()
        .join("biomeos-testuser")
        .join("sweetgrass.sock");
    assert_eq!(resolve_socket_path_with(&config), expected);
}

#[test]
fn di_temp_fallback() {
    let config = SocketConfig::default();
    let expected = std::env::temp_dir().join("sweetgrass.sock");
    assert_eq!(resolve_socket_path_with(&config), expected);
}

#[test]
fn di_custom_primal_name() {
    let config = SocketConfig {
        biomeos_socket_dir: Some("/run/biomeos".to_string()),
        primal_name: Some("sweetgrass-prod".to_string()),
        ..Default::default()
    };
    assert_eq!(
        resolve_socket_path_with(&config),
        PathBuf::from("/run/biomeos/sweetgrass-prod.sock")
    );
}

#[test]
fn di_family_id_in_temp_fallback() {
    let config = SocketConfig {
        family_id: Some("beta".to_string()),
        ..Default::default()
    };
    let expected = std::env::temp_dir().join("sweetgrass-beta.sock");
    assert_eq!(resolve_socket_path_with(&config), expected);
}

#[test]
fn di_priority_explicit_overrides_all() {
    let config = SocketConfig {
        explicit_socket: Some("/absolute/custom.sock".to_string()),
        biomeos_socket_dir: Some("/run/biomeos".to_string()),
        xdg_runtime_dir: Some("/run/user/1000".to_string()),
        user: Some("testuser".to_string()),
        ..Default::default()
    };
    assert_eq!(
        resolve_socket_path_with(&config),
        PathBuf::from("/absolute/custom.sock")
    );
}

// ==================== Cleanup tests ====================

#[test]
fn test_cleanup_socket_when_exists() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("cleanup-test.sock");
    std::fs::write(&sock_path, "").expect("create socket file");
    assert!(sock_path.exists());
    cleanup_socket_at(&sock_path);
    assert!(!sock_path.exists());
}

#[test]
fn test_cleanup_socket_nonexistent() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("nonexistent.sock");
    cleanup_socket_at(&sock_path);
}

// ==================== Capability symlink tests ====================

#[test]
fn test_create_capability_symlink() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("sweetgrass.sock");
    std::fs::write(&sock_path, "").expect("create socket file");

    create_capability_symlink(&sock_path);

    let symlink_path = dir.path().join("provenance.sock");
    assert!(symlink_path.is_symlink(), "symlink should exist");
    let target = std::fs::read_link(&symlink_path).expect("read symlink");
    assert_eq!(
        target,
        std::path::PathBuf::from("sweetgrass.sock"),
        "symlink should be relative"
    );
}

#[test]
fn test_create_capability_symlink_with_family() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("sweetgrass-alpha.sock");
    std::fs::write(&sock_path, "").expect("create socket file");

    create_capability_symlink(&sock_path);

    let symlink_path = dir.path().join("provenance.sock");
    assert!(symlink_path.is_symlink());
    let target = std::fs::read_link(&symlink_path).expect("read symlink");
    assert_eq!(target, std::path::PathBuf::from("sweetgrass-alpha.sock"));
}

#[test]
fn test_create_capability_symlink_replaces_stale() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("sweetgrass.sock");
    std::fs::write(&sock_path, "").expect("create socket file");

    let symlink_path = dir.path().join("provenance.sock");
    std::os::unix::fs::symlink("old-target.sock", &symlink_path).expect("create stale");

    create_capability_symlink(&sock_path);

    let target = std::fs::read_link(&symlink_path).expect("read symlink");
    assert_eq!(target, std::path::PathBuf::from("sweetgrass.sock"));
}

#[test]
fn test_cleanup_capability_symlink() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("sweetgrass.sock");
    std::fs::write(&sock_path, "").expect("create socket file");

    create_capability_symlink(&sock_path);
    let symlink_path = dir.path().join("provenance.sock");
    assert!(symlink_path.is_symlink());

    cleanup_capability_symlink(&sock_path);
    assert!(!symlink_path.exists());
    assert!(!symlink_path.is_symlink());
}

#[test]
fn test_cleanup_socket_at_removes_symlink_too() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("sweetgrass.sock");
    std::fs::write(&sock_path, "").expect("create socket file");

    create_capability_symlink(&sock_path);
    let symlink_path = dir.path().join("provenance.sock");
    assert!(symlink_path.is_symlink());
    assert!(sock_path.exists());

    cleanup_socket_at(&sock_path);
    assert!(!sock_path.exists());
    assert!(!symlink_path.exists());
}

#[test]
fn test_cleanup_capability_symlink_nonexistent() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("sweetgrass.sock");
    cleanup_capability_symlink(&sock_path);
}

// ==================== BTSP insecure guard tests ====================

#[test]
fn guard_passes_no_family_no_insecure() {
    assert!(validate_insecure_guard_with(None, false).is_ok());
}

#[test]
fn guard_passes_family_set_insecure_off() {
    assert!(validate_insecure_guard_with(Some("alpha"), false).is_ok());
}

#[test]
fn guard_passes_insecure_on_no_family() {
    assert!(validate_insecure_guard_with(None, true).is_ok());
}

#[test]
fn guard_fails_family_and_insecure() {
    let err = validate_insecure_guard_with(Some("alpha"), true).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("alpha"), "error should mention family: {msg}");
    assert!(msg.contains("BTSP"), "error should reference BTSP: {msg}");
    assert!(
        msg.contains("BIOMEOS_INSECURE"),
        "error should mention BIOMEOS_INSECURE: {msg}"
    );
}

#[test]
fn guard_error_display_is_descriptive() {
    let err = BtspGuardViolation {
        family_id: "myFamily42".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("myFamily42"));
    assert!(msg.contains("mutually exclusive"));
}

// ==================== UDS roundtrip tests ====================

#[tokio::test]
async fn test_uds_roundtrip() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("test-sweetgrass.sock");

    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkTest"));

    let path_clone = sock_path.clone();
    let state_clone = state.clone();
    let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let listener_handle = tokio::spawn(async move {
        let _ = start_uds_listener_at(state_clone, &path_clone, shutdown_rx).await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let stream = tokio::net::UnixStream::connect(&sock_path)
        .await
        .expect("connect");
    let (reader, mut writer) = stream.into_split();

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "health.check",
        "params": {},
        "id": 1
    });
    let mut req_str = serde_json::to_string(&request).unwrap();
    req_str.push('\n');
    writer.write_all(req_str.as_bytes()).await.unwrap();

    let mut lines = BufReader::new(reader).lines();
    let response_line = lines.next_line().await.unwrap().expect("response");
    let response: serde_json::Value = serde_json::from_str(&response_line).expect("parse response");

    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["result"].is_object());
    assert_eq!(response["result"]["status"], "healthy");

    listener_handle.abort();
}

#[tokio::test]
async fn test_uds_parse_error_returns_jsonrpc_error() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("parse-error-test.sock");

    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkTest"));

    let path_clone = sock_path.clone();
    let state_clone = state.clone();
    let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let listener_handle = tokio::spawn(async move {
        let _ = start_uds_listener_at(state_clone, &path_clone, shutdown_rx).await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let stream = tokio::net::UnixStream::connect(&sock_path)
        .await
        .expect("connect");
    let (reader, mut writer) = stream.into_split();

    writer
        .write_all(b"{ invalid json }\n")
        .await
        .expect("write");
    writer.flush().await.expect("flush");

    let mut lines = BufReader::new(reader).lines();
    let response_line = lines.next_line().await.unwrap().expect("response");
    let response: serde_json::Value = serde_json::from_str(&response_line).expect("parse response");

    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["error"].is_object());
    assert_eq!(
        response["error"]["code"],
        crate::handlers::jsonrpc::error_code::PARSE_ERROR
    );

    listener_handle.abort();
}

#[tokio::test]
async fn test_uds_empty_lines_skipped() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("empty-lines-test.sock");

    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkTest"));

    let path_clone = sock_path.clone();
    let state_clone = state.clone();
    let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let listener_handle = tokio::spawn(async move {
        let _ = start_uds_listener_at(state_clone, &path_clone, shutdown_rx).await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let stream = tokio::net::UnixStream::connect(&sock_path)
        .await
        .expect("connect");
    let (reader, mut writer) = stream.into_split();

    writer.write_all(b"\n").await.expect("write");
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "health.check",
        "params": {},
        "id": 2
    });
    let mut req_str = serde_json::to_string(&request).unwrap();
    req_str.push('\n');
    writer.write_all(req_str.as_bytes()).await.unwrap();
    writer.flush().await.expect("flush");

    let mut lines = BufReader::new(reader).lines();
    let response_line = lines.next_line().await.unwrap().expect("response");
    let response: serde_json::Value = serde_json::from_str(&response_line).expect("parse response");

    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["result"].is_object());

    listener_handle.abort();
}

// ==================== Concurrent load test (trio IPC hardening) ====================

#[tokio::test]
async fn test_uds_concurrent_clients() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("concurrent-test.sock");

    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkConcurrent"));

    let path_clone = sock_path.clone();
    let state_clone = state.clone();
    let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let listener_handle = tokio::spawn(async move {
        let _ = start_uds_listener_at(state_clone, &path_clone, shutdown_rx).await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let num_clients = 8;
    let requests_per_client = 5;
    let mut handles = Vec::with_capacity(num_clients);

    for client_id in 0..num_clients {
        let path = sock_path.clone();
        handles.push(tokio::spawn(async move {
            let stream = tokio::net::UnixStream::connect(&path)
                .await
                .expect("connect");
            let (reader, mut writer) = stream.into_split();
            let mut lines = BufReader::new(reader).lines();

            for req_id in 0..requests_per_client {
                let id = client_id * 100 + req_id;
                let request = serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "health.liveness",
                    "params": {},
                    "id": id
                });
                let mut req_str = serde_json::to_string(&request).unwrap();
                req_str.push('\n');
                writer.write_all(req_str.as_bytes()).await.unwrap();
                writer.flush().await.unwrap();

                let response_line = lines.next_line().await.unwrap().expect("response");
                let response: serde_json::Value =
                    serde_json::from_str(&response_line).expect("parse");
                assert_eq!(response["jsonrpc"], "2.0");
                assert_eq!(response["id"], id);
            }
        }));
    }

    for handle in handles {
        handle.await.expect("client task should succeed");
    }

    listener_handle.abort();
}

// ==================== Additional UDS coverage (uds.rs paths) ====================

#[tokio::test]
async fn test_uds_notification_no_response_then_request_ok() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("notification-test.sock");

    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkNotify"));

    let path_clone = sock_path.clone();
    let state_clone = state.clone();
    let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let listener_handle = tokio::spawn(async move {
        let _ = start_uds_listener_at(state_clone, &path_clone, shutdown_rx).await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let stream = tokio::net::UnixStream::connect(&sock_path)
        .await
        .expect("connect");
    let (reader, mut writer) = stream.into_split();

    let notification = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "health.check",
        "params": {},
    });
    let mut note_str = serde_json::to_string(&notification).unwrap();
    note_str.push('\n');
    writer.write_all(note_str.as_bytes()).await.unwrap();

    let follow_up = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "health.check",
        "params": {},
        "id": 77
    });
    let mut follow_str = serde_json::to_string(&follow_up).unwrap();
    follow_str.push('\n');
    writer.write_all(follow_str.as_bytes()).await.unwrap();
    writer.flush().await.expect("flush");

    let mut lines = BufReader::new(reader).lines();
    let response_line = lines
        .next_line()
        .await
        .unwrap()
        .expect("response after notification");
    let response: serde_json::Value = serde_json::from_str(&response_line).expect("parse response");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 77);
    assert_eq!(response["result"]["status"], "healthy");

    listener_handle.abort();
}

#[tokio::test]
async fn test_uds_sequential_methods_on_one_connection() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("sequential-test.sock");

    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkSequential"));

    let path_clone = sock_path.clone();
    let state_clone = state.clone();
    let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let listener_handle = tokio::spawn(async move {
        let _ = start_uds_listener_at(state_clone, &path_clone, shutdown_rx).await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let stream = tokio::net::UnixStream::connect(&sock_path)
        .await
        .expect("connect");
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    let steps = [
        (1_i64, "health.check", "status"),
        (2_i64, "health.liveness", "alive"),
        (3_i64, "capabilities.list", "capabilities"),
    ];

    for (id, method, key) in steps {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": {},
            "id": id
        });
        let mut req_str = serde_json::to_string(&request).unwrap();
        req_str.push('\n');
        writer.write_all(req_str.as_bytes()).await.unwrap();
        writer.flush().await.unwrap();

        let response_line = lines.next_line().await.unwrap().expect("response line");
        let response: serde_json::Value =
            serde_json::from_str(&response_line).expect("parse response");

        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], id);
        assert!(
            response["result"].get(key).is_some(),
            "result should contain {key}: {response}"
        );
    }

    listener_handle.abort();
}

#[tokio::test]
async fn test_uds_listener_graceful_shutdown() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("shutdown-test.sock");

    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkShutdown"));

    let path_clone = sock_path.clone();
    let state_clone = state.clone();
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let listener_handle =
        tokio::spawn(
            async move { start_uds_listener_at(state_clone, &path_clone, shutdown_rx).await },
        );

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    shutdown_tx.send(true).expect("signal shutdown");

    let finished = tokio::time::timeout(std::time::Duration::from_secs(3), listener_handle)
        .await
        .expect("listener should exit within timeout");

    let join_result = finished.expect("listener task join");
    assert!(
        join_result.is_ok(),
        "listener should exit cleanly: {join_result:?}"
    );
}

#[test]
fn test_start_uds_listener_resolves_path_from_self_knowledge() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let dir = tempfile::tempdir().expect("tempdir");
    let biome_dir = dir.path().to_string_lossy().into_owned();
    let primal_name = "uds-from-self-knowledge";
    let sock_path = dir.path().join(format!("{primal_name}.sock"));

    let sk = SelfKnowledge {
        name: primal_name.to_string(),
        ..Default::default()
    };
    let store: Arc<dyn BraidStore> = Arc::new(MemoryStore::new());
    let state = crate::state::AppState::with_self_knowledge(
        store,
        Did::new("did:key:z6MkUdsSelfKnowledge"),
        sk,
        "memory",
    );

    temp_env::with_vars(
        [
            ("SWEETGRASS_SOCKET", None::<&str>),
            ("FAMILY_ID", None::<&str>),
            (env_vars::SWEETGRASS_FAMILY_ID, None::<&str>),
            (env_vars::BIOMEOS_FAMILY_ID, None::<&str>),
            (env_vars::BIOMEOS_SOCKET_DIR, Some(biome_dir.as_str())),
        ],
        || {
            assert_eq!(
                resolve_socket_path(Some(primal_name)),
                sock_path,
                "resolve_socket_path must match start_uds_listener bind path"
            );

            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("runtime");

            rt.block_on(async {
                let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
                let state_clone = state.clone();
                let listener_handle =
                    tokio::spawn(async move { start_uds_listener(state_clone, shutdown_rx).await });

                tokio::time::sleep(std::time::Duration::from_millis(50)).await;

                let stream = tokio::net::UnixStream::connect(&sock_path)
                    .await
                    .expect("connect");
                let (reader, mut writer) = stream.into_split();

                let request = serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "health.check",
                    "params": {},
                    "id": 1
                });
                let mut req_str = serde_json::to_string(&request).unwrap();
                req_str.push('\n');
                writer.write_all(req_str.as_bytes()).await.unwrap();
                writer.flush().await.expect("flush");

                let mut lines = BufReader::new(reader).lines();
                let response_line = lines.next_line().await.unwrap().expect("response");
                let response: serde_json::Value =
                    serde_json::from_str(&response_line).expect("parse response");
                assert_eq!(response["result"]["status"], "healthy");

                shutdown_tx.send(true).expect("shutdown");
                let _ = tokio::time::timeout(std::time::Duration::from_secs(3), listener_handle)
                    .await
                    .expect("timeout")
                    .expect("join");
            });
        },
    );
}

#[tokio::test]
async fn test_uds_stale_socket_file_removed_before_bind() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("stale-socket-test.sock");

    std::fs::write(&sock_path, b"not-a-socket").expect("stale file");

    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkStaleSock"));

    let path_clone = sock_path.clone();
    let state_clone = state.clone();
    let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let listener_handle = tokio::spawn(async move {
        let _ = start_uds_listener_at(state_clone, &path_clone, shutdown_rx).await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let stream = tokio::net::UnixStream::connect(&sock_path)
        .await
        .expect("connect");
    let (reader, mut writer) = stream.into_split();

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "health.liveness",
        "params": {},
        "id": 42
    });
    let mut req_str = serde_json::to_string(&request).unwrap();
    req_str.push('\n');
    writer.write_all(req_str.as_bytes()).await.unwrap();
    writer.flush().await.expect("flush");

    let mut lines = BufReader::new(reader).lines();
    let response_line = lines.next_line().await.unwrap().expect("response");
    let response: serde_json::Value = serde_json::from_str(&response_line).expect("parse response");
    assert_eq!(response["id"], 42);
    assert_eq!(response["result"]["alive"], true);

    listener_handle.abort();
}

#[tokio::test]
async fn test_uds_listener_creates_parent_directories() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("nested/deep/run.sock");

    assert!(!sock_path.parent().expect("parent").exists());

    let state = crate::state::AppState::new_memory(Did::new("did:key:z6MkMkdir"));

    let path_clone = sock_path.clone();
    let state_clone = state.clone();
    let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let listener_handle = tokio::spawn(async move {
        let _ = start_uds_listener_at(state_clone, &path_clone, shutdown_rx).await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    assert!(sock_path.exists());

    let stream = tokio::net::UnixStream::connect(&sock_path)
        .await
        .expect("connect");
    let (reader, mut writer) = stream.into_split();

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "health.check",
        "params": {},
        "id": 0
    });
    let mut req_str = serde_json::to_string(&request).unwrap();
    req_str.push('\n');
    writer.write_all(req_str.as_bytes()).await.unwrap();
    writer.flush().await.expect("flush");

    let mut lines = BufReader::new(reader).lines();
    let response_line = lines.next_line().await.unwrap().expect("response");
    let response: serde_json::Value = serde_json::from_str(&response_line).expect("parse response");
    assert_eq!(response["result"]["status"], "healthy");

    listener_handle.abort();
}

// ==================== Env-reading wrapper coverage ====================

#[test]
fn resolve_family_id_sweetgrass_override() {
    temp_env::with_vars(
        [
            (env_vars::SWEETGRASS_FAMILY_ID, Some("sweet-fam")),
            (env_vars::BIOMEOS_FAMILY_ID, Some("biome-fam")),
            (env_vars::FAMILY_ID, Some("generic-fam")),
        ],
        || {
            assert_eq!(resolve_family_id_from_env(), Some("sweet-fam".to_string()));
        },
    );
}

#[test]
fn resolve_family_id_biomeos_fallback() {
    temp_env::with_vars(
        [
            (env_vars::SWEETGRASS_FAMILY_ID, None::<&str>),
            (env_vars::BIOMEOS_FAMILY_ID, Some("biome-fam")),
            (env_vars::FAMILY_ID, Some("generic-fam")),
        ],
        || {
            assert_eq!(resolve_family_id_from_env(), Some("biome-fam".to_string()));
        },
    );
}

#[test]
fn resolve_family_id_generic_fallback() {
    temp_env::with_vars(
        [
            (env_vars::SWEETGRASS_FAMILY_ID, None::<&str>),
            (env_vars::BIOMEOS_FAMILY_ID, None::<&str>),
            (env_vars::FAMILY_ID, Some("generic-fam")),
        ],
        || {
            assert_eq!(
                resolve_family_id_from_env(),
                Some("generic-fam".to_string())
            );
        },
    );
}

#[test]
fn resolve_family_id_none_when_all_absent() {
    temp_env::with_vars(
        [
            (env_vars::SWEETGRASS_FAMILY_ID, None::<&str>),
            (env_vars::BIOMEOS_FAMILY_ID, None::<&str>),
            (env_vars::FAMILY_ID, None::<&str>),
        ],
        || {
            assert_eq!(resolve_family_id_from_env(), None);
        },
    );
}

#[test]
fn resolve_family_id_filters_empty_string() {
    temp_env::with_vars(
        [
            (env_vars::SWEETGRASS_FAMILY_ID, Some("")),
            (env_vars::BIOMEOS_FAMILY_ID, None::<&str>),
            (env_vars::FAMILY_ID, None::<&str>),
        ],
        || {
            assert_eq!(resolve_family_id_from_env(), None);
        },
    );
}

#[test]
fn resolve_family_id_filters_default_string() {
    temp_env::with_vars(
        [
            (env_vars::SWEETGRASS_FAMILY_ID, Some("default")),
            (env_vars::BIOMEOS_FAMILY_ID, None::<&str>),
            (env_vars::FAMILY_ID, None::<&str>),
        ],
        || {
            assert_eq!(resolve_family_id_from_env(), None);
        },
    );
}

#[test]
fn validate_insecure_guard_env_passes_when_no_family() {
    temp_env::with_vars(
        [
            (env_vars::SWEETGRASS_FAMILY_ID, None::<&str>),
            (env_vars::BIOMEOS_FAMILY_ID, None::<&str>),
            (env_vars::FAMILY_ID, None::<&str>),
            (env_vars::BIOMEOS_INSECURE, Some("1")),
        ],
        || {
            assert!(validate_insecure_guard().is_ok());
        },
    );
}

#[test]
fn validate_insecure_guard_env_fails_when_family_and_insecure() {
    temp_env::with_vars(
        [
            (env_vars::SWEETGRASS_FAMILY_ID, None::<&str>),
            (env_vars::BIOMEOS_FAMILY_ID, None::<&str>),
            (env_vars::FAMILY_ID, Some("test-family")),
            (env_vars::BIOMEOS_INSECURE, Some("1")),
        ],
        || {
            assert!(validate_insecure_guard().is_err());
        },
    );
}

#[test]
fn validate_insecure_guard_env_passes_when_family_no_insecure() {
    temp_env::with_vars(
        [
            (env_vars::SWEETGRASS_FAMILY_ID, None::<&str>),
            (env_vars::BIOMEOS_FAMILY_ID, None::<&str>),
            (env_vars::FAMILY_ID, Some("test-family")),
            (env_vars::BIOMEOS_INSECURE, None::<&str>),
        ],
        || {
            assert!(validate_insecure_guard().is_ok());
        },
    );
}

#[test]
fn resolve_socket_path_env_reads_biomeos_dir() {
    temp_env::with_vars(
        [
            ("SWEETGRASS_SOCKET", None::<&str>),
            (env_vars::BIOMEOS_SOCKET_DIR, Some("/run/biomeos-env")),
            (env_vars::SWEETGRASS_FAMILY_ID, None::<&str>),
            (env_vars::BIOMEOS_FAMILY_ID, None::<&str>),
            (env_vars::FAMILY_ID, None::<&str>),
        ],
        || {
            let path = resolve_socket_path(Some("myprimal"));
            assert_eq!(path, PathBuf::from("/run/biomeos-env/myprimal.sock"));
        },
    );
}

#[test]
fn cleanup_socket_resolves_and_cleans() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("sweetgrass.sock");
    std::fs::write(&sock_path, "").expect("create");

    temp_env::with_vars(
        [
            ("SWEETGRASS_SOCKET", Some(sock_path.to_str().unwrap())),
            (env_vars::SWEETGRASS_FAMILY_ID, None::<&str>),
            (env_vars::BIOMEOS_FAMILY_ID, None::<&str>),
            (env_vars::FAMILY_ID, None::<&str>),
        ],
        || {
            cleanup_socket();
            assert!(!sock_path.exists());
        },
    );
}
