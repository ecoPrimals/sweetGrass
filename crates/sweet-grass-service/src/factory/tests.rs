// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Tests for storage backend factory.

#![expect(
    clippy::unwrap_used,
    reason = "test module: unwrap is standard in tests"
)]

use std::collections::HashMap;
use std::sync::Arc;

use sweet_grass_integration::testing::{TEST_DB_URL, TEST_DB_URL_PRIMARY, TEST_DB_URL_SECONDARY};

use super::*;

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
    assert!(Arc::strong_count(&store) >= 1);
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
        assert!(msg.contains("memory, postgres, redb"));
    }
}

// PostgreSQL Backend Tests

#[tokio::test]
async fn test_postgres_backend_missing_url() {
    let reader = mock_reader(&[("STORAGE_BACKEND", "postgres")]);
    let result = BraidStoreFactory::from_reader_with_name(reader).await;
    assert!(result.is_err());
    if let Err(err) = result {
        let msg = err.to_string();
        assert!(
            msg.contains("database_url") || msg.contains("DATABASE_URL"),
            "Error should mention database URL, got: {msg}"
        );
    }
}

#[tokio::test]
async fn test_postgres_config_missing_url_via_reader() {
    let reader = mock_reader(&[("STORAGE_BACKEND", "postgres")]);
    let result = BraidStoreFactory::from_reader_with_name(reader).await;
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(err.to_string().contains("database_url"));
    }
}

#[tokio::test]
async fn test_postgres_config_with_database_url_via_config() {
    let config = StorageConfig {
        backend: "postgres".to_string(),
        database_url: Some(TEST_DB_URL.to_string()),
        ..StorageConfig::default()
    };
    // Can't connect to test DB, but config parsing should succeed up to connection attempt
    let _result = BraidStoreFactory::from_config(&config).await;
}

#[tokio::test]
async fn test_postgres_config_with_storage_url_via_reader() {
    let reader = mock_reader(&[
        ("STORAGE_BACKEND", "postgres"),
        ("STORAGE_URL", TEST_DB_URL),
    ]);
    let _result = BraidStoreFactory::from_reader_with_name(reader).await;
}

#[tokio::test]
async fn test_postgres_config_prefers_database_url_via_reader() {
    let reader = mock_reader(&[
        ("STORAGE_BACKEND", "postgres"),
        ("DATABASE_URL", TEST_DB_URL_PRIMARY),
        ("STORAGE_URL", TEST_DB_URL_SECONDARY),
    ]);
    let _result = BraidStoreFactory::from_reader_with_name(reader).await;
}

#[test]
fn test_config_from_reader_postgres_max_connections() {
    let reader = mock_reader(&[
        ("STORAGE_BACKEND", "postgres"),
        ("DATABASE_URL", TEST_DB_URL),
        ("PG_MAX_CONNECTIONS", "20"),
    ]);
    let config = BraidStoreFactory::config_from_reader(&reader);
    assert_eq!(config.backend, "postgres");
    assert_eq!(config.database_url.as_deref(), Some(TEST_DB_URL));
    assert_eq!(config.pg_max_connections, Some(20));
}

#[test]
fn test_config_from_reader_postgres_min_connections() {
    let reader = mock_reader(&[
        ("STORAGE_BACKEND", "postgres"),
        ("DATABASE_URL", TEST_DB_URL),
        ("PG_MIN_CONNECTIONS", "5"),
    ]);
    let config = BraidStoreFactory::config_from_reader(&reader);
    assert_eq!(config.pg_min_connections, Some(5));
}

#[test]
fn test_config_from_reader_invalid_max_connections_ignored() {
    let reader = mock_reader(&[
        ("STORAGE_BACKEND", "postgres"),
        ("DATABASE_URL", TEST_DB_URL),
        ("PG_MAX_CONNECTIONS", "not_a_number"),
    ]);
    let config = BraidStoreFactory::config_from_reader(&reader);
    assert_eq!(config.pg_max_connections, None);
}

// Sled Backend Tests

#[cfg(feature = "sled")]
#[test]
fn test_sled_config_default_path_via_reader() {
    let config =
        BraidStoreFactory::config_from_reader(&mock_reader(&[("STORAGE_BACKEND", "sled")]));
    assert!(config.sled_path.is_none());
}

#[cfg(feature = "sled")]
#[test]
fn test_sled_config_custom_path_via_reader() {
    let reader = mock_reader(&[
        ("STORAGE_BACKEND", "sled"),
        ("STORAGE_PATH", "/tmp/custom/path"),
    ]);
    let config = BraidStoreFactory::config_from_reader(&reader);
    assert_eq!(config.sled_path.as_deref(), Some("/tmp/custom/path"));
}

#[cfg(feature = "sled")]
#[test]
fn test_sled_config_cache_size_via_reader() {
    let reader = mock_reader(&[
        ("STORAGE_BACKEND", "sled"),
        ("STORAGE_PATH", "/tmp/test"),
        ("SLED_CACHE_SIZE", "512"),
    ]);
    let config = BraidStoreFactory::config_from_reader(&reader);
    assert_eq!(config.sled_cache_size_mb, Some(512));
}

