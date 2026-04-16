// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

mod queries;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use serde_json::{Value, json};
use sweet_grass_core::Braid;
use sweet_grass_core::agent::Did;
use sweet_grass_core::entity::EntityReference;
use sweet_grass_store::{BraidStore, QueryFilter};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use super::NestGateStore;
use crate::NestGateConfig;

struct MockNestGate {
    data: Arc<Mutex<HashMap<String, Value>>>,
    socket_path: PathBuf,
    _dir: tempfile::TempDir,
}

impl MockNestGate {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let socket_path = dir.path().join("nestgate.sock");
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
            socket_path,
            _dir: dir,
        }
    }

    fn socket_path(&self) -> &std::path::Path {
        &self.socket_path
    }

    async fn run(&self) {
        let listener = UnixListener::bind(&self.socket_path).expect("bind mock nestgate");
        let data = Arc::clone(&self.data);

        loop {
            let Ok((stream, _)) = listener.accept().await else {
                break;
            };
            let data = Arc::clone(&data);
            tokio::spawn(async move {
                let (reader, mut writer) = stream.into_split();
                let mut buf = BufReader::new(reader);
                let mut line = String::new();
                if buf.read_line(&mut line).await.is_err() || line.is_empty() {
                    return;
                }

                let request: Value = match serde_json::from_str(&line) {
                    Ok(v) => v,
                    Err(_) => return,
                };

                let method = request["method"].as_str().unwrap_or("");
                let params = &request["params"];
                let id = &request["id"];

                let result = match method {
                    "storage.store" => {
                        let key = params["key"].as_str().unwrap_or("").to_string();
                        let value = params["value"].clone();
                        data.lock().await.insert(key, value);
                        json!({"stored": true})
                    },
                    "storage.retrieve" => {
                        let key = params["key"].as_str().unwrap_or("");
                        let value = data.lock().await.get(key).cloned();
                        value.map_or_else(|| json!({"value": null}), |v| json!({"value": v}))
                    },
                    "storage.delete" => {
                        let key = params["key"].as_str().unwrap_or("").to_string();
                        data.lock().await.remove(&key);
                        json!({"deleted": true})
                    },
                    "storage.exists" => {
                        let key = params["key"].as_str().unwrap_or("");
                        let exists = data.lock().await.contains_key(key);
                        json!({"exists": exists})
                    },
                    "storage.list" => {
                        let prefix = params["prefix"].as_str().unwrap_or("");
                        let keys: Vec<String> = data
                            .lock()
                            .await
                            .keys()
                            .filter(|k| k.starts_with(prefix))
                            .cloned()
                            .collect();
                        json!({"keys": keys})
                    },
                    _ => json!(null),
                };

                let response = json!({
                    "jsonrpc": "2.0",
                    "result": result,
                    "id": id,
                });

                let mut resp_line = serde_json::to_string(&response).unwrap();
                resp_line.push('\n');
                let _ = writer.write_all(resp_line.as_bytes()).await;
                let _ = writer.flush().await;
            });
        }
    }

    fn create_store(&self) -> NestGateStore {
        let config = NestGateConfig {
            socket_path: Some(self.socket_path().to_string_lossy().to_string()),
            family_id: None,
            key_prefix: "sg".to_string(),
        };
        NestGateStore::new(&config).expect("create store")
    }
}

fn test_braid() -> Braid {
    sweet_grass_factory::BraidFactory::new(Did::new("did:key:z6MkNestTest"))
        .from_hash(
            "sha256:nesttest001".to_string().into(),
            "text/plain".to_string(),
            256,
            None,
        )
        .expect("create test braid")
}

fn make_braid(hash: &str, agent: &str, mime: &str, size: u64) -> Braid {
    Braid::builder()
        .data_hash(hash)
        .mime_type(mime)
        .size(size)
        .attributed_to(Did::new(agent))
        .build()
        .expect("build braid")
}

async fn setup() -> (MockNestGate, NestGateStore, JoinHandle<()>) {
    let mock = MockNestGate::new();
    let handle = tokio::spawn({
        let mock_ref = MockNestGate {
            data: Arc::clone(&mock.data),
            socket_path: mock.socket_path().to_path_buf(),
            _dir: tempfile::tempdir().expect("unused"),
        };
        async move { mock_ref.run().await }
    });
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let store = mock.create_store();
    (mock, store, handle)
}

// ── Basic CRUD ──────────────────────────────────────────────────────

#[tokio::test]
async fn put_and_get_braid() {
    let (_mock, store, handle) = setup().await;
    let braid = test_braid();

    store.put(&braid).await.expect("put");
    let retrieved = store.get(&braid.id).await.expect("get");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, braid.id);

    handle.abort();
}

