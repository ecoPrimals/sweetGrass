// SPDX-License-Identifier: AGPL-3.0-only
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

    /// PostgreSQL connection URL.
    pub database_url: Option<String>,

    /// Sled database path.
    pub sled_path: Option<String>,

    /// redb database path.
    pub redb_path: Option<String>,

    /// PostgreSQL max connections.
    pub pg_max_connections: Option<u32>,

    /// PostgreSQL min connections.
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
    /// # Environment Configuration
    ///
    /// ## `STORAGE_BACKEND` (required)
    ///
    /// Selects the storage backend:
    /// - `memory` — In-memory storage (ephemeral, for testing)
    /// - `postgres` — PostgreSQL database (production)
    /// - `redb` — Embedded redb database (Pure Rust, recommended)
    /// - `sled` — Embedded Sled database (Pure Rust, legacy)
    ///
    /// Default: `memory`
    ///
    /// ## PostgreSQL Backend
    ///
    /// Requires one of:
    /// - `DATABASE_URL` — Connection string
    /// - `STORAGE_URL` — Alternative connection string
    ///
    /// Format: `postgresql://user:pass@host:port/database`
    ///
    /// Optional:
    /// - `PG_MAX_CONNECTIONS` — Pool size (default: 10)
    /// - `PG_MIN_CONNECTIONS` — Minimum connections (default: 2)
    /// - `PG_CONNECT_TIMEOUT` — Timeout in seconds (default: 30)
    ///
    /// ## redb Backend (recommended embedded)
    ///
    /// Optional:
    /// - `STORAGE_PATH` — Database file path (default: `./data/sweetgrass.redb`)
    ///
    /// ## Sled Backend (legacy)
    ///
    /// Optional:
    /// - `STORAGE_PATH` — Directory path (default: `./data/sweetgrass`)
    /// - `SLED_CACHE_SIZE` — Cache size in MB (default: 1024)
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Unknown backend type specified
    /// - Required environment variables missing
    /// - Backend initialization fails
    pub async fn from_env() -> Result<Arc<dyn BraidStore>> {
        Self::from_env_with_name().await.map(|(store, _)| store)
    }

    /// Create a storage backend from environment, returning the backend name.
    ///
    /// This is the single authoritative path for storage discovery. Both
    /// `infant_bootstrap` and direct callers use this — no redundant env checks.
    ///
    /// # Errors
    ///
    /// Returns error if backend initialization fails.
    pub async fn from_env_with_name() -> Result<(Arc<dyn BraidStore>, String)> {
        let backend = std::env::var("STORAGE_BACKEND").unwrap_or_else(|_| "memory".to_string());

        tracing::info!(backend = %backend, "Initializing storage backend from environment");

        match backend.as_str() {
            "memory" => {
                tracing::info!("Using in-memory storage backend");
                Ok((
                    Arc::new(MemoryStore::new()) as Arc<dyn BraidStore>,
                    "memory".to_string(),
                ))
            },

            "postgres" => Self::create_postgres_backend()
                .await
                .map(|s| (s, "postgres".to_string())),

            "redb" => Self::create_redb_backend().map(|s| (s, "redb".to_string())),

            #[cfg(feature = "sled")]
            "sled" => Self::create_sled_backend().map(|s| (s, "sled".to_string())),

            other => Err(StoreError::Internal(format!(
                "Unknown storage backend: '{other}'. Valid options: {}",
                Self::valid_backends()
            ))),
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
            "memory"
        } else {
            config.backend.as_str()
        };

        tracing::info!(backend = %backend, "Initializing storage backend from config");

        match backend {
            "memory" => {
                tracing::info!("Using in-memory storage backend");
                Ok((
                    Arc::new(MemoryStore::new()) as Arc<dyn BraidStore>,
                    "memory".to_string(),
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

    /// Create PostgreSQL backend from explicit config.
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
            .unwrap_or("./data/sweetgrass.redb");
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

        let path = config.sled_path.as_deref().unwrap_or("./data/sweetgrass");
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

    /// Create PostgreSQL backend from environment.
    async fn create_postgres_backend() -> Result<Arc<dyn BraidStore>> {
        use sweet_grass_store_postgres::PostgresStore;

        let config = Self::build_postgres_config()?;

        tracing::info!("Connecting to PostgreSQL database");
        let store = PostgresStore::connect(&config).await?;

        tracing::info!("Running database migrations");
        store.run_migrations().await?;

        tracing::info!("PostgreSQL backend initialized");
        Ok(Arc::new(store) as Arc<dyn BraidStore>)
    }

    /// Build PostgreSQL configuration from environment variables.
    #[doc(hidden)]
    pub(crate) fn build_postgres_config() -> Result<sweet_grass_store_postgres::PostgresConfig> {
        // Get connection URL (try DATABASE_URL first, then STORAGE_URL)
        let url = std::env::var("DATABASE_URL")
            .or_else(|_| std::env::var("STORAGE_URL"))
            .map_err(|_| {
                StoreError::Internal(
                    "PostgreSQL backend requires DATABASE_URL or STORAGE_URL".to_string(),
                )
            })?;

        let mut config = sweet_grass_store_postgres::PostgresConfig::new(&url);

        // Apply optional connection pool settings (idiomatic pattern)
        if let Some(max) = Self::parse_env_var("PG_MAX_CONNECTIONS") {
            config = config.max_connections(max);
        }

        if let Some(min) = Self::parse_env_var("PG_MIN_CONNECTIONS") {
            config = config.min_connections(min);
        }

        Ok(config)
    }

    /// Parse an environment variable as a specific type (helper for config building).
    #[doc(hidden)]
    pub(crate) fn parse_env_var<T: std::str::FromStr>(key: &str) -> Option<T> {
        std::env::var(key).ok()?.parse().ok()
    }

    /// Create redb backend from environment.
    fn create_redb_backend() -> Result<Arc<dyn BraidStore>> {
        use sweet_grass_store_redb::RedbStore;

        let (config, path) = Self::build_redb_config();

        tracing::info!(path = %path, "Opening redb database");
        let store = RedbStore::open(&config)?;

        tracing::info!("redb backend initialized");
        Ok(Arc::new(store) as Arc<dyn BraidStore>)
    }

    /// Build redb configuration from environment variables.
    ///
    /// Returns the config and the path for logging purposes.
    #[doc(hidden)]
    pub(crate) fn build_redb_config() -> (sweet_grass_store_redb::RedbConfig, String) {
        use sweet_grass_store_redb::RedbConfig;

        let path =
            std::env::var("STORAGE_PATH").unwrap_or_else(|_| "./data/sweetgrass.redb".to_string());
        let config = RedbConfig::new(&path);

        (config, path)
    }

    #[cfg(feature = "sled")]
    /// Create Sled backend from environment.
    fn create_sled_backend() -> Result<Arc<dyn BraidStore>> {
        use sweet_grass_store_sled::SledStore;

        let (config, path) = Self::build_sled_config();

        tracing::info!(path = %path, "Opening Sled database");
        let store = SledStore::open(&config)?;

        tracing::info!("Sled backend initialized");
        Ok(Arc::new(store) as Arc<dyn BraidStore>)
    }

    #[cfg(feature = "sled")]
    /// Build Sled configuration from environment variables.
    ///
    /// Returns the config and the path for logging purposes.
    #[doc(hidden)]
    pub(crate) fn build_sled_config() -> (sweet_grass_store_sled::SledConfig, String) {
        use sweet_grass_store_sled::SledConfig;

        let path =
            std::env::var("STORAGE_PATH").unwrap_or_else(|_| "./data/sweetgrass".to_string());
        let mut config = SledConfig::new(&path);

        // Apply optional cache size (convert MB to bytes idiomatically)
        if let Some(cache_mb) = Self::parse_env_var::<u64>("SLED_CACHE_SIZE") {
            config = config.cache_capacity(cache_mb * 1024 * 1024);
        }

        // Apply optional flush interval
        if let Some(flush_ms) = Self::parse_env_var::<u64>("SLED_FLUSH_MS") {
            config = config.flush_every_ms(Some(flush_ms));
        }

        (config, path)
    }

    /// List valid backend names (varies by enabled features).
    fn valid_backends() -> &'static str {
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
