// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Braid data structures - the core provenance record.
//!
//! A Braid is a PROV-O compatible provenance record that describes:
//! - What data was created (content hash, MIME type, size)
//! - How it was generated (activity)
//! - Who contributed (agents with roles)
//! - Where it came from (derivation chain)

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::activity::Activity;
use crate::agent::Did;
use crate::entity::EntityReference;

mod braid_type;
pub mod builder;
pub mod context;
mod tests;
pub mod types;

pub use builder::BraidBuilder;
pub use types::{
    BraidContext, BraidId, BraidMetadata, BraidType, CompressionMeta, ContentHash,
    DEFAULT_ECOP_BASE_URI, DEFAULT_ECOP_VOCAB_URI, EcoPrimalsAttributes, JsonLdVersion,
    LedgerCommitRef, LoamAnchor, LoamCommitRef, PROV_VOCAB_URI, RDFS_VOCAB_URI, SCHEMA_VOCAB_URI,
    SummaryType, Timestamp, XSD_VOCAB_URI, current_timestamp_nanos, ecop_base_uri,
    ecop_base_uri_with_reader, ecop_vocab_uri, ecop_vocab_uri_with_reader,
};

#[expect(
    deprecated,
    reason = "re-export for backward compat; remove with BraidSignature in v0.7.29"
)]
pub use types::BraidSignature;

/// A `SweetGrass` Braid (provenance record).
///
/// Braids are the fundamental unit of provenance in `SweetGrass`,
/// following the W3C PROV-O model with ecoPrimals extensions.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Braid {
    /// JSON-LD context for semantic interpretation.
    #[serde(rename = "@context")]
    pub context: BraidContext,

    /// Unique identifier.
    #[serde(rename = "@id")]
    pub id: BraidId,

    /// Braid type.
    #[serde(rename = "@type")]
    pub braid_type: BraidType,

    /// Hash of the data this Braid describes.
    pub data_hash: ContentHash,

    /// MIME type of the data.
    ///
    /// Uses `Arc<str>` for O(1) clone — MIME types are read-heavy and frequently
    /// cloned during indexing, filtering, and serialization.
    pub mime_type: Arc<str>,

    /// Size of the data in bytes.
    pub size: u64,

    /// How this data was generated.
    pub was_generated_by: Option<Activity>,

    /// What entities this was derived from.
    #[serde(default)]
    pub was_derived_from: Vec<EntityReference>,

    /// Who created/owns this Braid (DID).
    pub was_attributed_to: Did,

    /// When this Braid was created.
    pub generated_at_time: Timestamp,

    /// Domain-specific metadata.
    #[serde(default)]
    pub metadata: BraidMetadata,

    /// ecoPrimals-specific attributes.
    #[serde(default)]
    pub ecop: EcoPrimalsAttributes,

    /// Primary witness (WireWitnessRef-aligned provenance event).
    ///
    /// Supersedes the former `BraidSignature` (LD-Proof pattern).
    /// `kind: "signature"` with `algorithm` / `evidence` replaces the old
    /// `sig_type` / `proof_value` / `proof_purpose` fields.
    #[serde(alias = "signature")]
    pub witness: crate::dehydration::Witness,

    /// Anchoring provider anchor (if committed).
    pub loam_anchor: Option<LoamAnchor>,
}

impl Braid {
    /// Create a new Braid builder.
    #[must_use]
    pub fn builder() -> BraidBuilder {
        BraidBuilder::default()
    }

    /// Check if this Braid is anchored to permanent storage.
    #[must_use]
    pub const fn is_anchored(&self) -> bool {
        self.loam_anchor.is_some()
    }

    /// Check if this Braid carries a cryptographic signature witness.
    #[must_use]
    pub fn is_signed(&self) -> bool {
        self.witness.is_signed()
    }

    /// Get the content hash for verification.
    #[must_use]
    pub const fn content_hash(&self) -> &ContentHash {
        &self.data_hash
    }

    /// Compute the hash of this Braid's content for signing.
    #[must_use]
    pub fn compute_signing_hash(&self) -> ContentHash {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(self.id.as_str().as_bytes());
        hasher.update(self.data_hash.as_str().as_bytes());
        hasher.update(self.mime_type.as_bytes());
        hasher.update(self.size.to_le_bytes());
        hasher.update(self.was_attributed_to.as_str().as_bytes());
        hasher.update(self.generated_at_time.to_le_bytes());

        let result = hasher.finalize();
        ContentHash::new(format!("sha256:{}", crate::hash::hex_encode(result)))
    }
}
