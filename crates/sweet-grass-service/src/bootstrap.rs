// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Infant Discovery Bootstrap.
//!
//! The entry point for zero-knowledge startup. A primal is born knowing
//! only itself and discovers all other services at runtime.

use std::sync::Arc;

use tracing::{debug, info, instrument};

use crate::factory::{BraidStoreFactory, StorageConfig};
use crate::state::AppState;
use sweet_grass_core::SelfKnowledge;
use sweet_grass_core::agent::Did;

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
#[non_exhaustive]
pub enum BootstrapError {
    /// Failed to establish self-knowledge from environment.
    #[error("Self-knowledge error: {0}")]
    SelfKnowledge(#[from] sweet_grass_core::BootstrapEnvError),

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
/// - `STORAGE_BACKEND`: Backend type (`memory`, `redb`, `postgres`, `nestgate`)
/// - `STORAGE_PATH`: Path for file-backed stores (`redb`)
/// - `DATABASE_URL`: `PostgreSQL` connection string
/// - `NESTGATE_SOCKET`: `NestGate` UDS socket path
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
    let self_knowledge = SelfKnowledge::from_env()?;

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
    let mut app_state = AppState::with_self_knowledge(
        Arc::new(store),
        default_agent.clone(),
        self_knowledge.clone(),
        backend_type,
    );

    // Phase 4b: Resolve crypto delegate for Tower-delegated braid signing
    #[cfg(unix)]
    {
        app_state = resolve_crypto_delegate(app_state);
    }

    // Phase 5: Announce capabilities to discovery service (IPC v3.1)
    announce_capabilities(&self_knowledge, &app_state).await;

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
    let reader = |key: &str| std::env::var(key).ok();
    infant_bootstrap_with_config_and_reader(config, reader).await
}

/// Fully DI-friendly bootstrap: explicit storage config + env reader.
///
/// Tests use this to avoid `unsafe` env var mutation entirely.
///
/// # Errors
///
/// Returns error if self-knowledge or storage initialization fails.
#[instrument(skip(config, reader))]
pub async fn infant_bootstrap_with_config_and_reader(
    config: BootstrapConfig,
    reader: impl Fn(&str) -> Option<String>,
) -> Result<BootstrapResult, BootstrapError> {
    info!("Infant Bootstrap: Starting with explicit configuration");

    debug!("Phase 1: Establishing self-knowledge");
    let self_knowledge = SelfKnowledge::from_reader(reader)?;

    info!(
        primal_name = %self_knowledge.name,
        instance_id = %self_knowledge.instance_id,
        capabilities = ?self_knowledge.capabilities,
        "Self-knowledge established"
    );

    debug!("Phase 2: Initializing storage from explicit config");
    let (store, backend_type) = BraidStoreFactory::from_config_with_name(&config.storage).await?;

    info!(backend = backend_type, "Storage backend initialized");

    debug!("Phase 3: Establishing default agent identity");
    let default_agent = Did::new(format!("did:primal:{}", self_knowledge.instance_id));

    debug!("Phase 4: Creating application state");
    let mut app_state = AppState::with_self_knowledge(
        Arc::new(store),
        default_agent.clone(),
        self_knowledge.clone(),
        backend_type,
    );

    #[cfg(unix)]
    {
        app_state = resolve_crypto_delegate(app_state);
    }

    // Phase 5: Announce capabilities to discovery service (IPC v3.1)
    announce_capabilities(&self_knowledge, &app_state).await;

    info!(
        primal_name = %self_knowledge.name,
        default_agent = %default_agent,
        "Infant Bootstrap complete"
    );

    Ok(BootstrapResult {
        self_knowledge,
        app_state,
        default_agent,
    })
}

/// Resolve the `BearDog` crypto delegate and attach to `AppState`.
///
/// Attempts to find a `BearDog` socket via environment variables. When
/// found, Tower-delegated Ed25519 signing is enabled on `braid.create`.
/// Falls back gracefully to unsigned braids when unavailable.
#[cfg(unix)]
fn resolve_crypto_delegate(app_state: AppState) -> AppState {
    if let Some(crypto) = crate::crypto_delegate::CryptoDelegate::resolve() {
        tracing::info!(
            socket = %crypto.socket_path().display(),
            "crypto delegate: Tower signing enabled via BearDog"
        );
        app_state.with_crypto(crypto)
    } else {
        tracing::info!("crypto delegate: no BearDog socket found, braids will be unsigned");
        app_state
    }
}

/// Resolve the advertise host for discovery announcements (DI-friendly).
///
/// Uses `PRIMAL_ADVERTISE_ADDRESS` if set via the reader, otherwise falls
/// back to the system hostname. `0.0.0.0` is never announced — it's a
/// listen wildcard, not a routable address.
fn resolve_advertise_host(reader: &impl Fn(&str) -> Option<String>) -> String {
    reader(sweet_grass_core::primal_names::env_vars::PRIMAL_ADVERTISE_ADDRESS).unwrap_or_else(
        || {
            hostname::get()
                .ok()
                .and_then(|h| h.into_string().ok())
                .unwrap_or_else(|| sweet_grass_core::identity::FALLBACK_ADVERTISE_HOST.to_string())
        },
    )
}

/// Build an advertise address from the resolved host and a port.
///
/// Returns `None` when the port is `0` (not yet bound / auto-allocate).
fn advertise_address(reader: &impl Fn(&str) -> Option<String>, port: u16) -> Option<String> {
    if port > 0 {
        Some(format!("{}:{port}", resolve_advertise_host(reader)))
    } else {
        None
    }
}

/// Announce this primal's capabilities to the discovery service.
///
/// Per IPC v3.1 §Discovery, primals SHOULD register with Songbird (or any
/// primal offering `Capability::Discovery`) on startup.  When the discovery
/// service is unreachable the primal continues in standalone mode — this is
/// graceful degradation per the `PRIMAL_IPC_PROTOCOL` v3.1 standalone
/// startup requirement.
async fn announce_capabilities(self_knowledge: &SelfKnowledge, state: &AppState) {
    use sweet_grass_integration::discovery::{DiscoveredPrimal, PrimalDiscovery, create_discovery};

    if self_knowledge.capabilities.is_empty() {
        debug!("No capabilities to announce — standalone mode");
        return;
    }

    let discovery = create_discovery().await;
    let env_reader = |key: &str| std::env::var(key).ok();

    let primal = DiscoveredPrimal {
        instance_id: self_knowledge.instance_id.clone(),
        name: self_knowledge.name.clone(),
        capabilities: self_knowledge.capabilities.clone(),
        tarpc_address: state
            .self_knowledge
            .as_ref()
            .and_then(|sk| advertise_address(&env_reader, sk.tarpc_port)),
        rest_address: state
            .self_knowledge
            .as_ref()
            .and_then(|sk| advertise_address(&env_reader, sk.rest_port)),
        last_seen: std::time::SystemTime::now(),
        healthy: true,
    };

    if let Err(e) = discovery.announce(&primal).await {
        debug!(
            error = %e,
            "Discovery service unreachable — continuing in standalone mode \
             (IPC v3.1 §Standalone Startup)"
        );
        return;
    }

    info!(
        capabilities = ?self_knowledge.capabilities,
        "Announced capabilities to discovery service"
    );
}

/// Create application state with runtime-discovered storage.
///
/// Test/example convenience that avoids full bootstrap.
/// Reads `SWEETGRASS_AGENT_DID` from the environment; falls back to
/// a test DID when the variable is absent.
///
/// # Errors
///
/// Returns error if storage initialization fails.
#[cfg(test)]
pub async fn create_app_state_from_env() -> Result<AppState, BootstrapError> {
    let store = BraidStoreFactory::from_env().await?;
    let did_str =
        std::env::var("SWEETGRASS_AGENT_DID").unwrap_or_else(|_| "did:primal:test".to_string());
    let default_agent = Did::new(&did_str);
    Ok(AppState::with_store(Arc::new(store), default_agent))
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "test module: expect is standard in tests"
)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use super::*;
    use sweet_grass_core::Capability;

