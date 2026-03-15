// SPDX-License-Identifier: AGPL-3.0-only
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

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

use crate::identity;

/// Capabilities that `SweetGrass` can require from other primals.
/// Runtime discovery finds primals offering these capabilities.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    /// DID-based signing (offered by identity primals)
    Signing,
    /// Permanent data anchoring (offered by persistence primals)
    Anchoring,
    /// Session event streaming (offered by activity primals)
    SessionEvents,
    /// Service discovery (offered by orchestration primals)
    Discovery,
    /// Compute task execution (offered by compute primals)
    Compute,
    /// Custom capability with identifier
    Custom(String),
}

impl Capability {
    /// Create a custom capability.
    #[must_use]
    pub fn custom(name: impl Into<String>) -> Self {
        Self::Custom(name.into())
    }

    /// Parse a capability from a string representation.
    #[must_use]
    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "signing" => Some(Self::Signing),
            "anchoring" => Some(Self::Anchoring),
            "sessionevents" | "session_events" | "session-events" => Some(Self::SessionEvents),
            "discovery" => Some(Self::Discovery),
            "compute" => Some(Self::Compute),
            other => {
                if other.starts_with("custom:") {
                    Some(Self::Custom(
                        other.strip_prefix("custom:").unwrap_or(other).to_string(),
                    ))
                } else if !other.is_empty() {
                    Some(Self::Custom(other.to_string()))
                } else {
                    None
                }
            },
        }
    }
}

impl std::fmt::Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Signing => write!(f, "signing"),
            Self::Anchoring => write!(f, "anchoring"),
            Self::SessionEvents => write!(f, "session_events"),
            Self::Discovery => write!(f, "discovery"),
            Self::Compute => write!(f, "compute"),
            Self::Custom(name) => write!(f, "custom:{name}"),
        }
    }
}

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
        Self::apply_env_overrides(&mut config);
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
        let mut config = match Self::find_config_path() {
            Some(path) => Self::from_file(path)?,
            None => Self::default(),
        };

        Self::apply_env_overrides(&mut config);
        Ok(config)
    }

    /// Find config file path using standard locations.
    fn find_config_path() -> Option<std::path::PathBuf> {
        if let Ok(path) = std::env::var("SWEETGRASS_CONFIG") {
            let p = std::path::PathBuf::from(path);
            if p.is_file() {
                return Some(p);
            }
        }

        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            let p = std::path::PathBuf::from(xdg)
                .join("sweetgrass")
                .join("config.toml");
            if p.is_file() {
                return Some(p);
            }
        }

        if let Ok(home) = std::env::var("HOME") {
            let p = std::path::PathBuf::from(home)
                .join(".config")
                .join("sweetgrass")
                .join("config.toml");
            if p.is_file() {
                return Some(p);
            }
        }

        None
    }

    /// Apply environment variable overrides to config.
    fn apply_env_overrides(config: &mut Self) {
        if let Ok(name) = std::env::var("SWEETGRASS_NAME") {
            config.name = name;
        }

        if let Ok(tarpc) = std::env::var("SWEETGRASS_TARPC_LISTEN") {
            config.network.tarpc_listen = Some(tarpc);
        }

        if let Ok(rest) = std::env::var("SWEETGRASS_REST_LISTEN") {
            config.network.rest_listen = Some(rest);
        }

        if let Ok(bootstrap) = std::env::var("SWEETGRASS_DISCOVERY_BOOTSTRAP") {
            config.network.discovery_bootstrap = Some(bootstrap);
        }
    }
}

