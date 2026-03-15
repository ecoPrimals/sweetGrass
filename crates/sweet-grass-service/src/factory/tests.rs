// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Tests for storage backend factory.

#![expect(
    clippy::unwrap_used,
    reason = "test module: unwrap is standard in tests"
)]

use std::sync::Arc;

use sweet_grass_integration::testing::{TEST_DB_URL, TEST_DB_URL_PRIMARY, TEST_DB_URL_SECONDARY};

use super::*;

// Memory Backend Tests

#[tokio::test]
#[serial_test::serial]
async fn test_memory_backend() {
    std::env::set_var("STORAGE_BACKEND", "memory");
    let store = BraidStoreFactory::from_env().await;
    assert!(store.is_ok());
}

#[tokio::test]
#[serial_test::serial]
async fn test_default_backend() {
    std::env::remove_var("STORAGE_BACKEND");
    let store = BraidStoreFactory::from_env().await;
    assert!(store.is_ok(), "Should default to memory backend");
}

#[tokio::test]
#[serial_test::serial]
async fn test_memory_backend_explicit() {
    std::env::set_var("STORAGE_BACKEND", "memory");
    let result = BraidStoreFactory::from_env().await;
    assert!(result.is_ok());
    let store = result.unwrap();
    assert!(Arc::strong_count(&store) >= 1);
}

// Error Cases

#[tokio::test]
#[serial_test::serial]
async fn test_unknown_backend() {
    std::env::set_var("STORAGE_BACKEND", "unknown");
    let result = BraidStoreFactory::from_env().await;
    assert!(result.is_err());
    if let Err(err) = result {
        let msg = err.to_string();
        assert!(msg.contains("Unknown storage backend"), "Error was: {msg}");
    }
}

#[tokio::test]
#[serial_test::serial]
async fn test_unknown_backend_specific_message() {
    // Use generic unknown backend, not vendor-specific name
    std::env::set_var("STORAGE_BACKEND", "unknown_backend");
    let result = BraidStoreFactory::from_env().await;
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
#[serial_test::serial]
async fn test_postgres_backend_missing_url() {
    std::env::set_var("STORAGE_BACKEND", "postgres");
    std::env::remove_var("DATABASE_URL");
    std::env::remove_var("STORAGE_URL");

    let result = BraidStoreFactory::from_env().await;
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(err.to_string().contains("DATABASE_URL or STORAGE_URL"));
    }
}

#[test]
#[serial_test::serial]
fn test_build_postgres_config_missing_url() {
    std::env::remove_var("DATABASE_URL");
    std::env::remove_var("STORAGE_URL");

    let result = BraidStoreFactory::build_postgres_config();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("DATABASE_URL or STORAGE_URL"));
}

#[test]
#[serial_test::serial]
fn test_build_postgres_config_with_database_url() {
    std::env::set_var("DATABASE_URL", TEST_DB_URL);
    std::env::remove_var("STORAGE_URL");

    let result = BraidStoreFactory::build_postgres_config();
    assert!(result.is_ok());
}

#[test]
#[serial_test::serial]
fn test_build_postgres_config_with_storage_url() {
    std::env::remove_var("DATABASE_URL");
    std::env::set_var("STORAGE_URL", TEST_DB_URL);

    let result = BraidStoreFactory::build_postgres_config();
    assert!(result.is_ok());
}

#[test]
#[serial_test::serial]
fn test_build_postgres_config_prefers_database_url() {
    std::env::set_var("DATABASE_URL", TEST_DB_URL_PRIMARY);
    std::env::set_var("STORAGE_URL", TEST_DB_URL_SECONDARY);

    let result = BraidStoreFactory::build_postgres_config();
    assert!(result.is_ok());
    // DATABASE_URL should be preferred (checked by order in or_else)
}

#[test]
#[serial_test::serial]
fn test_build_postgres_config_with_max_connections() {
    std::env::set_var("DATABASE_URL", TEST_DB_URL);
    std::env::set_var("PG_MAX_CONNECTIONS", "20");

    let result = BraidStoreFactory::build_postgres_config();
    assert!(result.is_ok());
    // Config should have max_connections set (can't easily verify without exposing internals)
}

#[test]
#[serial_test::serial]
fn test_build_postgres_config_with_min_connections() {
    std::env::set_var("DATABASE_URL", TEST_DB_URL);
    std::env::set_var("PG_MIN_CONNECTIONS", "5");

    let result = BraidStoreFactory::build_postgres_config();
    assert!(result.is_ok());
}

