// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Storage backend factory for infant discovery.
//!
//! This module provides runtime selection of storage backends based on
//! environment configuration, enabling zero-knowledge startup.

use std::sync::Arc;

use sweet_grass_store::{BraidStore, MemoryStore, StoreError};

type Result<T> = std::result::Result<T, StoreError>;

/// Explicit storage configuration (no env var mutation needed).
///
/// Use with `BraidStoreFactory::from_config()` to initialize storage without
/// mutating process environment variables. Safe for multi-threaded contexts.
#[derive(Clone, Debug, Default)]
pub struct StorageConfig {
    /// Backend type: "memory", "postgres", "sled", "redb".
    pub backend: String,

    /// `PostgreSQL` connection URL.
    pub database_url: Option<String>,

    /// Sled database path.
    pub sled_path: Option<String>,

    /// redb database path.
    pub redb_path: Option<String>,

    /// `PostgreSQL` max connections.
    pub pg_max_connections: Option<u32>,

    /// `PostgreSQL` min connections.
    pub pg_min_connections: Option<u32>,

    /// Sled cache size in MB.
    pub sled_cache_size_mb: Option<u64>,

    /// Sled flush interval in ms.
    pub sled_flush_ms: Option<u64>,
}

/// Factory for creating storage backends from environment configuration.
///
/// ## Infant Discovery Pattern
///
/// The factory enables zero-knowledge startup:
/// 1. Service reads environment variables (no hardcoding)
/// 2. Factory selects appropriate backend at runtime
/// 3. Service operates with discovered backend
///
/// ## Environment Variables
///
/// - `STORAGE_BACKEND`: Backend type (`memory`, `postgres`, `sled`, `redb`)
/// - `DATABASE_URL` or `STORAGE_URL`: Connection string (postgres)
/// - `STORAGE_PATH`: Directory path (sled/redb)
pub struct BraidStoreFactory;

impl BraidStoreFactory {
    /// Create a storage backend from environment variables.
    ///
    /// # Errors
    ///
    /// Returns error if backend initialization fails.
    pub async fn from_env() -> Result<Arc<dyn BraidStore>> {
        Self::from_env_with_name().await.map(|(store, _)| store)
    }

    /// Create a storage backend from environment, returning the backend name.
    ///
    /// # Errors
    ///
    /// Returns error if backend initialization fails.
    pub async fn from_env_with_name() -> Result<(Arc<dyn BraidStore>, String)> {
        let config = Self::config_from_reader(&|key| std::env::var(key).ok());
        Self::from_config_with_name(&config).await
    }

    /// Create a storage backend using an injectable key reader (DI-friendly).
    ///
    /// Tests inject a closure instead of mutating process-global env vars.
    /// The reader is consumed synchronously before any async work begins,
    /// so it does not need to be `Send` or `Sync`.
    ///
    /// # Errors
    ///
    /// Returns error if backend initialization fails.
    pub async fn from_reader_with_name(
        reader: impl Fn(&str) -> Option<String>,
    ) -> Result<(Arc<dyn BraidStore>, String)> {
        let config = Self::config_from_reader(&reader);
        Self::from_config_with_name(&config).await
    }

    /// Build a `StorageConfig` from a key reader (synchronous, no Send required).
    #[doc(hidden)]
    pub(crate) fn config_from_reader(reader: &impl Fn(&str) -> Option<String>) -> StorageConfig {
        StorageConfig {
            backend: reader("STORAGE_BACKEND")
                .unwrap_or_else(|| sweet_grass_core::identity::DEFAULT_STORAGE_BACKEND.to_string()),
            database_url: reader("DATABASE_URL").or_else(|| reader("STORAGE_URL")),
            sled_path: reader("STORAGE_PATH"),
            redb_path: reader("STORAGE_PATH"),
            pg_max_connections: Self::parse_reader_var(reader, "PG_MAX_CONNECTIONS"),
            pg_min_connections: Self::parse_reader_var(reader, "PG_MIN_CONNECTIONS"),
            sled_cache_size_mb: Self::parse_reader_var(reader, "SLED_CACHE_SIZE"),
            sled_flush_ms: Self::parse_reader_var(reader, "SLED_FLUSH_MS"),
        }
    }

    /// Create a storage backend from explicit configuration.
    ///
    /// Use this instead of `from_env()` when config is known at call site (e.g. CLI args)
    /// to avoid mutating process environment variables.
    ///
    /// # Errors
    ///
    /// Returns error if backend initialization fails.
    pub async fn from_config(config: &StorageConfig) -> Result<Arc<dyn BraidStore>> {
        Self::from_config_with_name(config)
            .await
            .map(|(store, _)| store)
    }