/// Configuration error types.
#[derive(Debug, thiserror::Error)]
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
            max_provenance_depth: 10,
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
            max_depth: 10,
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

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "test module: expect/unwrap are standard in tests"
)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SweetGrassConfig::default();
        assert_eq!(config.name, "SweetGrass");
        assert_eq!(config.compression.split_threshold, 100);
        assert!((config.attribution.inheritance_decay - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn test_storage_backend_serialization() {
        let backend = StorageBackend::Postgres;
        let json = serde_json::to_string(&backend).expect("should serialize");
        assert_eq!(json, "\"postgres\"");

        let parsed: StorageBackend = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(parsed, StorageBackend::Postgres);
    }

    #[test]
    fn test_config_serialization() {
        let config = SweetGrassConfig::default();
        let json = serde_json::to_string_pretty(&config).expect("should serialize");
        assert!(json.contains("SweetGrass"));
        assert!(json.contains("compression"));

        let parsed: SweetGrassConfig = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(parsed.name, config.name);
    }

    #[test]
    fn test_compression_config_defaults() {
        let config = CompressionConfig::default();
        assert_eq!(config.min_vertices, 1);
        assert_eq!(config.split_threshold, 100);
        assert!((config.coherence_threshold - 0.7).abs() < f64::EPSILON);
        assert!(config.generate_summaries);
    }

    #[test]
    fn test_attribution_config_defaults() {
        let config = AttributionConfig::default();
        assert_eq!(config.max_depth, 10);
        assert!((config.inheritance_decay - 0.7).abs() < f64::EPSILON);
        assert!((config.min_share_threshold - 0.001).abs() < f64::EPSILON);
    }

    #[test]
    fn test_query_timeout_serialization() {
        let config = QueryConfig::default();
        let json = serde_json::to_string(&config).expect("should serialize");
        assert!(json.contains("30s"));
    }

    #[test]
    fn test_capability_based_config() {
        let config = SweetGrassConfig::builder()
            .name("TestPrimal")
            .require_capability(Capability::Signing)
            .offer_capability(Capability::custom("test_capability"))
            .tarpc_listen("0.0.0.0:0")
            .build();

        assert_eq!(config.name, "TestPrimal");
        assert!(config
            .network
            .required_capabilities
            .contains(&Capability::Signing));
        assert!(config
            .network
            .offered_capabilities
            .contains(&Capability::Custom("test_capability".to_string())));
    }

    #[test]
    fn test_network_config_defaults() {
        let config = NetworkConfig::default();
        assert!(config.tarpc_listen.is_none());
        assert!(config.rest_listen.is_none());
        assert!(config.discovery_bootstrap.is_none());
        assert!(!config.required_capabilities.is_empty());
    }

    #[test]
    fn test_config_from_env() {
        // This test verifies the from_env method exists and returns Ok
        // Actual env var testing would require setup/teardown
        let result = SweetGrassConfig::from_env();
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_from_file() {
        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join("sweetgrass_test_config.toml");
        let toml_content = r#"
name = "FromFile"
[compression]
min_vertices = 42
split_threshold = 200
"#;
        std::fs::write(&config_path, toml_content).expect("write temp config");
        let result = SweetGrassConfig::from_file(&config_path);
        let _ = std::fs::remove_file(&config_path);
        let config = result.expect("should parse valid TOML");
        assert_eq!(config.name, "FromFile");
        assert_eq!(config.compression.min_vertices, 42);
        assert_eq!(config.compression.split_threshold, 200);
    }

    #[test]
    #[serial_test::serial]
    fn test_load_env_overrides_file() {
        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join("sweetgrass_test_env_override.toml");
        let toml_content = r#"
name = "FromFile"
[network]
tarpc_listen = "127.0.0.1:9999"
"#;
        std::fs::write(&config_path, toml_content).expect("write temp config");
        std::env::set_var("SWEETGRASS_CONFIG", config_path.as_os_str());
        std::env::set_var("SWEETGRASS_NAME", "FromEnv");
        let result = SweetGrassConfig::load();
        std::env::remove_var("SWEETGRASS_CONFIG");
        std::env::remove_var("SWEETGRASS_NAME");
        let _ = std::fs::remove_file(&config_path);
        let config = result.expect("load should succeed");
        assert_eq!(config.name, "FromEnv", "env var should override file");
        assert_eq!(
            config.network.tarpc_listen.as_deref(),
            Some("127.0.0.1:9999"),
            "file value should be used when env not set"
        );
    }

    #[test]
    #[serial_test::serial]
    fn test_load_missing_file_returns_defaults() {
        let temp_dir = std::env::temp_dir().join("sweetgrass_test_no_config");
        let config_subdir = temp_dir.join(".config").join("sweetgrass");
        std::fs::create_dir_all(&config_subdir).ok();
        let old_xdg = std::env::var("XDG_CONFIG_HOME").ok();
        let old_home = std::env::var("HOME").ok();
        let old_sweetgrass_config = std::env::var("SWEETGRASS_CONFIG").ok();
        std::env::remove_var("SWEETGRASS_CONFIG");
        std::env::set_var("XDG_CONFIG_HOME", &temp_dir);
        std::env::set_var("HOME", &temp_dir);
        let result = SweetGrassConfig::load();
        if let Some(x) = old_xdg {
            std::env::set_var("XDG_CONFIG_HOME", x);
        } else {
            std::env::remove_var("XDG_CONFIG_HOME");
        }
        if let Some(h) = old_home {
            std::env::set_var("HOME", h);
        } else {
            std::env::remove_var("HOME");
        }
        if let Some(c) = old_sweetgrass_config {
            std::env::set_var("SWEETGRASS_CONFIG", c);
        } else {
            std::env::remove_var("SWEETGRASS_CONFIG");
        }
        let _ = std::fs::remove_dir_all(&temp_dir);
        let config = result.expect("load should succeed with no file");
        assert_eq!(config.name, identity::PRIMAL_DISPLAY_NAME);
        assert_eq!(config.compression.split_threshold, 100);
    }

    #[test]
    fn test_from_file_invalid_toml_returns_error() {
        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join("sweetgrass_test_invalid.toml");
        std::fs::write(&config_path, "name = [ invalid toml").expect("write invalid");
        let result = SweetGrassConfig::from_file(&config_path);
        let _ = std::fs::remove_file(&config_path);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::Parse(_)));
    }

    #[test]
    fn test_capability_from_string() {
        assert_eq!(
            Capability::from_string("signing"),
            Some(Capability::Signing)
        );
        assert_eq!(
            Capability::from_string("SIGNING"),
            Some(Capability::Signing)
        );
        assert_eq!(
            Capability::from_string("anchoring"),
            Some(Capability::Anchoring)
        );
        assert_eq!(
            Capability::from_string("session_events"),
            Some(Capability::SessionEvents)
        );
        assert_eq!(
            Capability::from_string("sessionevents"),
            Some(Capability::SessionEvents)
        );
        assert_eq!(
            Capability::from_string("session-events"),
            Some(Capability::SessionEvents)
        );
        assert_eq!(
            Capability::from_string("discovery"),
            Some(Capability::Discovery)
        );
        assert_eq!(
            Capability::from_string("compute"),
            Some(Capability::Compute)
        );
        assert_eq!(
            Capability::from_string("custom:my_cap"),
            Some(Capability::Custom("my_cap".to_string()))
        );
        assert_eq!(
            Capability::from_string("unknown_cap"),
            Some(Capability::Custom("unknown_cap".to_string()))
        );
        assert_eq!(Capability::from_string(""), None);
    }

    #[test]
    fn test_capability_display() {
        assert_eq!(Capability::Signing.to_string(), "signing");
        assert_eq!(Capability::Anchoring.to_string(), "anchoring");
        assert_eq!(Capability::SessionEvents.to_string(), "session_events");
        assert_eq!(Capability::Discovery.to_string(), "discovery");
        assert_eq!(Capability::Compute.to_string(), "compute");
        assert_eq!(
            Capability::Custom("my_cap".to_string()).to_string(),
            "custom:my_cap"
        );
    }

    #[test]
    fn test_capability_roundtrip() {
        let capabilities = [
            Capability::Signing,
            Capability::Anchoring,
            Capability::SessionEvents,
            Capability::Discovery,
            Capability::Compute,
            Capability::Custom("test".to_string()),
        ];

        for cap in &capabilities {
            let s = cap.to_string();
            let parsed = Capability::from_string(&s);
            assert_eq!(parsed, Some(cap.clone()));
        }
    }
}