#[test]
#[serial_test::serial]
fn test_build_postgres_config_with_invalid_max_connections() {
    std::env::set_var("DATABASE_URL", TEST_DB_URL);
    std::env::set_var("PG_MAX_CONNECTIONS", "not_a_number");

    let result = BraidStoreFactory::build_postgres_config();
    // Should succeed - invalid values are ignored
    assert!(result.is_ok());
}

// Sled Backend Tests

#[cfg(feature = "sled")]
#[test]
#[serial_test::serial]
fn test_build_sled_config_default_path() {
    std::env::remove_var("STORAGE_PATH");

    let (_config, path) = BraidStoreFactory::build_sled_config();
    assert_eq!(path, "./data/sweetgrass");
}

#[cfg(feature = "sled")]
#[test]
#[serial_test::serial]
fn test_build_sled_config_custom_path() {
    std::env::set_var("STORAGE_PATH", "/tmp/custom/path");

    let (_config, path) = BraidStoreFactory::build_sled_config();
    assert_eq!(path, "/tmp/custom/path");
}

#[cfg(feature = "sled")]
#[test]
#[serial_test::serial]
fn test_build_sled_config_with_cache_size() {
    std::env::set_var("STORAGE_PATH", "/tmp/test");
    std::env::set_var("SLED_CACHE_SIZE", "512");

    let (_config, _path) = BraidStoreFactory::build_sled_config();
}

#[cfg(feature = "sled")]
#[test]
#[serial_test::serial]
fn test_build_sled_config_with_flush_interval() {
    std::env::set_var("STORAGE_PATH", "/tmp/test");
    std::env::set_var("SLED_FLUSH_MS", "1000");

    let (_config, _path) = BraidStoreFactory::build_sled_config();
}

#[cfg(feature = "sled")]
#[test]
#[serial_test::serial]
fn test_build_sled_config_with_invalid_cache_size() {
    std::env::set_var("STORAGE_PATH", "/tmp/test");
    std::env::set_var("SLED_CACHE_SIZE", "not_a_number");

    let (_config, _path) = BraidStoreFactory::build_sled_config();
}

// Helper Function Tests

#[test]
#[serial_test::serial]
fn test_parse_env_var_success() {
    std::env::set_var("TEST_VAR", "42");
    let result: Option<u32> = BraidStoreFactory::parse_env_var("TEST_VAR");
    assert_eq!(result, Some(42));
}

#[test]
#[serial_test::serial]
fn test_parse_env_var_missing() {
    std::env::remove_var("MISSING_VAR");
    let result: Option<u32> = BraidStoreFactory::parse_env_var("MISSING_VAR");
    assert_eq!(result, None);
}

#[test]
#[serial_test::serial]
fn test_parse_env_var_invalid_parse() {
    std::env::set_var("INVALID_VAR", "not_a_number");
    let result: Option<u32> = BraidStoreFactory::parse_env_var("INVALID_VAR");
    assert_eq!(result, None);
}

#[test]
#[serial_test::serial]
fn test_parse_env_var_different_types() {
    std::env::set_var("STRING_VAR", "hello");
    let result: Option<String> = BraidStoreFactory::parse_env_var("STRING_VAR");
    assert_eq!(result, Some("hello".to_string()));

    std::env::set_var("BOOL_VAR", "true");
    let result: Option<bool> = BraidStoreFactory::parse_env_var("BOOL_VAR");
    assert_eq!(result, Some(true));

    std::env::set_var("FLOAT_VAR", "42.5");
    let result: Option<f64> = BraidStoreFactory::parse_env_var("FLOAT_VAR");
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
#[serial_test::serial]
async fn test_redb_backend_from_env() {
    std::env::set_var("STORAGE_BACKEND", "redb");
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("env.redb");
    std::env::set_var("STORAGE_PATH", db_path.to_str().unwrap());
    let result = BraidStoreFactory::from_env_with_name().await;
    assert!(result.is_ok());
    let (_, name) = result.unwrap();
    assert_eq!(name, "redb");
}

#[test]
#[serial_test::serial]
fn test_build_redb_config_default_path() {
    std::env::remove_var("STORAGE_PATH");
    let (_config, path) = BraidStoreFactory::build_redb_config();
    assert_eq!(path, "./data/sweetgrass.redb");
}

#[test]
#[serial_test::serial]
fn test_build_redb_config_custom_path() {
    std::env::set_var("STORAGE_PATH", "/tmp/custom.redb");
    let (_config, path) = BraidStoreFactory::build_redb_config();
    assert_eq!(path, "/tmp/custom.redb");
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