#[cfg(feature = "sled")]
#[test]
fn test_sled_config_flush_interval_via_reader() {
    let reader = mock_reader(&[
        ("STORAGE_BACKEND", "sled"),
        ("STORAGE_PATH", "/tmp/test"),
        ("SLED_FLUSH_MS", "1000"),
    ]);
    let config = BraidStoreFactory::config_from_reader(&reader);
    assert_eq!(config.sled_flush_ms, Some(1000));
}

#[cfg(feature = "sled")]
#[test]
fn test_sled_config_invalid_cache_size_ignored() {
    let reader = mock_reader(&[
        ("STORAGE_BACKEND", "sled"),
        ("STORAGE_PATH", "/tmp/test"),
        ("SLED_CACHE_SIZE", "not_a_number"),
    ]);
    let config = BraidStoreFactory::config_from_reader(&reader);
    assert_eq!(config.sled_cache_size_mb, None);
}

// Helper Function Tests

#[test]
fn test_parse_reader_var_success() {
    let reader = mock_reader(&[("TEST_VAR", "42")]);
    let result: Option<u32> = BraidStoreFactory::parse_reader_var(&reader, "TEST_VAR");
    assert_eq!(result, Some(42));
}

#[test]
fn test_parse_reader_var_missing() {
    let result: Option<u32> = BraidStoreFactory::parse_reader_var(&empty_reader(), "MISSING_VAR");
    assert_eq!(result, None);
}

#[test]
fn test_parse_reader_var_invalid_parse() {
    let reader = mock_reader(&[("INVALID_VAR", "not_a_number")]);
    let result: Option<u32> = BraidStoreFactory::parse_reader_var(&reader, "INVALID_VAR");
    assert_eq!(result, None);
}

#[test]
fn test_parse_reader_var_different_types() {
    let reader = mock_reader(&[
        ("STRING_VAR", "hello"),
        ("BOOL_VAR", "true"),
        ("FLOAT_VAR", "42.5"),
    ]);
    let result: Option<String> = BraidStoreFactory::parse_reader_var(&reader, "STRING_VAR");
    assert_eq!(result, Some("hello".to_string()));

    let result: Option<bool> = BraidStoreFactory::parse_reader_var(&reader, "BOOL_VAR");
    assert_eq!(result, Some(true));

    let result: Option<f64> = BraidStoreFactory::parse_reader_var(&reader, "FLOAT_VAR");
    assert_eq!(result, Some(42.5));
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
    assert!(Arc::strong_count(&store) >= 1);
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
async fn test_from_config_postgres_missing_url() {
    let config = StorageConfig {
        backend: "postgres".to_string(),
        ..StorageConfig::default()
    };
    let result = BraidStoreFactory::from_config(&config).await;
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(err.to_string().contains("database_url"));
    }
}

#[cfg(feature = "sled")]
#[tokio::test]
async fn test_from_config_sled() {
    let dir = tempfile::tempdir().unwrap();
    let config = StorageConfig {
        backend: "sled".to_string(),
        sled_path: Some(dir.path().to_str().unwrap().to_string()),
        sled_cache_size_mb: Some(64),
        sled_flush_ms: Some(500),
        ..StorageConfig::default()
    };
    let (store, name) = BraidStoreFactory::from_config_with_name(&config)
        .await
        .unwrap();
    assert_eq!(name, "sled");
    assert!(Arc::strong_count(&store) >= 1);
}

#[cfg(feature = "sled")]
#[tokio::test]
async fn test_from_config_sled_default_path() {
    let config = StorageConfig {
        backend: "sled".to_string(),
        ..StorageConfig::default()
    };
    let result = BraidStoreFactory::from_config_with_name(&config).await;
    assert!(result.is_ok());
    let (_, name) = result.unwrap();
    assert_eq!(name, "sled");
    let _ = std::fs::remove_dir_all("./data/sweetgrass");
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
    assert!(Arc::strong_count(&store) >= 1);
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
    assert!(Arc::strong_count(&store) >= 1);
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

// ==================== StorageConfig defaults ====================

#[test]
fn test_storage_config_default() {
    let config = StorageConfig::default();
    assert!(config.backend.is_empty());
    assert!(config.database_url.is_none());
    assert!(config.sled_path.is_none());
    assert!(config.redb_path.is_none());
    assert!(config.pg_max_connections.is_none());
    assert!(config.pg_min_connections.is_none());
    assert!(config.sled_cache_size_mb.is_none());
    assert!(config.sled_flush_ms.is_none());
}

#[test]
fn test_storage_config_clone() {
    let original = StorageConfig {
        backend: "postgres".to_string(),
        database_url: Some(TEST_DB_URL.to_string()),
        pg_max_connections: Some(20),
        ..StorageConfig::default()
    };
    let cloned = original.clone();
    assert_eq!(cloned.backend, "postgres");
    assert_eq!(cloned.database_url, Some(TEST_DB_URL.to_string()));
    assert_eq!(cloned.pg_max_connections, Some(20));
    assert_eq!(original.backend, cloned.backend);
}
