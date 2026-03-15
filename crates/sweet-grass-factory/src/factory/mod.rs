// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Braid Factory implementation.
//!
//! Creates Braids from various input sources.

use sha2::{Digest, Sha256};
use sweet_grass_core::{
    activity::{Activity, ActivityType, UsedEntity},
    agent::{AgentAssociation, AgentRole, Did},
    braid::{
        Braid, BraidId, BraidMetadata, BraidSignature, BraidType, CompressionMeta,
        EcoPrimalsAttributes, LoamCommitRef, SummaryType,
    },
    entity::EntityReference,
    hash::hex_encode,
    primal_info::SelfKnowledge,
    ContentHash,
};

use crate::error::FactoryError;
use crate::Result;

pub mod contribution;

/// Default source primal name when self-knowledge is unavailable.
///
/// Prefer [`BraidFactory::from_self_knowledge()`] in production so the source
/// primal comes from the primal's `SelfKnowledge` at runtime.
pub const DEFAULT_SOURCE_PRIMAL: &str = "unknown";

/// Parameters for creating a Braid from an anchoring provider (Loam) entry.
#[derive(Clone, Debug)]
pub struct LoamEntryParams {
    /// Spine identifier.
    pub spine_id: String,
    /// Entry content hash within the spine.
    pub entry_hash: ContentHash,
    /// Entry index.
    pub index: u64,
    /// Content hash of the data.
    pub data_hash: ContentHash,
    /// MIME type of the data.
    pub mime_type: String,
    /// Size in bytes.
    pub size: u64,
    /// Optional metadata.
    pub metadata: Option<BraidMetadata>,
}

/// Braid Factory - creates Braids from various sources.
pub struct BraidFactory {
    /// Default agent for attributing new Braids.
    default_agent: Did,

    /// Source primal name for ecoPrimals attributes.
    source_primal: String,

    /// Niche context.
    niche: Option<String>,
}

impl BraidFactory {
    /// Create from self-knowledge (preferred constructor).
    ///
    /// Uses the primal's self-discovered name instead of hardcoding.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let self_knowledge = SelfKnowledge::from_env()?;
    /// let factory = BraidFactory::from_self_knowledge(
    ///     Did::new("did:key:agent"),
    ///     &self_knowledge
    /// );
    /// ```
    #[must_use]
    pub fn from_self_knowledge(default_agent: Did, self_knowledge: &SelfKnowledge) -> Self {
        Self {
            default_agent,
            source_primal: self_knowledge.name.clone(),
            niche: None,
        }
    }

    /// Create with explicit source (for testing or when self-knowledge unavailable).
    ///
    /// Prefer `from_self_knowledge()` in production code.
    #[must_use]
    pub fn new(default_agent: Did) -> Self {
        Self {
            default_agent,
            source_primal: DEFAULT_SOURCE_PRIMAL.to_string(),
            niche: None,
        }
    }

    /// Set the source primal name.
    #[must_use]
    pub fn with_source_primal(mut self, primal: impl Into<String>) -> Self {
        self.source_primal = primal.into();
        self
    }

    /// Set the niche context.
    #[must_use]
    pub fn with_niche(mut self, niche: impl Into<String>) -> Self {
        self.niche = Some(niche.into());
        self
    }

    /// Create a Braid from raw data.
    ///
    /// This computes the content hash and creates a fully-formed Braid.
    ///
    /// # Errors
    ///
    /// Returns an error if the data is too large or Braid construction fails.
    pub fn from_data(
        &self,
        data: &[u8],
        mime_type: impl Into<String>,
        metadata: Option<BraidMetadata>,
    ) -> Result<Braid> {
        let hash = compute_sha256(data);
        let size = u64::try_from(data.len())
            .map_err(|_| FactoryError::InvalidInput("Data too large".to_string()))?;

        self.from_hash(hash, mime_type, size, metadata)
    }

    /// Create a Braid from a pre-computed hash.
    ///
    /// # Errors
    ///
    /// Returns an error if Braid construction fails.
    pub fn from_hash(
        &self,
        hash: ContentHash,
        mime_type: impl Into<String>,
        size: u64,
        metadata: Option<BraidMetadata>,
    ) -> Result<Braid> {
        let ecop = EcoPrimalsAttributes {
            source_primal: Some(self.source_primal.clone()),
            niche: self.niche.clone(),
            ..Default::default()
        };

        Braid::builder()
            .data_hash(hash)
            .mime_type(mime_type)
            .size(size)
            .attributed_to(self.default_agent.clone())
            .metadata(metadata.unwrap_or_default())
            .ecop(ecop)
            .build()
            .map_err(FactoryError::Core)
    }

