//! Infant Discovery Bootstrap.
//!
//! The entry point for zero-knowledge startup. A primal is born knowing
//! only itself and discovers all other services at runtime.

use std::sync::Arc;
use tracing::{debug, info, instrument};

use sweet_grass_core::agent::Did;
use sweet_grass_core::SelfKnowledge;
use sweet_grass_store::BraidStore;

use crate::factory::BraidStoreFactory;
use crate::state::AppState;

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
/// - `REST_PORT`: REST endpoint port (default: 8080)
///
/// ### Storage
/// - `DATABASE_URL`: PostgreSQL connection string
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

    // Phase 2: Discover and initialize storage backend
    debug!("Phase 2: Discovering storage backend from environment");
    let (store, backend_type): (Arc<dyn BraidStore>, &str) =
        if std::env::var("DATABASE_URL").ok().is_some() {
            info!("Initializing PostgreSQL backend");
            (BraidStoreFactory::from_env().await?, "postgres")
        } else if std::env::var("SLED_PATH").is_ok() {
            info!("Initializing Sled backend");
            (BraidStoreFactory::from_env().await?, "sled")
        } else {
            info!("Initializing Memory backend (no DATABASE_URL or SLED_PATH)");
            (BraidStoreFactory::from_env().await?, "memory")
        };

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
        backend_type,
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
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use sweet_grass_core::Capability;

    // NOTE: These tests modify environment variables and should be run with --test-threads=1

    #[tokio::test]
    #[serial_test::serial]
    async fn test_infant_bootstrap_defaults() {
        // Clear environment for clean test
        std::env::remove_var("PRIMAL_NAME");
        std::env::remove_var("PRIMAL_INSTANCE_ID");
        std::env::remove_var("PRIMAL_CAPABILITIES");
        std::env::remove_var("DATABASE_URL");
        std::env::remove_var("SLED_PATH");

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
        std::env::set_var("PRIMAL_NAME", "sweetgrass-test");
        std::env::set_var("PRIMAL_INSTANCE_ID", "test-123");
        std::env::set_var("PRIMAL_CAPABILITIES", "signing,anchoring");

        let result = infant_bootstrap().await.expect("should bootstrap");

        assert_eq!(result.self_knowledge.name, "sweetgrass-test");
        assert_eq!(result.self_knowledge.instance_id, "test-123");
        assert_eq!(result.self_knowledge.capabilities.len(), 2);
        assert!(result.self_knowledge.offers(&Capability::Signing));
        assert_eq!(result.default_agent.as_str(), "did:primal:test-123");

        // Cleanup
        std::env::remove_var("PRIMAL_NAME");
        std::env::remove_var("PRIMAL_INSTANCE_ID");
        std::env::remove_var("PRIMAL_CAPABILITIES");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_create_app_state_from_env() {
        std::env::remove_var("DATABASE_URL");
        std::env::remove_var("SLED_PATH");

        let app_state = create_app_state_from_env()
            .await
            .expect("should create app state");

        // Verify app state has a store
        assert!(Arc::strong_count(&app_state.store) >= 1);
    }
}