    /// Create a storage backend from explicit config, returning the backend name.
    ///
    /// Use this instead of `from_env_with_name()` when config is known at call site.
    ///
    /// # Errors
    ///
    /// Returns error if backend initialization fails.
    pub async fn from_config_with_name(
        config: &StorageConfig,
    ) -> Result<(Arc<dyn BraidStore>, String)> {
        let backend = if config.backend.is_empty() {
            sweet_grass_core::identity::DEFAULT_STORAGE_BACKEND
        } else {
            config.backend.as_str()
        };

        tracing::info!(backend = %backend, "Initializing storage backend from config");

        match backend {
            "memory" => {
                tracing::info!("Using in-memory storage backend");
                Ok((
                    Arc::new(MemoryStore::new()) as Arc<dyn BraidStore>,
                    sweet_grass_core::identity::DEFAULT_STORAGE_BACKEND.to_string(),
                ))
            },
            "postgres" => Self::create_postgres_from_config(config)
                .await
                .map(|s| (s, "postgres".to_string())),
            "redb" => Self::create_redb_from_config(config).map(|s| (s, "redb".to_string())),
            #[cfg(feature = "sled")]
            "sled" => Self::create_sled_from_config(config).map(|s| (s, "sled".to_string())),
            other => Err(StoreError::Internal(format!(
                "Unknown storage backend: '{other}'. Valid options: {}",
                Self::valid_backends()
            ))),
        }
    }

    /// Create `PostgreSQL` backend from explicit config.
    async fn create_postgres_from_config(config: &StorageConfig) -> Result<Arc<dyn BraidStore>> {
        use sweet_grass_store_postgres::PostgresStore;

        let url = config.database_url.as_deref().ok_or_else(|| {
            StoreError::Internal("PostgreSQL backend requires database_url".to_string())
        })?;

        let mut pg_config = sweet_grass_store_postgres::PostgresConfig::new(url);
        if let Some(max) = config.pg_max_connections {
            pg_config = pg_config.max_connections(max);
        }
        if let Some(min) = config.pg_min_connections {
            pg_config = pg_config.min_connections(min);
        }

        tracing::info!("Connecting to PostgreSQL database");
        let store = PostgresStore::connect(&pg_config).await?;
        tracing::info!("Running database migrations");
        store.run_migrations().await?;
        tracing::info!("PostgreSQL backend initialized");
        Ok(Arc::new(store) as Arc<dyn BraidStore>)
    }

    /// Create redb backend from explicit config.
    fn create_redb_from_config(config: &StorageConfig) -> Result<Arc<dyn BraidStore>> {
        use sweet_grass_store_redb::{RedbConfig, RedbStore};

        let path = config
            .redb_path
            .as_deref()
            .unwrap_or(sweet_grass_core::identity::DEFAULT_REDB_PATH);
        let redb_config = RedbConfig::new(path);

        tracing::info!(path = %path, "Opening redb database");
        let store = RedbStore::open(&redb_config)?;
        tracing::info!("redb backend initialized");
        Ok(Arc::new(store) as Arc<dyn BraidStore>)
    }

    #[cfg(feature = "sled")]
    /// Create Sled backend from explicit config.
    fn create_sled_from_config(config: &StorageConfig) -> Result<Arc<dyn BraidStore>> {
        use sweet_grass_store_sled::{SledConfig, SledStore};

        let path = config
            .sled_path
            .as_deref()
            .unwrap_or(sweet_grass_core::identity::DEFAULT_SLED_PATH);
        let mut sled_config = SledConfig::new(path);
        if let Some(cache_mb) = config.sled_cache_size_mb {
            sled_config = sled_config.cache_capacity(cache_mb * 1024 * 1024);
        }
        if let Some(flush_ms) = config.sled_flush_ms {
            sled_config = sled_config.flush_every_ms(Some(flush_ms));
        }

        tracing::info!(path = %path, "Opening Sled database");
        let store = SledStore::open(&sled_config)?;
        tracing::info!("Sled backend initialized");
        Ok(Arc::new(store) as Arc<dyn BraidStore>)
    }

    /// Parse a key from a reader as a specific type.
    #[doc(hidden)]
    pub(crate) fn parse_reader_var<T: std::str::FromStr>(
        reader: &impl Fn(&str) -> Option<String>,
        key: &str,
    ) -> Option<T> {
        reader(key)?.parse().ok()
    }

    /// List valid backend names (varies by enabled features).
    const fn valid_backends() -> &'static str {
        #[cfg(feature = "sled")]
        {
            "memory, postgres, redb, sled"
        }
        #[cfg(not(feature = "sled"))]
        {
            "memory, postgres, redb"
        }
    }
}

#[cfg(test)]
mod tests;