    /// Create a Braid from JSON-serializable data.
    ///
    /// # Errors
    ///
    /// Returns an error if JSON serialization or Braid construction fails.
    pub fn from_json<T: serde::Serialize>(
        &self,
        value: &T,
        metadata: Option<BraidMetadata>,
    ) -> Result<Braid> {
        let json = serde_json::to_vec(value)?;
        self.from_data(&json, "application/json", metadata)
    }

    /// Create a Braid with derivation links.
    ///
    /// # Errors
    ///
    /// Returns an error if the data is too large or Braid construction fails.
    pub fn derived_from(
        &self,
        data: &[u8],
        mime_type: impl Into<String>,
        sources: Vec<EntityReference>,
        activity_type: ActivityType,
        metadata: Option<BraidMetadata>,
    ) -> Result<Braid> {
        let hash = compute_sha256(data);
        let size = u64::try_from(data.len())
            .map_err(|_| FactoryError::InvalidInput("Data too large".to_string()))?;

        let activity = Activity::builder(activity_type)
            .associated_with(AgentAssociation::new(
                self.default_agent.clone(),
                AgentRole::Transformer,
            ))
            .build();

        let mut activity_with_uses = activity;
        for source in &sources {
            activity_with_uses
                .used
                .push(UsedEntity::new(source.clone()));
        }

        let ecop = EcoPrimalsAttributes {
            source_primal: Some(self.source_primal.clone()),
            niche: self.niche.as_ref().map(ToString::to_string),
            ..Default::default()
        };

        let mut braid = Braid::builder()
            .data_hash(hash)
            .mime_type(mime_type)
            .size(size)
            .attributed_to(self.default_agent.clone())
            .generated_by(activity_with_uses)
            .metadata(metadata.unwrap_or_default())
            .ecop(ecop)
            .build()
            .map_err(FactoryError::Core)?;

        braid.was_derived_from = sources;

        Ok(braid)
    }

