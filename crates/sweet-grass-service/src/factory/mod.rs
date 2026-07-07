// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Storage backend factory for infant discovery.
//!
//! This module provides runtime selection of storage backends based on
//! environment configuration, enabling zero-knowledge startup.

use sweet_grass_store::{MemoryStore, StoreError};

use crate::backend::BraidBackend;

type Result<T> = std::result::Result<T, StoreError>;

/// Explicit storage configuration (no env var mutation needed).
///
/// Use with `BraidStoreFactory::from_config()` to initialize storage without
/// mutating process environment variables. Safe for multi-threaded contexts.
#[derive(Clone, Debug, Default)]
pub struct StorageConfig {
    /// Backend type: "memory", "redb", "nestgate".
    pub backend: String,

    /// redb database path.
    pub redb_path: Option<String>,

    /// `NestGate` socket path (explicit override; uses discovery if absent).
    pub nestgate_socket: Option<String>,

    /// `NestGate` family ID for multi-instance scoping.
    pub nestgate_family_id: Option<String>,
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
/// - `STORAGE_BACKEND`: Backend type (`memory`, `redb`)
/// - `STORAGE_PATH`: File path (redb)
pub struct BraidStoreFactory;

impl BraidStoreFactory {
    /// Create a storage backend from environment variables.
    ///
    /// # Errors
    ///
    /// Returns error if backend initialization fails.
    pub async fn from_env() -> Result<BraidBackend> {
        Self::from_env_with_name().await.map(|(store, _)| store)
    }

    /// Create a storage backend from environment, returning the backend name.
    ///
    /// # Errors
    ///
    /// Returns error if backend initialization fails.
    pub async fn from_env_with_name() -> Result<(BraidBackend, &'static str)> {
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
    ) -> Result<(BraidBackend, &'static str)> {
        let config = Self::config_from_reader(&reader);
        Self::from_config_with_name(&config).await
    }

    /// Build a `StorageConfig` from a key reader (synchronous, no Send required).
    #[doc(hidden)]
    pub(crate) fn config_from_reader(reader: &impl Fn(&str) -> Option<String>) -> StorageConfig {
        StorageConfig {
            backend: reader("STORAGE_BACKEND")
                .unwrap_or_else(|| sweet_grass_core::identity::DEFAULT_STORAGE_BACKEND.to_string()),
            redb_path: reader("STORAGE_PATH"),
            nestgate_socket: reader(sweet_grass_core::primal_names::env_vars::NESTGATE_SOCKET),
            nestgate_family_id: reader(sweet_grass_core::primal_names::env_vars::FAMILY_ID),
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
    pub async fn from_config(config: &StorageConfig) -> Result<BraidBackend> {
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
    #[expect(
        clippy::unused_async,
        reason = "async retained for API stability with callers"
    )]
    pub async fn from_config_with_name(
        config: &StorageConfig,
    ) -> Result<(BraidBackend, &'static str)> {
        let backend = if config.backend.is_empty() {
            sweet_grass_core::identity::DEFAULT_STORAGE_BACKEND
        } else {
            config.backend.as_str()
        };

        tracing::info!(backend = %backend, "Initializing storage backend from config");

        match backend {
            "memory" => {
                tracing::info!("Using in-memory storage backend");
                Ok((BraidBackend::Memory(MemoryStore::new()), "memory"))
            },
            "redb" => Self::create_redb_from_config(config).map(|s| (s, "redb")),
            #[cfg(feature = "nestgate")]
            "nestgate" => Self::create_nestgate_from_config(config).map(|s| (s, "nestgate")),
            other => Err(StoreError::Internal(format!(
                "Unknown storage backend: '{other}'. Valid options: {}",
                Self::valid_backends()
            ))),
        }
    }

    /// Create redb backend from explicit config.
    fn create_redb_from_config(config: &StorageConfig) -> Result<BraidBackend> {
        use sweet_grass_store_redb::{RedbConfig, RedbStore};

        let path = config
            .redb_path
            .as_deref()
            .unwrap_or(sweet_grass_core::identity::DEFAULT_REDB_PATH);
        let redb_config = RedbConfig::new(path);

        tracing::info!(path = %path, "Opening redb database");
        let store = RedbStore::open(&redb_config)?;
        tracing::info!("redb backend initialized");
        Ok(BraidBackend::Redb(store))
    }

    #[cfg(feature = "nestgate")]
    /// Create `NestGate` backend from explicit config.
    fn create_nestgate_from_config(config: &StorageConfig) -> Result<BraidBackend> {
        use sweet_grass_store_nestgate::{NestGateConfig, NestGateStore};

        let ng_config = NestGateConfig {
            socket_path: config.nestgate_socket.clone(),
            family_id: config.nestgate_family_id.clone(),
            key_prefix: sweet_grass_core::identity::PRIMAL_NAME.to_string(),
        };

        tracing::info!("Configuring NestGate storage backend");
        let store = NestGateStore::new(&ng_config)?;
        tracing::info!("NestGate backend initialized");
        Ok(BraidBackend::NestGate(store))
    }

    /// List valid backend names (varies by enabled features).
    const fn valid_backends() -> &'static str {
        #[cfg(feature = "nestgate")]
        {
            "memory, redb, nestgate"
        }
        #[cfg(not(feature = "nestgate"))]
        {
            "memory, redb"
        }
    }
}

#[cfg(test)]
mod tests;
