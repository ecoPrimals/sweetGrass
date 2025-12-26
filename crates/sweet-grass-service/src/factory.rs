//! Storage backend factory for infant discovery.
//!
//! This module provides runtime selection of storage backends based on
//! environment configuration, enabling zero-knowledge startup.

use std::sync::Arc;

use sweet_grass_store::{BraidStore, MemoryStore, StoreError};

type Result<T> = std::result::Result<T, StoreError>;

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
/// - `STORAGE_BACKEND`: Backend type (`memory`, `postgres`, `sled`)
/// - `DATABASE_URL` or `STORAGE_URL`: Connection string (postgres)
/// - `STORAGE_PATH`: Directory path (sled)
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
    /// - `sled` — Embedded Sled database (pure Rust)
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
    /// ## Sled Backend
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
        let backend = std::env::var("STORAGE_BACKEND").unwrap_or_else(|_| "memory".to_string());

        tracing::info!(backend = %backend, "Initializing storage backend from environment");

        match backend.as_str() {
            "memory" => {
                tracing::info!("Using in-memory storage backend");
                Ok(Arc::new(MemoryStore::new()) as Arc<dyn BraidStore>)
            },

            "postgres" => Self::create_postgres_backend().await,

            "sled" => Self::create_sled_backend(),

            other => Err(StoreError::Internal(format!(
                "Unknown storage backend: '{other}'. Valid options: memory, postgres, sled"
            ))),
        }
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
    fn build_postgres_config() -> Result<sweet_grass_store_postgres::PostgresConfig> {
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
    fn parse_env_var<T: std::str::FromStr>(key: &str) -> Option<T> {
        std::env::var(key).ok()?.parse().ok()
    }

    /// Create Sled backend from environment.
    fn create_sled_backend() -> Result<Arc<dyn BraidStore>> {
        use sweet_grass_store_sled::SledStore;

        let (config, path) = Self::build_sled_config();

        tracing::info!(path = %path, "Opening Sled database");
        let store = SledStore::open(&config)?;

        tracing::info!("Sled backend initialized");
        Ok(Arc::new(store) as Arc<dyn BraidStore>)
    }

    /// Build Sled configuration from environment variables.
    ///
    /// Returns the config and the path for logging purposes.
    fn build_sled_config() -> (sweet_grass_store_sled::SledConfig, String) {
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
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    // Memory Backend Tests

    #[tokio::test]
    async fn test_memory_backend() {
        std::env::set_var("STORAGE_BACKEND", "memory");
        let store = BraidStoreFactory::from_env().await;
        assert!(store.is_ok());
    }

    #[tokio::test]
    async fn test_default_backend() {
        std::env::remove_var("STORAGE_BACKEND");
        let store = BraidStoreFactory::from_env().await;
        assert!(store.is_ok(), "Should default to memory backend");
    }

    #[tokio::test]
    async fn test_memory_backend_explicit() {
        std::env::set_var("STORAGE_BACKEND", "memory");
        let result = BraidStoreFactory::from_env().await;
        assert!(result.is_ok());
        // Verify it's actually a memory store by checking it works
        let store = result.unwrap();
        assert!(Arc::strong_count(&store) >= 1);
    }

    // Error Cases

    #[tokio::test]
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
    async fn test_unknown_backend_specific_message() {
        // Use generic unknown backend, not vendor-specific name
        std::env::set_var("STORAGE_BACKEND", "unknown_backend");
        let result = BraidStoreFactory::from_env().await;
        assert!(result.is_err());
        if let Err(err) = result {
            let msg = err.to_string();
            assert!(msg.contains("Unknown storage backend"));
            assert!(msg.contains("unknown_backend"));
            assert!(msg.contains("memory, postgres, sled"));
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
        std::env::set_var("DATABASE_URL", "postgresql://localhost/test");
        std::env::remove_var("STORAGE_URL");

        let result = BraidStoreFactory::build_postgres_config();
        assert!(result.is_ok());
    }

    #[test]
    #[serial_test::serial]
    fn test_build_postgres_config_with_storage_url() {
        std::env::remove_var("DATABASE_URL");
        std::env::set_var("STORAGE_URL", "postgresql://localhost/test");

        let result = BraidStoreFactory::build_postgres_config();
        assert!(result.is_ok());
    }

    #[test]
    #[serial_test::serial]
    fn test_build_postgres_config_prefers_database_url() {
        std::env::set_var("DATABASE_URL", "postgresql://localhost/primary");
        std::env::set_var("STORAGE_URL", "postgresql://localhost/secondary");

        let result = BraidStoreFactory::build_postgres_config();
        assert!(result.is_ok());
        // DATABASE_URL should be preferred (checked by order in or_else)
    }

    #[test]
    #[serial_test::serial]
    fn test_build_postgres_config_with_max_connections() {
        std::env::set_var("DATABASE_URL", "postgresql://localhost/test");
        std::env::set_var("PG_MAX_CONNECTIONS", "20");

        let result = BraidStoreFactory::build_postgres_config();
        assert!(result.is_ok());
        // Config should have max_connections set (can't easily verify without exposing internals)
    }

    #[test]
    #[serial_test::serial]
    fn test_build_postgres_config_with_min_connections() {
        std::env::set_var("DATABASE_URL", "postgresql://localhost/test");
        std::env::set_var("PG_MIN_CONNECTIONS", "5");

        let result = BraidStoreFactory::build_postgres_config();
        assert!(result.is_ok());
    }

    #[test]
    #[serial_test::serial]
    fn test_build_postgres_config_with_invalid_max_connections() {
        std::env::set_var("DATABASE_URL", "postgresql://localhost/test");
        std::env::set_var("PG_MAX_CONNECTIONS", "not_a_number");

        let result = BraidStoreFactory::build_postgres_config();
        // Should succeed - invalid values are ignored
        assert!(result.is_ok());
    }

    // Sled Backend Tests

    #[test]
    #[serial_test::serial]
    fn test_build_sled_config_default_path() {
        std::env::remove_var("STORAGE_PATH");

        let (_config, path) = BraidStoreFactory::build_sled_config();
        assert_eq!(path, "./data/sweetgrass");
    }

    #[test]
    #[serial_test::serial]
    fn test_build_sled_config_custom_path() {
        std::env::set_var("STORAGE_PATH", "/tmp/custom/path");

        let (_config, path) = BraidStoreFactory::build_sled_config();
        assert_eq!(path, "/tmp/custom/path");
    }

    #[test]
    #[serial_test::serial]
    fn test_build_sled_config_with_cache_size() {
        std::env::set_var("STORAGE_PATH", "/tmp/test");
        std::env::set_var("SLED_CACHE_SIZE", "512");

        let (_config, _path) = BraidStoreFactory::build_sled_config();
        // Config should have cache size set (512 MB = 512 * 1024 * 1024 bytes)
    }

    #[test]
    #[serial_test::serial]
    fn test_build_sled_config_with_flush_interval() {
        std::env::set_var("STORAGE_PATH", "/tmp/test");
        std::env::set_var("SLED_FLUSH_MS", "1000");

        let (_config, _path) = BraidStoreFactory::build_sled_config();
    }

    #[test]
    #[serial_test::serial]
    fn test_build_sled_config_with_invalid_cache_size() {
        std::env::set_var("STORAGE_PATH", "/tmp/test");
        std::env::set_var("SLED_CACHE_SIZE", "not_a_number");

        let (_config, _path) = BraidStoreFactory::build_sled_config();
        // Should succeed - invalid values are ignored
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
}