    fn mock_reader(vars: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> = vars
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect();
        move |key: &str| map.get(key).cloned()
    }

    fn empty_reader() -> impl Fn(&str) -> Option<String> {
        |_: &str| None
    }

    fn memory_config() -> BootstrapConfig {
        BootstrapConfig {
            storage: crate::factory::StorageConfig {
                backend: "memory".to_string(),
                ..Default::default()
            },
            primal_name: None,
            instance_id: None,
        }
    }

    #[tokio::test]
    async fn test_infant_bootstrap_defaults() {
        let result = infant_bootstrap_with_config_and_reader(memory_config(), empty_reader())
            .await
            .expect("should bootstrap");

        assert_eq!(result.self_knowledge.name, "sweetgrass");
        assert!(!result.self_knowledge.instance_id.is_empty());
        assert!(result.default_agent.as_str().starts_with("did:primal:"));
        assert!(Arc::strong_count(&result.app_state.store) >= 1);
    }

    #[tokio::test]
    async fn test_infant_bootstrap_with_custom_identity() {
        let reader = mock_reader(&[
            ("PRIMAL_NAME", "sweetgrass-test"),
            ("PRIMAL_INSTANCE_ID", "test-123"),
            ("PRIMAL_CAPABILITIES", "signing,anchoring"),
        ]);

        let result = infant_bootstrap_with_config_and_reader(memory_config(), reader)
            .await
            .expect("should bootstrap");

        assert_eq!(result.self_knowledge.name, "sweetgrass-test");
        assert_eq!(result.self_knowledge.instance_id, "test-123");
        assert_eq!(result.self_knowledge.capabilities.len(), 2);
        assert!(result.self_knowledge.offers(&Capability::Signing));
        assert_eq!(result.default_agent.as_str(), "did:primal:test-123");
    }

