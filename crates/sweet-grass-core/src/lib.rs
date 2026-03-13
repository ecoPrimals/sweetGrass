// SPDX-License-Identifier: AGPL-3.0-only
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
#![forbid(unsafe_code)]

pub mod activity;
pub mod agent;
pub mod braid;
pub mod config;
pub mod contribution;
pub mod dehydration;
pub mod entity;
pub mod error;
pub mod hash;
pub mod primal;
pub mod primal_info;
pub mod privacy;

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
pub use primal::{HealthStatus, PrimalState, SweetGrass};
pub use primal_info::SelfKnowledge;
pub use privacy::{
    DataSubjectRequest, ErasureReason, PrivacyLevel, PrivacyMetadata, ProcessingType,
    RetentionPolicy,
};

/// Result type for `SweetGrass` operations.
pub type Result<T> = std::result::Result<T, SweetGrassError>;
