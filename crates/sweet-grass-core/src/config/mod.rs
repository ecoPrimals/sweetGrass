// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! `SweetGrass` configuration.
//!
//! Configuration for the `SweetGrass` primal, including compression settings,
//! query configuration, and integration options.
//!
//! ## Capability-Based Discovery
//!
//! `SweetGrass` uses runtime discovery to find other primals. It has no hardcoded
//! knowledge of specific primal locations - only the capabilities it requires.
//!
//! ```rust,ignore
//! let config = SweetGrassConfig::builder()
//!     .require_capability(Capability::Signing)
//!     .require_capability(Capability::Anchoring)
//!     .build();
//! ```

mod capability;

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

use crate::identity;

pub use capability::Capability;

/// Default maximum depth for provenance graph traversal and attribution chains.
///
/// Shared across `QueryEngine`, `ProvenanceGraphBuilder`, and
/// `AttributionCalculator` to prevent drift between components.
pub const DEFAULT_MAX_PROVENANCE_DEPTH: u32 = 10;

/// Network configuration for capability-based discovery.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct NetworkConfig {
    /// Listen address for tarpc service (primary protocol).
    /// If None, binds to an available port.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tarpc_listen: Option<String>,

    /// Listen address for REST API (fallback/debug).
    /// If None, REST API is disabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rest_listen: Option<String>,

    /// Required capabilities - discovery will find primals offering these.
    #[serde(default)]
    pub required_capabilities: Vec<Capability>,

    /// Capabilities this primal offers to others.
    #[serde(default)]
    pub offered_capabilities: Vec<Capability>,

    /// Discovery service endpoint (bootstrap only, optional).
    /// If None, uses multicast/mDNS for local discovery.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discovery_bootstrap: Option<String>,

    /// Connection timeout for primal-to-primal communication.
    #[serde(with = "humantime_serde")]
    pub connection_timeout: Duration,

    /// Maximum retries for capability discovery.
    pub discovery_retries: u32,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            tarpc_listen: None, // Bind to available port
            rest_listen: None,  // Disabled by default
            required_capabilities: vec![
                Capability::Signing,       // Need signing capability
                Capability::Anchoring,     // Need anchoring capability
                Capability::SessionEvents, // Need session events
            ],
            offered_capabilities: vec![
                Capability::Custom("attribution".to_string()),
                Capability::Custom("provenance".to_string()),
            ],
            discovery_bootstrap: None, // Use local discovery
            connection_timeout: Duration::from_secs(10),
            discovery_retries: 3,
        }
    }
}

/// Default primal name for config deserialization.
fn default_name() -> String {
    identity::PRIMAL_DISPLAY_NAME.to_string()
}

/// Configuration for `SweetGrass`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct SweetGrassConfig {
    /// Primal name.
    #[serde(default = "default_name")]
    pub name: String,

    /// Primal instance ID (auto-generated if None).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,

    /// Network and discovery configuration.
    pub network: NetworkConfig,

    /// Storage configuration.
    pub storage: StorageConfig,

    /// Compression configuration.
    pub compression: CompressionConfig,

    /// Query configuration.
    pub query: QueryConfig,

    /// Attribution configuration.
    pub attribution: AttributionConfig,

    /// Listener configuration.
    pub listener: ListenerConfig,
}

impl Default for SweetGrassConfig {
    fn default() -> Self {
        Self {
            name: identity::PRIMAL_DISPLAY_NAME.to_string(),
            instance_id: None,
            network: NetworkConfig::default(),
            storage: StorageConfig::default(),
            compression: CompressionConfig::default(),
            query: QueryConfig::default(),
            attribution: AttributionConfig::default(),
            listener: ListenerConfig::default(),
        }
    }
}

impl SweetGrassConfig {
    /// Create a new configuration builder.
    #[must_use]
    pub fn builder() -> SweetGrassConfigBuilder {
        SweetGrassConfigBuilder::default()
    }

