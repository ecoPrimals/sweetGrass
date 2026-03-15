// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Infant Discovery Bootstrap.
//!
//! The entry point for zero-knowledge startup. A primal is born knowing
//! only itself and discovers all other services at runtime.

use tracing::{debug, info, instrument};

use crate::factory::{BraidStoreFactory, StorageConfig};
use crate::state::AppState;
use sweet_grass_core::agent::Did;
use sweet_grass_core::SelfKnowledge;

/// Explicit bootstrap configuration (avoids env var mutation).
///
/// Use with `infant_bootstrap_with_config()` to bootstrap without mutating
/// process environment variables. Safe for multi-threaded contexts.
#[derive(Clone, Debug, Default)]
pub struct BootstrapConfig {
    /// Storage configuration.
    pub storage: StorageConfig,

    /// Override primal name (otherwise read from `PRIMAL_NAME` env var).
    pub primal_name: Option<String>,

    /// Override instance ID (otherwise read from `PRIMAL_INSTANCE_ID` env var).
    pub instance_id: Option<String>,
}

/// Bootstrap error.
#[derive(Debug, thiserror::Error)]
pub enum BootstrapError {
    /// Failed to establish self-knowledge.
    #[error("Self-knowledge error: {0}")]
    SelfKnowledge(String),

    /// Failed to initialize storage.
    #[error("Storage initialization error: {0}")]
    Storage(#[from] sweet_grass_store::StoreError),

    /// Failed to discover required capability.
    #[error("Capability discovery error: {0}")]
    Discovery(String),
}

/// The result of infant bootstrap.
pub struct BootstrapResult {
    /// Self-knowledge established from environment.
    pub self_knowledge: SelfKnowledge,

    /// Application state ready for service startup.
    pub app_state: AppState,

