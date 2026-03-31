// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Config module tests.

#![cfg(test)]
#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "test module: expect/unwrap are standard in tests"
)]

use std::collections::HashMap;

use super::*;
use crate::identity;

fn mock_reader(vars: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> + use<> {
    let map: HashMap<String, String> = vars
        .iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect();
    move |key: &str| map.get(key).cloned()
}

fn empty_reader() -> impl Fn(&str) -> Option<String> {
    |_: &str| None
}

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
    assert!(
        config
            .network
            .required_capabilities
            .contains(&Capability::Signing)
    );
    assert!(
        config
            .network
            .offered_capabilities
            .contains(&Capability::Custom("test_capability".to_string()))
    );
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
fn test_load_env_overrides_file() {
    let temp_dir = std::env::temp_dir();
    let config_path = temp_dir.join("sweetgrass_test_env_override.toml");
    let toml_content = r#"
name = "FromFile"
[network]
tarpc_listen = "127.0.0.1:9999"
"#;
    std::fs::write(&config_path, toml_content).expect("write temp config");

    let path_str = config_path.to_str().expect("valid path").to_string();
    let reader = mock_reader(&[
        ("SWEETGRASS_CONFIG", &path_str),
        ("SWEETGRASS_NAME", "FromEnv"),
    ]);

    let result = SweetGrassConfig::load_with_reader(reader);
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
fn test_load_missing_file_returns_defaults() {
    let config = SweetGrassConfig::load_with_reader(empty_reader())
        .expect("load should succeed with no file");
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

#[test]
fn test_env_overrides_all_fields() {
    let reader = mock_reader(&[
        ("SWEETGRASS_NAME", "EnvPrimal"),
        ("SWEETGRASS_TARPC_LISTEN", "0.0.0.0:5000"),
        ("SWEETGRASS_REST_LISTEN", "0.0.0.0:8080"),
        ("SWEETGRASS_DISCOVERY_BOOTSTRAP", "registry:9000"),
    ]);

    let config = SweetGrassConfig::load_with_reader(reader).expect("load");
    assert_eq!(config.name, "EnvPrimal");
    assert_eq!(config.network.tarpc_listen.as_deref(), Some("0.0.0.0:5000"));
    assert_eq!(config.network.rest_listen.as_deref(), Some("0.0.0.0:8080"));
    assert_eq!(
        config.network.discovery_bootstrap.as_deref(),
        Some("registry:9000")
    );
}

#[test]
fn test_builder_rest_listen_and_storage() {
    let config = SweetGrassConfig::builder()
        .rest_listen("127.0.0.1:3000")
        .storage_backend(StorageBackend::Postgres)
        .build();

    assert_eq!(
        config.network.rest_listen.as_deref(),
        Some("127.0.0.1:3000")
    );
    assert_eq!(config.storage.backend, StorageBackend::Postgres);
}

#[test]
fn test_listener_config_serialization() {
    let config = ListenerConfig::default();
    let json = serde_json::to_string(&config).expect("serialize");
    let parsed: ListenerConfig = serde_json::from_str(&json).expect("deserialize");
    assert!(parsed.session_events);
    assert!(parsed.anchoring);
    assert!(parsed.compute);
    assert_eq!(parsed.buffer_size, 1000);
    assert_eq!(parsed.batch_size, 100);
}

#[test]
fn test_storage_backend_variants() {
    for (variant, expected) in [
        (StorageBackend::Memory, "\"memory\""),
        (StorageBackend::File, "\"file\""),
        (StorageBackend::Postgres, "\"postgres\""),
        (StorageBackend::Oxigraph, "\"oxigraph\""),
    ] {
        let json = serde_json::to_string(&variant).expect("serialize");
        assert_eq!(json, expected);
    }

    let custom = StorageBackend::Custom("s3".to_string());
    let json = serde_json::to_string(&custom).expect("serialize");
    let parsed: StorageBackend = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed, custom);
}

#[test]
fn test_config_error_display() {
    let err = ConfigError::Invalid("bad value".to_string());
    assert!(err.to_string().contains("bad value"));

    let err = ConfigError::Missing("required_field".to_string());
    assert!(err.to_string().contains("required_field"));

    let err = ConfigError::Environment("VAR_NOT_SET".to_string());
    assert!(err.to_string().contains("VAR_NOT_SET"));

    let err = ConfigError::Parse("line 3: unexpected".to_string());
    assert!(err.to_string().contains("unexpected"));
}

#[test]
fn test_network_config_serialization() {
    let config = NetworkConfig::default();
    let json = serde_json::to_string(&config).expect("serialize");
    let parsed: NetworkConfig = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed.discovery_retries, 3);
    assert_eq!(
        parsed.connection_timeout,
        std::time::Duration::from_secs(10)
    );
}

#[test]
fn test_capability_custom_prefix_variants() {
    assert_eq!(
        Capability::from_string("Custom:my_cap"),
        Some(Capability::Custom("my_cap".to_string()))
    );
    assert_eq!(
        Capability::from_string("CUSTOM:my_cap"),
        Some(Capability::Custom("my_cap".to_string()))
    );
}
