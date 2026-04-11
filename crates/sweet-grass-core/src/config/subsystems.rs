// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Subsystem and storage configuration types for `SweetGrass`.

use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::DEFAULT_MAX_PROVENANCE_DEPTH;

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
    #[serde(with = "super::humantime_serde")]
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
    #[serde(with = "super::humantime_serde")]
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