#[tokio::test]
async fn get_nonexistent_returns_none() {
    let (_mock, store, handle) = setup().await;
    let result = store
        .get(&sweet_grass_core::BraidId::new())
        .await
        .expect("get");
    assert!(result.is_none());
    handle.abort();
}

#[tokio::test]
async fn delete_braid() {
    let (_mock, store, handle) = setup().await;
    let braid = test_braid();

    store.put(&braid).await.expect("put");
    let deleted = store.delete(&braid.id).await.expect("delete");
    assert!(deleted);

    let after = store.get(&braid.id).await.expect("get after delete");
    assert!(after.is_none());
    handle.abort();
}

#[tokio::test]
async fn delete_nonexistent_returns_false() {
    let (_mock, store, handle) = setup().await;
    let deleted = store
        .delete(&sweet_grass_core::BraidId::new())
        .await
        .expect("delete nonexistent");
    assert!(!deleted);
    handle.abort();
}

#[tokio::test]
async fn exists_check() {
    let (_mock, store, handle) = setup().await;
    let braid = test_braid();

    assert!(!store.exists(&braid.id).await.expect("exists before put"));
    store.put(&braid).await.expect("put");
    assert!(store.exists(&braid.id).await.expect("exists after put"));
    handle.abort();
}

#[tokio::test]
async fn count_braids() {
    let (_mock, store, handle) = setup().await;
    let filter = QueryFilter::default();

    assert_eq!(store.count(&filter).await.expect("count empty"), 0);
    store.put(&test_braid()).await.expect("put");
    assert_eq!(store.count(&filter).await.expect("count one"), 1);
    handle.abort();
}

#[tokio::test]
async fn get_by_hash_finds_stored_braid() {
    let (_mock, store, handle) = setup().await;
    let braid = make_braid("sha256:byhash01", "did:key:z6MkA", "text/plain", 100);

    store.put(&braid).await.expect("put");
    let found = store
        .get_by_hash(&braid.data_hash)
        .await
        .expect("get_by_hash");
    assert!(found.is_some());
    assert_eq!(found.unwrap().data_hash, braid.data_hash);
    handle.abort();
}

#[tokio::test]
async fn get_by_hash_returns_none_for_missing() {
    let (_mock, store, handle) = setup().await;
    let hash = sweet_grass_core::ContentHash::new("sha256:nonexistent");
    let found = store.get_by_hash(&hash).await.expect("get_by_hash");
    assert!(found.is_none());
    handle.abort();
}

// ── Config, errors, client ──────────────────────────────────────────

#[tokio::test]
async fn connection_error_on_missing_socket() {
    let config = NestGateConfig {
        socket_path: Some("/tmp/nonexistent-nestgate-test.sock".to_string()),
        family_id: None,
        key_prefix: "sg".to_string(),
    };
    let store = NestGateStore::new(&config).expect("create store");

    let result = store.get(&sweet_grass_core::BraidId::new()).await;
    assert!(result.is_err(), "should fail on missing socket");
}

#[tokio::test]
async fn store_debug_display() {
    let config = NestGateConfig {
        socket_path: Some("/tmp/test-debug.sock".to_string()),
        family_id: None,
        key_prefix: "sg".to_string(),
    };
    let store = NestGateStore::new(&config).expect("create store");
    let debug_str = format!("{store:?}");
    assert!(debug_str.contains("NestGateStore"));
    assert!(debug_str.contains("test-debug.sock"));
}

#[tokio::test]
async fn new_with_reader_uses_discovery() {
    let config = NestGateConfig {
        socket_path: None,
        family_id: None,
        key_prefix: "sg".to_string(),
    };
    let store = NestGateStore::new_with_reader(&config, &|key| match key {
        "NESTGATE_SOCKET" => Some("/custom/nestgate.sock".to_string()),
        _ => None,
    })
    .expect("create store with reader");

    let debug_str = format!("{store:?}");
    assert!(debug_str.contains("/custom/nestgate.sock"));
}

#[test]
fn nestgate_config_default_values() {
    let config = NestGateConfig::default();
    assert!(config.socket_path.is_none());
    assert!(config.family_id.is_none());
    assert_eq!(config.key_prefix, "sweetgrass");
}

#[test]
fn nestgate_error_display_variants() {
    use crate::NestGateStoreError;

    let socket_err = NestGateStoreError::SocketNotFound("missing".into());
    assert!(socket_err.to_string().contains("missing"));

    let conn_err = NestGateStoreError::ConnectionFailed("refused".into());
    assert!(conn_err.to_string().contains("refused"));

    let rpc_err = NestGateStoreError::Rpc("timeout".into());
    assert!(rpc_err.to_string().contains("timeout"));

    let jsonrpc_err = NestGateStoreError::JsonRpcError {
        code: -32600,
        message: "Invalid Request".into(),
    };
    let display = jsonrpc_err.to_string();
    assert!(display.contains("-32600"));
    assert!(display.contains("Invalid Request"));

    let io_err = NestGateStoreError::Io(std::io::Error::new(
        std::io::ErrorKind::BrokenPipe,
        "pipe broke",
    ));
    assert!(io_err.to_string().contains("pipe broke"));
}

