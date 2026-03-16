// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! # `SweetGrass`
//!
//! Attribution Layer - Semantic Provenance & PROV-O
//!
//! `SweetGrass` is the semantic provenance layer of the ecoPrimals ecosystem.
//! It creates **Braids**—cryptographically signed, machine-readable provenance
//! documents following W3C PROV-O standards.
//!
//! ## Overview
//!
//! `SweetGrass` answers the fundamental question: **"What is the story of this data?"**
//!
//! - **Braids** — Provenance records tracking what created data, who contributed
//! - **Attribution** — Fair credit assignment for economic distribution
//! - **PROV-O Compatible** — W3C standard interoperability
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use sweet_grass_core::{SweetGrass, SweetGrassConfig};
//!
//! let config = SweetGrassConfig::default();
//! let mut primal = SweetGrass::new(config);
//! primal.start().await?;
//! ```

#![warn(missing_docs)]
#![cfg_attr(not(test), forbid(unsafe_code))]
#![cfg_attr(test, deny(unsafe_code))]

pub mod activity;
pub mod agent;
pub mod braid;
pub mod config;
pub mod contribution;
pub mod dehydration;
pub mod entity;
pub mod error;
pub mod hash;
pub mod niche;
pub mod primal;
pub mod primal_info;
pub mod primal_names;
pub mod privacy;
pub mod scyborg;

/// Primal identity constants.
///
/// Centralized here so every crate references the same values
/// instead of scattering string literals. In production the primal name
/// comes from environment (`PRIMAL_NAME`) via [`SelfKnowledge`], but
/// these constants serve as the canonical defaults.
pub mod identity {
    /// Canonical lowercase primal name (used for sockets, env defaults, IPC).
    pub const PRIMAL_NAME: &str = "sweetgrass";

    /// Human-readable display name (used in config defaults, logs, health).
    pub const PRIMAL_DISPLAY_NAME: &str = "SweetGrass";

    /// Fallback DID when no agent is available (e.g. dehydration without operations).
    pub const UNKNOWN_AGENT_DID: &str = "did:key:unknown";

    /// MIME type for merkle root Braids produced during dehydration.
    pub const MIME_MERKLE_ROOT: &str = "application/x-merkle-root";

    /// MIME type for generic binary data (dehydration operations).
    pub const MIME_OCTET_STREAM: &str = "application/octet-stream";

    /// Default storage backend when none is configured.
    pub const DEFAULT_STORAGE_BACKEND: &str = "memory";

    /// Default redb storage file path.
    pub const DEFAULT_REDB_PATH: &str = "./data/sweetgrass.redb";

    /// Default sled storage directory path.
    pub const DEFAULT_SLED_PATH: &str = "./data/sweetgrass";
}

// Re-exports for convenience
pub use activity::{Activity, ActivityId, ActivityType, EntityRole, UsedEntity};
pub use agent::{Agent, AgentAssociation, AgentRole, AgentType, Did};
pub use braid::{Braid, BraidId, BraidSignature, BraidType, ContentHash, Timestamp};
pub use config::{
    Capability, ConfigError, NetworkConfig, SweetGrassConfig, SweetGrassConfigBuilder,
};
pub use dehydration::{Attestation, DehydrationSummary, SessionOperation};
pub use entity::{Encoding, EntityReference, InlineEntity};
pub use error::SweetGrassError;
pub use hash::HexDecodeError;
pub use primal::{HealthStatus, PrimalState, SweetGrass};
pub use primal_info::{BootstrapEnvError, SelfKnowledge};
pub use privacy::{
    DataSubjectRequest, ErasureReason, PrivacyLevel, PrivacyMetadata, ProcessingType,
    RetentionPolicy,
};
pub use scyborg::{AttributionNotice, ContentCategory, LicenseExpression, LicenseId};

/// Result type for `SweetGrass` operations.
pub type Result<T> = std::result::Result<T, SweetGrassError>;

/// Test fixture constants for capability-based test isolation.
///
/// Available when building tests or with the `test` feature.
/// These replace hardcoded primal names with capability-agnostic identifiers.
#[cfg(any(test, feature = "test"))]
pub mod test_fixtures {
    /// Capability-based test primal name for source attribution.
    ///
    /// Represents any primal offering the session capture capability in tests,
    /// rather than hardcoding a specific primal name.
    pub const TEST_SOURCE_PRIMAL: &str = "test-session-source";
}
