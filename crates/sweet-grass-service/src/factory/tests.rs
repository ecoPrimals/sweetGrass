// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Tests for storage backend factory.

#![expect(
    clippy::unwrap_used,
    reason = "test module: unwrap is standard in tests"
)]

use std::collections::HashMap;

use super::*;

use crate::backend::BraidBackend;

fn mock_reader(vars: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> + use<> {
    let map: HashMap<String, String> = vars
        .iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect();
    move |key: &str| map.get(key).cloned()
}

fn empty_reader() -> impl Fn(&str) -> Option<String> {
    |_: &str| None
}

// Memory Backend Tests

#[tokio::test]
async fn test_memory_backend() {
    let reader = mock_reader(&[("STORAGE_BACKEND", "memory")]);
    let store = BraidStoreFactory::from_reader_with_name(reader).await;
    assert!(store.is_ok());
}

#[tokio::test]
async fn test_default_backend() {
    let store = BraidStoreFactory::from_reader_with_name(empty_reader()).await;
    assert!(store.is_ok(), "Should default to memory backend");
}

#[tokio::test]
async fn test_memory_backend_explicit() {
    let reader = mock_reader(&[("STORAGE_BACKEND", "memory")]);
    let result = BraidStoreFactory::from_reader_with_name(reader).await;
    assert!(result.is_ok());
    let (store, _) = result.unwrap();
    assert!(matches!(store, BraidBackend::Memory(_)));
}

// Error Cases

#[tokio::test]
async fn test_unknown_backend() {
    let reader = mock_reader(&[("STORAGE_BACKEND", "unknown")]);
    let result = BraidStoreFactory::from_reader_with_name(reader).await;
    assert!(result.is_err());
    if let Err(err) = result {
        let msg = err.to_string();
        assert!(msg.contains("Unknown storage backend"), "Error was: {msg}");
    }
}

#[tokio::test]
async fn test_unknown_backend_specific_message() {
    let reader = mock_reader(&[("STORAGE_BACKEND", "unknown_backend")]);
    let result = BraidStoreFactory::from_reader_with_name(reader).await;
    assert!(result.is_err());
    if let Err(err) = result {
        let msg = err.to_string();
        assert!(msg.contains("Unknown storage backend"));
        assert!(msg.contains("unknown_backend"));
        assert!(msg.contains("memory, redb"));
    }
}

// ==================== Config-based factory ====================

#[tokio::test]
async fn test_from_config_memory() {
    let config = StorageConfig {
        backend: "memory".to_string(),
        ..StorageConfig::default()
    };
    let store = BraidStoreFactory::from_config(&config).await;
    assert!(store.is_ok());
}

#[tokio::test]
async fn test_from_config_empty_backend_defaults_to_memory() {
    let config = StorageConfig::default();
    let (store, name) = BraidStoreFactory::from_config_with_name(&config)
        .await
        .unwrap();
    assert_eq!(name, "memory");
    assert!(matches!(store, BraidBackend::Memory(_)));
}

#[tokio::test]
async fn test_from_config_unknown_backend() {
    let config = StorageConfig {
        backend: "redis".to_string(),
        ..StorageConfig::default()
    };
    let result = BraidStoreFactory::from_config(&config).await;
    assert!(result.is_err());
    if let Err(err) = result {
        let msg = err.to_string();
        assert!(msg.contains("Unknown storage backend"));
        assert!(msg.contains("redis"));
    }
}

#[tokio::test]
async fn test_from_config_with_name_memory() {
    let config = StorageConfig {
        backend: "memory".to_string(),
        ..StorageConfig::default()
    };
    let (store, name) = BraidStoreFactory::from_config_with_name(&config)
        .await
        .unwrap();
    assert_eq!(name, "memory");
    assert!(matches!(store, BraidBackend::Memory(_)));
}

// ==================== redb Backend Tests ====================

#[tokio::test]
async fn test_from_config_redb() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.redb").to_str().unwrap().to_string();
    let config = StorageConfig {
        backend: "redb".to_string(),
        redb_path: Some(db_path),
        ..StorageConfig::default()
    };
    let (store, name) = BraidStoreFactory::from_config_with_name(&config)
        .await
        .unwrap();
    assert_eq!(name, "redb");
    assert!(matches!(store, BraidBackend::Redb(_)));
}