#[test]
fn nestgate_error_converts_to_store_error() {
    use sweet_grass_store::StoreError;

    let nestgate_err = crate::NestGateStoreError::Rpc("test error".into());
    let store_err: StoreError = nestgate_err.into();
    assert!(store_err.to_string().contains("test error"));
}

#[tokio::test]
async fn client_handles_jsonrpc_error_response() {
    let dir = tempfile::tempdir().expect("tempdir");
    let socket_path = dir.path().join("err.sock");
    let listener = UnixListener::bind(&socket_path).expect("bind");

    let handle = tokio::spawn(async move {
        let Ok((stream, _)) = listener.accept().await else {
            return;
        };
        let (reader, mut writer) = stream.into_split();
        let mut buf = BufReader::new(reader);
        let mut line = String::new();
        if buf.read_line(&mut line).await.is_err() || line.is_empty() {
            return;
        }
        let request: Value = serde_json::from_str(&line).unwrap();
        let id = &request["id"];
        let response = json!({
            "jsonrpc": "2.0",
            "error": {"code": -32601, "message": "Method not found"},
            "id": id,
        });
        let mut resp = serde_json::to_string(&response).unwrap();
        resp.push('\n');
        let _ = writer.write_all(resp.as_bytes()).await;
        let _ = writer.flush().await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let client = crate::client::NestGateClient::new(socket_path, None);
    let result = client.call("nonexistent.method", json!({})).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Method not found"));
    handle.abort();
}

#[tokio::test]
async fn client_handles_empty_response() {
    let dir = tempfile::tempdir().expect("tempdir");
    let socket_path = dir.path().join("empty.sock");
    let listener = UnixListener::bind(&socket_path).expect("bind");

    let handle = tokio::spawn(async move {
        let Ok((stream, _)) = listener.accept().await else {
            return;
        };
        drop(stream);
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let client = crate::client::NestGateClient::new(socket_path, None);
    let result = client.call("storage.retrieve", json!({})).await;
    assert!(
        result.is_err(),
        "should error on immediate close: {result:?}"
    );
    handle.abort();
}

// ── Index cleanup ───────────────────────────────────────────────────

#[tokio::test]
async fn delete_cleans_agent_index() {
    let (_mock, store, handle) = setup().await;
    let agent = Did::new("did:key:z6MkIdx");
    let braid = make_braid("sha256:idx01", agent.as_str(), "text/plain", 100);

    store.put(&braid).await.expect("put");
    assert_eq!(store.by_agent(&agent).await.expect("by_agent").len(), 1);

    store.delete(&braid.id).await.expect("delete");
    assert!(
        store
            .by_agent(&agent)
            .await
            .expect("by_agent after delete")
            .is_empty()
    );
    handle.abort();
}

#[tokio::test]
async fn delete_preserves_other_agent_index_entries() {
    let (_mock, store, handle) = setup().await;
    let agent = Did::new("did:key:z6MkShared");
    let b1 = make_braid("sha256:shared01", agent.as_str(), "text/plain", 100);
    let b2 = make_braid("sha256:shared02", agent.as_str(), "text/plain", 200);

    store.put(&b1).await.expect("put b1");
    store.put(&b2).await.expect("put b2");
    assert_eq!(store.by_agent(&agent).await.expect("by_agent").len(), 2);

    store.delete(&b1.id).await.expect("delete b1");
    let remaining = store.by_agent(&agent).await.expect("by_agent after delete");
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].id, b2.id);
    handle.abort();
}

#[tokio::test]
async fn delete_cleans_derived_index() {
    let (_mock, store, handle) = setup().await;
    let source_hash = sweet_grass_core::ContentHash::new("sha256:delsrc");

    let derived = Braid::builder()
        .data_hash("sha256:delderived01")
        .mime_type("text/plain")
        .size(100)
        .attributed_to(Did::new("did:key:z6MkA"))
        .derived_from(EntityReference::by_hash("sha256:delsrc"))
        .build()
        .expect("derived braid");

    store.put(&derived).await.expect("put");
    assert_eq!(
        store
            .derived_from(&source_hash)
            .await
            .expect("derived")
            .len(),
        1
    );

    store.delete(&derived.id).await.expect("delete");
    assert!(
        store
            .derived_from(&source_hash)
            .await
            .expect("derived after delete")
            .is_empty()
    );
    handle.abort();
}