    /// Load configuration from environment variables.
    /// Uses SWEETGRASS_ prefix for all variables.
    ///
    /// # Errors
    ///
    /// This function always succeeds currently, but returns `Result` for forward
    /// compatibility with validation errors.
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut config = Self::default();
        Self::apply_env_overrides_from_reader(&mut config, &|key| std::env::var(key).ok());
        Ok(config)
    }

    /// Load configuration from a specific TOML file.
    ///
    /// # Errors
    ///
    /// Returns error if the file cannot be read or parsed.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let contents = std::fs::read_to_string(path).map_err(ConfigError::Io)?;
        let config: Self = toml::from_str(&contents)
            .map_err(|e| ConfigError::Parse(format!("{}: {e}", path.display())))?;
        Ok(config)
    }

    /// Load configuration with full hierarchy: env vars > config file > defaults.
    ///
    /// Config file locations (checked in order):
    /// 1. `SWEETGRASS_CONFIG` environment variable
    /// 2. `$XDG_CONFIG_HOME/sweetgrass/config.toml`
    /// 3. `~/.config/sweetgrass/config.toml`
    ///
    /// Values from environment variables override values from the config file.
    ///
    /// # Errors
    ///
    /// Returns error if the config file exists but cannot be parsed.
    pub fn load() -> Result<Self, ConfigError> {
        Self::load_with_reader(|key| std::env::var(key).ok())
    }

    /// Load configuration using an injectable key reader (DI-friendly).
    ///
    /// Tests inject a closure instead of mutating process-global env vars.
    ///
    /// # Errors
    ///
    /// Returns error if the config file exists but cannot be parsed.
    pub fn load_with_reader(reader: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let mut config = match Self::find_config_path_with_reader(&reader) {
            Some(path) => Self::from_file(path)?,
            None => Self::default(),
        };

        Self::apply_env_overrides_from_reader(&mut config, &reader);
        Ok(config)
    }

    fn find_config_path_with_reader(
        reader: &impl Fn(&str) -> Option<String>,
    ) -> Option<std::path::PathBuf> {
        if let Some(path) = reader("SWEETGRASS_CONFIG") {
            let p = std::path::PathBuf::from(path);
            if p.is_file() {
                return Some(p);
            }
        }

        if let Some(xdg) = reader("XDG_CONFIG_HOME") {
            let p = std::path::PathBuf::from(xdg)
                .join(crate::identity::PRIMAL_NAME)
                .join("config.toml");
            if p.is_file() {
                return Some(p);
            }
        }

        if let Some(home) = reader("HOME") {
            let p = std::path::PathBuf::from(home)
                .join(".config")
                .join(crate::identity::PRIMAL_NAME)
                .join("config.toml");
            if p.is_file() {
                return Some(p);
            }
        }

        None
    }

    fn apply_env_overrides_from_reader(
        config: &mut Self,
        reader: &impl Fn(&str) -> Option<String>,
    ) {
        if let Some(name) = reader("SWEETGRASS_NAME") {
            config.name = name;
        }

        if let Some(tarpc) = reader("SWEETGRASS_TARPC_LISTEN") {
            config.network.tarpc_listen = Some(tarpc);
        }

        if let Some(rest) = reader("SWEETGRASS_REST_LISTEN") {
            config.network.rest_listen = Some(rest);
        }

        if let Some(bootstrap) = reader("SWEETGRASS_DISCOVERY_BOOTSTRAP") {
            config.network.discovery_bootstrap = Some(bootstrap);
        }
    }
}

/// Configuration error types.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ConfigError {
    /// Invalid configuration value.
    #[error("invalid configuration: {0}")]
    Invalid(String),

    /// Missing required configuration.
    #[error("missing required configuration: {0}")]
    Missing(String),

    /// Environment variable error.
    #[error("environment error: {0}")]
    Environment(String),

    /// I/O error reading config file.
    #[error("config file I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Config file parse error.
    #[error("config file parse error: {0}")]
    Parse(String),
}

/// Builder for `SweetGrass` configuration.
#[derive(Default)]
pub struct SweetGrassConfigBuilder {
    config: SweetGrassConfig,
}

impl SweetGrassConfigBuilder {
    /// Set the primal name.
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.config.name = name.into();
        self
    }

    /// Require a capability from other primals.
    #[must_use]
    pub fn require_capability(mut self, capability: Capability) -> Self {
        self.config.network.required_capabilities.push(capability);
        self
    }

    /// Offer a capability to other primals.
    #[must_use]
    pub fn offer_capability(mut self, capability: Capability) -> Self {
        self.config.network.offered_capabilities.push(capability);
        self
    }

    /// Set tarpc listen address.
    #[must_use]
    pub fn tarpc_listen(mut self, addr: impl Into<String>) -> Self {
        self.config.network.tarpc_listen = Some(addr.into());
        self
    }

    /// Set REST listen address.
    #[must_use]
    pub fn rest_listen(mut self, addr: impl Into<String>) -> Self {
        self.config.network.rest_listen = Some(addr.into());
        self
    }

    /// Set storage backend.
    #[must_use]
    pub fn storage_backend(mut self, backend: StorageBackend) -> Self {
        self.config.storage.backend = backend;
        self
    }

    /// Build the configuration.
    #[must_use]
    pub fn build(self) -> SweetGrassConfig {
        self.config
    }
}