    /// Default agent DID for braid creation.
    pub default_agent: Did,
}

/// Infant Bootstrap - Zero-knowledge startup sequence.
///
/// ## Infant Discovery Pattern
///
/// 1. **Birth**: Load self-knowledge from environment (no hardcoding)
/// 2. **Storage**: Discover and initialize storage backend
/// 3. **Capabilities**: Discover required services (signing, anchoring, etc.)
/// 4. **Ready**: Full network effects without hardcoded connections
///
/// ## Environment Variables
///
/// ### Self-Knowledge
/// - `PRIMAL_NAME`: Human-readable name (default: "sweetgrass")
/// - `PRIMAL_INSTANCE_ID`: Unique ID (default: random UUID)
/// - `PRIMAL_CAPABILITIES`: Comma-separated capabilities offered
/// - `TARPC_PORT`: tarpc endpoint port (0 = auto-allocate)
/// - `REST_PORT`: REST endpoint port (default: 0 - dynamic allocation)
///
/// ### Storage
/// - `DATABASE_URL`: `PostgreSQL` connection string
/// - `SLED_PATH`: Sled database path
/// - (default: in-memory)
///
/// ### Discovery (Phase 2)
/// - `DISCOVERY_URL`: Service discovery endpoint
/// - `DISCOVERY_NAMESPACE`: Discovery namespace
///
/// ## Example
///
/// ```rust,ignore
/// use sweet_grass_service::bootstrap::infant_bootstrap;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Environment:
///     // PRIMAL_NAME=sweetgrass-prod
///     // DATABASE_URL=postgres://...
///
///     let result = infant_bootstrap().await?;
///     println!("Primal: {}", result.self_knowledge.name);
///     println!("Uptime: {:?}", result.self_knowledge.uptime());
///
///     // Start services with app_state...
///     Ok(())
/// }
/// ```
///
/// # Errors
///
/// Returns error if:
/// - Environment variables are malformed
/// - Storage backend initialization fails
/// - Required capability discovery fails
#[instrument]
pub async fn infant_bootstrap() -> Result<BootstrapResult, BootstrapError> {
    info!("🌱 Infant Bootstrap: Starting zero-knowledge initialization");

    // Phase 1: Establish self-knowledge from environment
    debug!("Phase 1: Establishing self-knowledge from environment");
    let self_knowledge = SelfKnowledge::from_env().map_err(BootstrapError::SelfKnowledge)?;

    info!(
        primal_name = %self_knowledge.name,
        instance_id = %self_knowledge.instance_id,
        capabilities = ?self_knowledge.capabilities,
        tarpc_port = self_knowledge.tarpc_port,
        rest_port = self_knowledge.rest_port,
        "Self-knowledge established"
    );

    // Phase 2: Discover and initialize storage backend via factory
    // Single path through BraidStoreFactory — no redundant env checks here.
    debug!("Phase 2: Discovering storage backend from environment");
    let (store, backend_type) = BraidStoreFactory::from_env_with_name().await?;

    info!(backend = backend_type, "Storage backend initialized");

    // Phase 3: Establish default agent identity
    debug!("Phase 3: Establishing default agent identity");
    let default_agent = Did::new(format!("did:primal:{}", self_knowledge.instance_id));

    // Phase 4: Create application state with self-knowledge
    debug!("Phase 4: Creating application state");
    let app_state = AppState::with_self_knowledge(
        store,
        default_agent.clone(),
        self_knowledge.clone(),
        &backend_type,
    );

    // Phase 5: Capability discovery (future enhancement)
    // For now, we log what capabilities we offer and will discover in Phase 2
    if !self_knowledge.capabilities.is_empty() {
        debug!(
            capabilities = ?self_knowledge.capabilities,
            "Capabilities offered (discovery integration pending)"
        );
    }

    info!(
        primal_name = %self_knowledge.name,
        default_agent = %default_agent,
        "🌾 Infant Bootstrap complete: Born knowing nothing, ready for everything"
    );

    Ok(BootstrapResult {
        self_knowledge,
        app_state,
        default_agent,
    })
}

/// Infant Bootstrap with explicit configuration (no env var mutation).
///
/// Use this instead of `infant_bootstrap()` when storage config is known at
/// call site (e.g. CLI args). Primal identity still comes from environment.
///
/// # Errors
///
/// Returns error if self-knowledge or storage initialization fails.
#[instrument(skip(config))]
pub async fn infant_bootstrap_with_config(
    config: BootstrapConfig,
) -> Result<BootstrapResult, BootstrapError> {
    info!("🌱 Infant Bootstrap: Starting with explicit configuration");

    // Phase 1: Establish self-knowledge from environment
    // (primal identity still comes from env — only storage is config-driven)
    debug!("Phase 1: Establishing self-knowledge from environment");
    let self_knowledge = SelfKnowledge::from_env().map_err(BootstrapError::SelfKnowledge)?;

    info!(
        primal_name = %self_knowledge.name,
        instance_id = %self_knowledge.instance_id,
        capabilities = ?self_knowledge.capabilities,
        "Self-knowledge established"
    );

    // Phase 2: Initialize storage from explicit config (no env mutation)
    debug!("Phase 2: Initializing storage from explicit config");
    let (store, backend_type) = BraidStoreFactory::from_config_with_name(&config.storage).await?;

    info!(backend = backend_type, "Storage backend initialized");

    // Phase 3: Establish default agent identity
    debug!("Phase 3: Establishing default agent identity");
    let default_agent = Did::new(format!("did:primal:{}", self_knowledge.instance_id));

    // Phase 4: Create application state
    debug!("Phase 4: Creating application state");
    let app_state = AppState::with_self_knowledge(
        store,
        default_agent.clone(),
        self_knowledge.clone(),
        &backend_type,
    );

    if !self_knowledge.capabilities.is_empty() {
        debug!(
            capabilities = ?self_knowledge.capabilities,
            "Capabilities offered (discovery integration pending)"
        );
    }

    info!(
        primal_name = %self_knowledge.name,
        default_agent = %default_agent,
        "🌾 Infant Bootstrap complete"
    );

    Ok(BootstrapResult {
        self_knowledge,
        app_state,
        default_agent,
    })
}

/// Create application state with runtime-discovered storage.
///
/// This is a convenience function for tests and examples that don't need
/// full bootstrap. Uses a test DID as the default agent.
///
/// # Errors
///
/// Returns error if storage initialization fails.
pub async fn create_app_state_from_env() -> Result<AppState, BootstrapError> {
    let store = BraidStoreFactory::from_env().await?;
    let default_agent = Did::new("did:primal:test");
    Ok(AppState::with_store(store, default_agent))
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "test module: expect is standard in tests"
)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use sweet_grass_core::Capability;

    // NOTE: These tests modify environment variables and should be run with --test-threads=1

    /// All storage-related env vars that must be cleared for test isolation.
    const STORAGE_ENV_VARS: &[&str] = &[
        "PRIMAL_NAME",
        "PRIMAL_INSTANCE_ID",
        "PRIMAL_CAPABILITIES",
        "DATABASE_URL",
        "STORAGE_URL",
        "STORAGE_BACKEND",
        "STORAGE_PATH",
        "SLED_PATH",
    ];