    /// Create a meta-Braid that summarizes other Braids.
    ///
    /// # Errors
    ///
    /// Returns an error if too many Braids or construction fails.
    pub fn meta_braid(
        &self,
        summarized: Vec<BraidId>,
        summary_type: SummaryType,
        metadata: Option<BraidMetadata>,
    ) -> Result<Braid> {
        let member_count = u64::try_from(summarized.len())
            .map_err(|_| FactoryError::InvalidInput("Too many Braids".to_string()))?;

        let mut hasher = Sha256::new();
        for id in &summarized {
            hasher.update(id.as_str().as_bytes());
        }
        let result = hasher.finalize();
        let hash = format!("sha256:{}", hex_encode(result));

        #[expect(
            clippy::cast_precision_loss,
            reason = "member_count is small; u64->f64 precision loss is acceptable for compression ratio"
        )]
        let ecop = EcoPrimalsAttributes {
            source_primal: Some(self.source_primal.clone()),
            niche: self.niche.clone(),
            compression: Some(CompressionMeta {
                vertex_count: member_count,
                branch_count: 1,
                ratio: 1.0 / (member_count as f64),
                summarizes: summarized.clone(),
            }),
            ..Default::default()
        };

        let braid_type = BraidType::Collection {
            member_count,
            summary_type,
        };

        let mut braid = Braid::builder()
            .data_hash(hash)
            .mime_type("application/vnd.ecoprimals.meta-braid")
            .size(0)
            .braid_type(braid_type)
            .attributed_to(self.default_agent.clone())
            .metadata(metadata.unwrap_or_default())
            .ecop(ecop)
            .build()
            .map_err(FactoryError::Core)?;

        braid.was_derived_from = summarized.into_iter().map(EntityReference::by_id).collect();

        Ok(braid)
    }

    /// Create a session summary Braid.
    ///
    /// # Errors
    ///
    /// Returns an error if Braid construction fails.
    pub fn session_summary(
        &self,
        session_id: impl Into<String>,
        braids: Vec<BraidId>,
        metadata: Option<BraidMetadata>,
    ) -> Result<Braid> {
        let session_id = session_id.into();
        let summary_type = SummaryType::Session {
            session_id: session_id.clone(),
        };

        let mut braid = self.meta_braid(braids, summary_type, metadata)?;

        braid.ecop.rhizo_session = Some(session_id);

        Ok(braid)
    }

    /// Create a temporal summary Braid.
    ///
    /// # Errors
    ///
    /// Returns an error if Braid construction fails.
    pub fn temporal_summary(
        &self,
        start: u64,
        end: u64,
        braids: Vec<BraidId>,
        metadata: Option<BraidMetadata>,
    ) -> Result<Braid> {
        let summary_type = SummaryType::Temporal { start, end };
        self.meta_braid(braids, summary_type, metadata)
    }

    /// Create a Braid from an anchoring provider entry reference.
    ///
    /// # Errors
    ///
    /// Returns an error if Braid construction fails.
    pub fn from_loam_entry(&self, entry: &LoamEntryParams) -> Result<Braid> {
        let ecop = EcoPrimalsAttributes {
            source_primal: Some(self.source_primal.clone()),
            niche: self.niche.clone(),
            loam_commit: Some(LoamCommitRef {
                spine_id: entry.spine_id.clone(),
                entry_hash: entry.entry_hash.clone(),
                index: entry.index,
            }),
            ..Default::default()
        };

        let mut braid = Braid::builder()
            .data_hash(entry.data_hash.clone())
            .mime_type(&entry.mime_type)
            .size(entry.size)
            .attributed_to(self.default_agent.clone())
            .metadata(entry.metadata.clone().unwrap_or_default())
            .ecop(ecop)
            .build()
            .map_err(FactoryError::Core)?;

        braid.was_derived_from.push(EntityReference::by_loam_entry(
            &entry.spine_id,
            entry.entry_hash.clone(),
        ));

        Ok(braid)
    }

    /// Attach a local placeholder signature to a Braid.
    ///
    /// **This is NOT a cryptographic signature.** It derives a deterministic
    /// placeholder from the Braid's signing hash so that the signature field
    /// is populated for local operations, testing, and pre-signing workflows.
    ///
    /// For real Ed25519 signing, route through a primal offering
    /// [`Capability::Signing`] at runtime via capability-based discovery.
    ///
    /// [`Capability::Signing`]: sweet_grass_core::config::Capability::Signing
    pub fn sign_placeholder(&self, braid: &mut Braid, key_id: &str) {
        let signing_hash = braid.compute_signing_hash();
        let placeholder_sig = signing_hash.as_str().as_bytes();
        braid.signature =
            BraidSignature::new_ed25519(&braid.was_attributed_to, key_id, placeholder_sig);
    }

    /// Create a Braid for a certificate mint event.
    ///
    /// # Errors
    ///
    /// Returns an error if Braid construction fails.
    pub fn certificate_mint(
        &self,
        certificate_id: impl Into<String>,
        data_hash: ContentHash,
        size: u64,
        recipient: Did,
        metadata: Option<BraidMetadata>,
    ) -> Result<Braid> {
        let certificate_id = certificate_id.into();

        let activity = Activity::builder(ActivityType::CertificateMint)
            .associated_with(AgentAssociation::new(
                self.default_agent.clone(),
                AgentRole::Creator,
            ))
            .associated_with(AgentAssociation::new(recipient.clone(), AgentRole::Owner))
            .build();

        let ecop = EcoPrimalsAttributes {
            source_primal: Some(self.source_primal.clone()),
            niche: self.niche.clone(),
            certificate: Some(certificate_id),
            ..Default::default()
        };

        Braid::builder()
            .data_hash(data_hash)
            .mime_type("application/vnd.ecoprimals.certificate")
            .size(size)
            .attributed_to(recipient)
            .generated_by(activity)
            .metadata(metadata.unwrap_or_default())
            .ecop(ecop)
            .build()
            .map_err(FactoryError::Core)
    }
}

mod tests;

fn compute_sha256(data: &[u8]) -> ContentHash {
    let result = Sha256::digest(data);
    ContentHash::new(format!("sha256:{}", hex_encode(result)))
}