    #[tokio::test]
    async fn test_create_app_state_from_env() {
        let app_state = create_app_state_from_env()
            .await
            .expect("should create app state");
        assert!(Arc::strong_count(&app_state.store) >= 1);
    }

    #[tokio::test]
    async fn test_infant_bootstrap_with_explicit_config_memory() {
        let result = infant_bootstrap_with_config_and_reader(memory_config(), empty_reader())
            .await
            .expect("should bootstrap with config");

        assert_eq!(result.self_knowledge.name, "sweetgrass");
        assert_eq!(result.app_state.store_backend, "memory");
        assert!(Arc::strong_count(&result.app_state.store) >= 1);
    }

    #[tokio::test]
    async fn test_infant_bootstrap_with_explicit_config_primal_identity() {
        let reader = mock_reader(&[
            ("PRIMAL_NAME", "sg-config-test"),
            ("PRIMAL_INSTANCE_ID", "cfg-instance-42"),
        ]);

        let result = infant_bootstrap_with_config_and_reader(memory_config(), reader)
            .await
            .expect("bootstrap");
        assert_eq!(result.self_knowledge.name, "sg-config-test");
        assert_eq!(result.self_knowledge.instance_id, "cfg-instance-42");
        assert_eq!(result.default_agent.as_str(), "did:primal:cfg-instance-42");
        assert!(result.app_state.self_knowledge.is_some());
    }

    #[tokio::test]
    async fn test_infant_bootstrap_with_config_capabilities() {
        let reader = mock_reader(&[("PRIMAL_CAPABILITIES", "signing,anchoring,session_events")]);

        let result = infant_bootstrap_with_config_and_reader(memory_config(), reader)
            .await
            .expect("bootstrap");
        assert_eq!(result.self_knowledge.capabilities.len(), 3);
    }

    #[tokio::test]
    async fn test_infant_bootstrap_with_config_invalid_storage() {
        let config = BootstrapConfig {
            storage: crate::factory::StorageConfig {
                backend: "invalid_backend".to_string(),
                ..Default::default()
            },
            primal_name: None,
            instance_id: None,
        };

        let result = infant_bootstrap_with_config_and_reader(config, empty_reader()).await;
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
        let err = BootstrapError::from(sweet_grass_core::BootstrapEnvError::InvalidPort {
            var_name: "TARPC_PORT".to_string(),
            value: "bad".to_string(),
        });
        assert!(err.to_string().contains("TARPC_PORT"));

        let err = BootstrapError::Discovery("no service".to_string());
        assert!(err.to_string().contains("no service"));
    }
}