    fn clear_env() {
        for var in STORAGE_ENV_VARS {
            std::env::remove_var(var);
        }
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_infant_bootstrap_defaults() {
        clear_env();

        let result = infant_bootstrap().await.expect("should bootstrap");

        assert_eq!(result.self_knowledge.name, "sweetgrass");
        assert!(!result.self_knowledge.instance_id.is_empty());
        assert!(result.default_agent.as_str().starts_with("did:primal:"));
        // Verify app state has a store
        assert!(Arc::strong_count(&result.app_state.store) >= 1);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_infant_bootstrap_with_config() {
        clear_env();
        std::env::set_var("PRIMAL_NAME", "sweetgrass-test");
        std::env::set_var("PRIMAL_INSTANCE_ID", "test-123");
        std::env::set_var("PRIMAL_CAPABILITIES", "signing,anchoring");

        let result = infant_bootstrap().await.expect("should bootstrap");

        assert_eq!(result.self_knowledge.name, "sweetgrass-test");
        assert_eq!(result.self_knowledge.instance_id, "test-123");
        assert_eq!(result.self_knowledge.capabilities.len(), 2);
        assert!(result.self_knowledge.offers(&Capability::Signing));
        assert_eq!(result.default_agent.as_str(), "did:primal:test-123");

        clear_env();
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_create_app_state_from_env() {
        clear_env();

        let app_state = create_app_state_from_env()
            .await
            .expect("should create app state");

        // Verify app state has a store
        assert!(Arc::strong_count(&app_state.store) >= 1);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_infant_bootstrap_with_explicit_config_memory() {
        clear_env();
        let config = BootstrapConfig {
            storage: crate::factory::StorageConfig {
                backend: "memory".to_string(),
                ..Default::default()
            },
            primal_name: None,
            instance_id: None,
        };

        let result = infant_bootstrap_with_config(config)
            .await
            .expect("should bootstrap with config");

        assert_eq!(result.self_knowledge.name, "sweetgrass");
        assert_eq!(result.app_state.store_backend, "memory");
        assert!(Arc::strong_count(&result.app_state.store) >= 1);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_infant_bootstrap_with_explicit_config_primal_identity() {
        clear_env();
        std::env::set_var("PRIMAL_NAME", "sg-config-test");
        std::env::set_var("PRIMAL_INSTANCE_ID", "cfg-instance-42");

        let config = BootstrapConfig {
            storage: crate::factory::StorageConfig {
                backend: "memory".to_string(),
                ..Default::default()
            },
            primal_name: None,
            instance_id: None,
        };

        let result = infant_bootstrap_with_config(config)
            .await
            .expect("bootstrap");
        assert_eq!(result.self_knowledge.name, "sg-config-test");
        assert_eq!(result.self_knowledge.instance_id, "cfg-instance-42");
        assert_eq!(result.default_agent.as_str(), "did:primal:cfg-instance-42");
        assert!(result.app_state.self_knowledge.is_some());

        clear_env();
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_infant_bootstrap_with_config_capabilities() {
        clear_env();
        std::env::set_var("PRIMAL_CAPABILITIES", "signing,anchoring,session_events");

        let config = BootstrapConfig {
            storage: crate::factory::StorageConfig {
                backend: "memory".to_string(),
                ..Default::default()
            },
            primal_name: None,
            instance_id: None,
        };

        let result = infant_bootstrap_with_config(config)
            .await
            .expect("bootstrap");
        assert_eq!(result.self_knowledge.capabilities.len(), 3);

        clear_env();
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_infant_bootstrap_with_config_invalid_storage() {
        clear_env();

        let config = BootstrapConfig {
            storage: crate::factory::StorageConfig {
                backend: "invalid_backend".to_string(),
                ..Default::default()
            },
            primal_name: None,
            instance_id: None,
        };

        let result = infant_bootstrap_with_config(config).await;
        assert!(result.is_err());
        if let Err(err) = result {
            assert!(err.to_string().contains("Unknown storage backend"));
        }
    }

    #[test]
    fn test_bootstrap_config_default() {
        let config = BootstrapConfig::default();
        assert!(config.storage.backend.is_empty());
        assert!(config.primal_name.is_none());
        assert!(config.instance_id.is_none());
    }

    #[test]
    fn test_bootstrap_error_display() {
        let err = BootstrapError::SelfKnowledge("bad config".to_string());
        assert!(err.to_string().contains("bad config"));

        let err = BootstrapError::Discovery("no service".to_string());
        assert!(err.to_string().contains("no service"));
    }
}