/// Storage backend configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct StorageConfig {
    /// Storage backend type.
    pub backend: StorageBackend,

    /// Connection string (for database backends).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_string: Option<String>,

    /// Path (for file-based backends).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Maximum Braids to cache in memory.
    pub cache_size: usize,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            backend: StorageBackend::Memory,
            connection_string: None,
            path: None,
            cache_size: 10_000,
        }
    }
}

/// Storage backend types.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum StorageBackend {
    /// In-memory storage (for testing/development).
    Memory,
    /// File-based storage.
    File,
    /// `PostgreSQL` with graph extension.
    Postgres,
    /// `Oxigraph` RDF store.
    Oxigraph,
    /// Custom backend.
    Custom(String),
}

/// Compression configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct CompressionConfig {
    /// Minimum vertices for compression (below = single or none).
    pub min_vertices: usize,

    /// Threshold for splitting into multiple Braids.
    pub split_threshold: usize,

    /// Threshold for hierarchical compression.
    pub hierarchical_threshold: usize,

    /// Coherence threshold for single Braid (0.0 - 1.0).
    pub coherence_threshold: f64,

    /// Maximum Braids per session.
    pub max_braids_per_session: usize,

    /// Enable meta-Braid generation.
    pub generate_summaries: bool,

    /// Maximum summary depth.
    pub max_summary_depth: u32,

    /// Honor compression hints from sessions.
    pub honor_hints: bool,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            min_vertices: 1,
            split_threshold: 100,
            hierarchical_threshold: 1000,
            coherence_threshold: 0.7,
            max_braids_per_session: 100,
            generate_summaries: true,
            max_summary_depth: 3,
            honor_hints: true,
        }
    }
}

/// Query configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct QueryConfig {
    /// Enable GraphQL API.
    pub graphql: bool,

    /// Enable SPARQL API.
    pub sparql: bool,

    /// Enable full-text search.
    pub full_text_search: bool,

    /// Maximum query depth for provenance graphs.
    pub max_provenance_depth: u32,

    /// Query timeout.
    #[serde(with = "humantime_serde")]
    pub timeout: Duration,

    /// Maximum results per query.
    pub max_results: usize,
}

impl Default for QueryConfig {
    fn default() -> Self {
        Self {
            graphql: true,
            sparql: false,
            full_text_search: true,
            max_provenance_depth: DEFAULT_MAX_PROVENANCE_DEPTH,
            timeout: Duration::from_secs(30),
            max_results: 1000,
        }
    }
}

/// Attribution calculation configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct AttributionConfig {
    /// Maximum depth to traverse provenance graph.
    pub max_depth: u32,

    /// Decay factor for inherited contributions (0.0 - 1.0).
    pub inheritance_decay: f64,

    /// Minimum share threshold (below this = 0).
    pub min_share_threshold: f64,

    /// Include resource-based attribution.
    pub include_resources: bool,

    /// Compute contribution weight.
    pub compute_weight: f64,

    /// Storage contribution weight.
    pub storage_weight: f64,

    /// Data contribution weight.
    pub data_weight: f64,

    /// Network contribution weight.
    pub network_weight: f64,
}

impl Default for AttributionConfig {
    fn default() -> Self {
        Self {
            max_depth: DEFAULT_MAX_PROVENANCE_DEPTH,
            inheritance_decay: 0.7,
            min_share_threshold: 0.001, // 0.1%
            include_resources: true,
            compute_weight: 0.3,
            storage_weight: 0.2,
            data_weight: 0.4,
            network_weight: 0.1,
        }
    }
}

/// Listener configuration for ecosystem events.
///
/// Uses capability-based configuration instead of primal names.
/// Each capability is discovered at runtime via the universal adapter.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ListenerConfig {
    /// Enable session event listener (discovers `SessionEvents` capability).
    pub session_events: bool,

    /// Enable anchoring listener (discovers `Anchoring` capability).
    pub anchoring: bool,

    /// Enable compute listener (discovers `Compute` capability).
    pub compute: bool,

    /// Buffer size for event processing.
    pub buffer_size: usize,

    /// Event processing batch size.
    pub batch_size: usize,

    /// Reconnection delay on failure.
    #[serde(with = "humantime_serde")]
    pub reconnect_delay: Duration,
}

impl Default for ListenerConfig {
    fn default() -> Self {
        Self {
            session_events: true,
            anchoring: true,
            compute: true,
            buffer_size: 1000,
            batch_size: 100,
            reconnect_delay: Duration::from_secs(5),
        }
    }
}

// Custom serde module for Duration
mod humantime_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}s", duration.as_secs()))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let s = s.trim_end_matches('s');
        let secs: u64 = s.parse().map_err(serde::de::Error::custom)?;
        Ok(Duration::from_secs(secs))
    }
}

mod tests;