#[tokio::test]
async fn test_from_config_redb_default_path() {
    let config = StorageConfig {
        backend: "redb".to_string(),
        ..StorageConfig::default()
    };
    let result = BraidStoreFactory::from_config_with_name(&config).await;
    assert!(result.is_ok());
    let (_, name) = result.unwrap();
    assert_eq!(name, "redb");
    let _ = std::fs::remove_file("./data/sweetgrass.redb");
    let _ = std::fs::remove_dir("./data");
}

#[tokio::test]
async fn test_redb_backend_from_reader() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("env.redb");
    let path_str = db_path.to_str().unwrap().to_string();
    let reader = mock_reader(&[("STORAGE_BACKEND", "redb"), ("STORAGE_PATH", &path_str)]);
    let result = BraidStoreFactory::from_reader_with_name(reader).await;
    assert!(result.is_ok());
    let (_, name) = result.unwrap();
    assert_eq!(name, "redb");
}

#[test]
fn test_redb_config_default_path_via_reader() {
    let config =
        BraidStoreFactory::config_from_reader(&mock_reader(&[("STORAGE_BACKEND", "redb")]));
    assert!(config.redb_path.is_none());
}

#[test]
fn test_redb_config_custom_path_via_reader() {
    let reader = mock_reader(&[
        ("STORAGE_BACKEND", "redb"),
        ("STORAGE_PATH", "/tmp/custom.redb"),
    ]);
    let config = BraidStoreFactory::config_from_reader(&reader);
    assert_eq!(config.redb_path.as_deref(), Some("/tmp/custom.redb"));
}

// ==================== NestGate Backend Tests ====================

#[cfg(feature = "nestgate")]
#[tokio::test]
async fn test_from_config_nestgate() {
    let config = StorageConfig {
        backend: "nestgate".to_string(),
        nestgate_socket: Some("/tmp/test-nestgate-factory.sock".to_string()),
        nestgate_family_id: Some("test-family".to_string()),
        ..StorageConfig::default()
    };
    let result = BraidStoreFactory::from_config_with_name(&config).await;
    assert!(result.is_ok());
    let (store, name) = result.unwrap();
    assert_eq!(name, "nestgate");
    assert!(matches!(store, BraidBackend::NestGate(_)));
}

#[cfg(feature = "nestgate")]
#[tokio::test]
async fn test_from_config_nestgate_via_reader() {
    let reader = mock_reader(&[
        ("STORAGE_BACKEND", "nestgate"),
        ("NESTGATE_SOCKET", "/tmp/test-ng-reader.sock"),
    ]);
    let result = BraidStoreFactory::from_reader_with_name(reader).await;
    assert!(result.is_ok());
    let (_, name) = result.unwrap();
    assert_eq!(name, "nestgate");
}

#[cfg(feature = "nestgate")]
#[test]
fn test_nestgate_config_from_reader() {
    let reader = mock_reader(&[
        ("STORAGE_BACKEND", "nestgate"),
        ("NESTGATE_SOCKET", "/custom/nestgate.sock"),
        ("FAMILY_ID", "fam-001"),
    ]);
    let config = BraidStoreFactory::config_from_reader(&reader);
    assert_eq!(config.backend, "nestgate");
    assert_eq!(
        config.nestgate_socket.as_deref(),
        Some("/custom/nestgate.sock")
    );
    assert_eq!(config.nestgate_family_id.as_deref(), Some("fam-001"));
}

// ==================== StorageConfig defaults ====================

#[test]
fn test_storage_config_default() {
    let config = StorageConfig::default();
    assert!(config.backend.is_empty());
    assert!(config.redb_path.is_none());
}

#[test]
fn test_storage_config_clone() {
    let original = StorageConfig {
        backend: "redb".to_string(),
        redb_path: Some("/tmp/test.redb".to_string()),
        ..StorageConfig::default()
    };
    let cloned = original.clone();
    assert_eq!(cloned.backend, "redb");
    assert_eq!(cloned.redb_path, Some("/tmp/test.redb".to_string()));
    assert_eq!(original.backend, cloned.backend);
}
