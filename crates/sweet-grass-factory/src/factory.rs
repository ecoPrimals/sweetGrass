// SPDX-License-Identifier: AGPL-3.0-only
//! Braid Factory implementation.
//!
//! Creates Braids from various input sources.

use sha2::{Digest, Sha256};
use std::fmt::Write;
use sweet_grass_core::{
    activity::{Activity, ActivityMetadata, ActivityType, UsedEntity},
    agent::{AgentAssociation, AgentRole, Did},
    braid::{
        Braid, BraidId, BraidMetadata, BraidSignature, BraidType, CompressionMeta,
        EcoPrimalsAttributes, LoamCommitRef, SummaryType,
    },
    contribution::{ContributionRecord, SessionContribution},
    entity::EntityReference,
    primal_info::SelfKnowledge,
    ContentHash,
};

use crate::error::FactoryError;
use crate::Result;

/// Default source primal name when self-knowledge is unavailable.
pub const DEFAULT_SOURCE_PRIMAL: &str = "unknown";

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

        // Create the generating activity
        let activity = Activity::builder(activity_type)
            .associated_with(AgentAssociation::new(
                self.default_agent.clone(),
                AgentRole::Transformer,
            ))
            .build();

        // Build the activity with used entities
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

        // Add derivation links
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

        // Create a hash of the summarized Braid IDs
        let mut hasher = Sha256::new();
        for id in &summarized {
            hasher.update(id.as_str().as_bytes());
        }
        let result = hasher.finalize();
        let hash = format!("sha256:{}", hex_encode(&result));

        #[allow(clippy::cast_precision_loss)]
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

        // Add derivation links to summarized Braids
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

        // Add `RhizoCrypt` session reference
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
    #[allow(clippy::too_many_arguments)]
    pub fn from_loam_entry(
        &self,
        spine_id: impl Into<String>,
        entry_hash: impl Into<String>,
        index: u64,
        data_hash: ContentHash,
        mime_type: impl Into<String>,
        size: u64,
        metadata: Option<BraidMetadata>,
    ) -> Result<Braid> {
        let spine_id = spine_id.into();
        let entry_hash = entry_hash.into();

        let ecop = EcoPrimalsAttributes {
            source_primal: Some(self.source_primal.clone()),
            niche: self.niche.clone(),
            loam_commit: Some(LoamCommitRef {
                spine_id: spine_id.clone(),
                entry_hash: entry_hash.clone(),
                index,
            }),
            ..Default::default()
        };

        let mut braid = Braid::builder()
            .data_hash(data_hash)
            .mime_type(mime_type)
            .size(size)
            .attributed_to(self.default_agent.clone())
            .metadata(metadata.unwrap_or_default())
            .ecop(ecop)
            .build()
            .map_err(FactoryError::Core)?;

        // Add derivation from anchoring provider entry
        braid
            .was_derived_from
            .push(EntityReference::by_loam_entry(spine_id, entry_hash));

        Ok(braid)
    }

    /// Create a Braid from a single contribution record.
    ///
    /// This is the primary inter-primal interface: another primal sends a
    /// `ContributionRecord`, and sweetGrass creates a provenance Braid.
    ///
    /// # Errors
    ///
    /// Returns an error if braid construction fails.
    pub fn from_contribution(&self, record: &ContributionRecord) -> Result<Braid> {
        let activity_type = if record.session_id.is_some() {
            ActivityType::SessionCommit
        } else {
            ActivityType::Creation
        };

        let activity = Activity::builder(activity_type)
            .associated_with(AgentAssociation::new(
                record.agent.clone(),
                record.role.clone(),
            ))
            .metadata(ActivityMetadata {
                description: record.description.clone(),
                ..ActivityMetadata::default()
            })
            .build();

        let mut metadata = BraidMetadata {
            custom: record.domain.clone(),
            ..BraidMetadata::default()
        };
        if let Some(desc) = &record.description {
            metadata.description = Some(desc.clone());
        }

        let source_primal = record
            .source_primal
            .clone()
            .unwrap_or_else(|| self.source_primal.clone());

        let ecop = EcoPrimalsAttributes {
            source_primal: Some(source_primal),
            niche: self.niche.clone(),
            rhizo_session: record.session_id.clone(),
            ..EcoPrimalsAttributes::default()
        };

        let mut braid = Braid::builder()
            .data_hash(record.content_hash.clone())
            .mime_type(record.mime_type.clone())
            .size(record.size)
            .attributed_to(record.agent.clone())
            .generated_by(activity)
            .metadata(metadata)
            .ecop(ecop)
            .build()
            .map_err(FactoryError::Core)?;

        if record.timestamp != 0 {
            braid.generated_at_time = record.timestamp;
        }

        Ok(braid)
    }

    /// Create braids from a session contribution (batch).
    ///
    /// When rhizoCrypt dehydrates a session, it sends a `SessionContribution`
    /// containing multiple contribution records. This creates one braid per
    /// contribution and returns them all.
    ///
    /// # Errors
    ///
    /// Returns an error if any braid construction fails.
    pub fn from_session(&self, session: &SessionContribution) -> Result<Vec<Braid>> {
        let loam_commit = parse_loam_entry(session.loam_entry.as_deref());

        let mut braids = Vec::with_capacity(session.contributions.len());
        for contrib in &session.contributions {
            let mut record = contrib.clone();
            if record.session_id.is_none() {
                record.session_id = Some(session.session_id.clone());
            }
            if record.source_primal.is_none() {
                record.source_primal = Some(session.source_primal.clone());
            }

            let mut braid = self.from_contribution(&record)?;

            if let Some(ref niche) = session.niche {
                braid.ecop.niche = Some(niche.clone());
            }
            if let Some(ref loam) = loam_commit {
                braid.ecop.loam_commit = Some(loam.clone());
            }

            braids.push(braid);
        }

        Ok(braids)
    }

    /// Sign a Braid with agent credentials.
    ///
    /// Note: This creates a placeholder signature. Real signing requires
    /// integration with signing capability provider.
    pub fn sign(&self, braid: &mut Braid, key_id: &str) {
        // Compute signing hash
        let signing_hash = braid.compute_signing_hash();

        // Create placeholder signature (real implementation discovers signing capability provider)
        let placeholder_sig = signing_hash.as_bytes();
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

        // Create the mint activity
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

/// Parse loam_entry string into LoamCommitRef.
/// Format: "spine_id|entry_hash|index" (pipe-separated).
fn parse_loam_entry(s: Option<&str>) -> Option<LoamCommitRef> {
    let s = s?;
    let parts: Vec<&str> = s.split('|').collect();
    if parts.len() != 3 {
        return None;
    }
    let index = parts[2].parse::<u64>().ok()?;
    Some(LoamCommitRef {
        spine_id: parts[0].to_string(),
        entry_hash: parts[1].to_string(),
        index,
    })
}

/// Compute SHA-256 hash of data.
fn compute_sha256(data: &[u8]) -> ContentHash {
    let result = Sha256::digest(data);
    format!("sha256:{}", hex_encode(&result))
}

/// Hex encode bytes.
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut output, b| {
        let _ = write!(output, "{b:02x}");
        output
    })
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use sweet_grass_core::contribution::{ContributionRecord, SessionContribution};

    fn make_factory() -> BraidFactory {
        BraidFactory::new(Did::new("did:key:z6MkTestFactory"))
    }

    #[test]
    fn test_from_data() {
        let factory = make_factory();
        let data = b"Hello, World!";

        let braid = factory
            .from_data(data, "text/plain", None)
            .expect("should create");

        assert!(braid.data_hash.starts_with("sha256:"));
        assert_eq!(braid.mime_type, "text/plain");
        assert_eq!(braid.size, 13);
        assert_eq!(braid.ecop.source_primal, Some("unknown".to_string()));
    }

    #[test]
    fn test_from_json() {
        #[derive(serde::Serialize)]
        struct Data {
            value: i32,
        }

        let factory = make_factory();
        let braid = factory
            .from_json(&Data { value: 42 }, None)
            .expect("should create");

        assert_eq!(braid.mime_type, "application/json");
    }

    #[test]
    fn test_derived_from() {
        let factory = make_factory();
        let data = b"derived data";
        let sources = vec![EntityReference::by_hash("sha256:source1")];

        let braid = factory
            .derived_from(
                data,
                "text/plain",
                sources,
                ActivityType::Transformation,
                None,
            )
            .expect("should create");

        assert!(!braid.was_derived_from.is_empty());
        assert!(braid.was_generated_by.is_some());
    }

    #[test]
    fn test_meta_braid() {
        let factory = make_factory();
        let braids = vec![BraidId::new(), BraidId::new(), BraidId::new()];

        let summary_type = SummaryType::Custom {
            criteria: "test".to_string(),
        };

        let braid = factory
            .meta_braid(braids, summary_type, None)
            .expect("should create");

        match braid.braid_type {
            BraidType::Collection { member_count, .. } => {
                assert_eq!(member_count, 3);
            },
            _ => panic!("Expected Collection type"),
        }

        assert_eq!(braid.was_derived_from.len(), 3);
    }

    #[test]
    fn test_session_summary() {
        let factory = make_factory();
        let braids = vec![BraidId::new(), BraidId::new()];

        let braid = factory
            .session_summary("session-123", braids, None)
            .expect("should create");

        assert_eq!(braid.ecop.rhizo_session, Some("session-123".to_string()));
    }

    #[test]
    fn test_temporal_summary() {
        let factory = make_factory();
        let braids = vec![BraidId::new()];

        let braid = factory
            .temporal_summary(1000, 2000, braids, None)
            .expect("should create");

        match braid.braid_type {
            BraidType::Collection {
                summary_type: SummaryType::Temporal { start, end },
                ..
            } => {
                assert_eq!(start, 1000);
                assert_eq!(end, 2000);
            },
            _ => panic!("Expected Collection with Temporal summary"),
        }
    }

    #[test]
    fn test_from_loam_entry() {
        let factory = make_factory();

        let braid = factory
            .from_loam_entry(
                "spine-1",
                "sha256:entry123",
                42,
                "sha256:data456".to_string(),
                "application/json",
                1024,
                None,
            )
            .expect("should create");

        assert!(braid.ecop.loam_commit.is_some());
        let commit = braid.ecop.loam_commit.unwrap();
        assert_eq!(commit.spine_id, "spine-1");
        assert_eq!(commit.index, 42);
    }

    #[test]
    fn test_certificate_mint() {
        let factory = make_factory();
        let recipient = Did::new("did:key:z6MkRecipient");

        let braid = factory
            .certificate_mint(
                "cert-001",
                "sha256:certdata".to_string(),
                512,
                recipient.clone(),
                None,
            )
            .expect("should create");

        assert_eq!(braid.ecop.certificate, Some("cert-001".to_string()));
        assert_eq!(braid.was_attributed_to, recipient);
    }

    #[test]
    fn test_sign() {
        let factory = make_factory();
        let mut braid = factory
            .from_data(b"test", "text/plain", None)
            .expect("should create");

        assert!(!braid.is_signed());

        factory.sign(&mut braid, "key-1");

        assert!(braid.is_signed());
        assert!(braid
            .signature
            .verification_method
            .contains("did:key:z6MkTestFactory"));
    }

    #[test]
    fn test_with_niche() {
        let factory = make_factory().with_niche("distributed-science");

        let braid = factory
            .from_data(b"test", "text/plain", None)
            .expect("should create");

        assert_eq!(braid.ecop.niche, Some("distributed-science".to_string()));
    }

    #[test]
    fn test_with_source_primal() {
        let factory = make_factory().with_source_primal("rhizoCrypt");

        let braid = factory
            .from_data(b"test", "text/plain", None)
            .expect("should create");

        assert_eq!(braid.ecop.source_primal, Some("rhizoCrypt".to_string()));
    }

    #[test]
    fn test_from_self_knowledge() {
        use sweet_grass_core::primal_info::SelfKnowledge;

        let self_knowledge = SelfKnowledge {
            name: "test-primal".to_string(),
            ..Default::default()
        };

        let factory =
            BraidFactory::from_self_knowledge(Did::new("did:key:z6MkTest"), &self_knowledge);

        let braid = factory
            .from_data(b"test", "text/plain", None)
            .expect("should create");

        assert_eq!(braid.ecop.source_primal, Some("test-primal".to_string()));
    }

    #[test]
    fn test_from_contribution_creates_valid_braid() {
        let factory = make_factory();
        let record = ContributionRecord {
            agent: Did::new("did:key:z6MkContributor"),
            role: AgentRole::Creator,
            content_hash: "sha256:contrib123".to_string(),
            mime_type: "application/json".to_string(),
            size: 256,
            timestamp: 1_000_000_000,
            description: Some("Test contribution".to_string()),
            source_primal: Some("rhizoCrypt".to_string()),
            session_id: Some("session-xyz".to_string()),
            domain: std::collections::HashMap::new(),
        };

        let braid = factory.from_contribution(&record).expect("should create");

        assert_eq!(braid.data_hash, "sha256:contrib123");
        assert_eq!(braid.was_attributed_to.as_str(), "did:key:z6MkContributor");
        assert_eq!(braid.generated_at_time, 1_000_000_000);
        assert_eq!(braid.ecop.source_primal, Some("rhizoCrypt".to_string()));
        assert_eq!(braid.ecop.rhizo_session, Some("session-xyz".to_string()));
        assert!(braid.was_generated_by.is_some());
        let activity = braid.was_generated_by.as_ref().unwrap();
        assert!(matches!(
            activity.activity_type,
            ActivityType::SessionCommit
        ));
        assert_eq!(activity.was_associated_with.len(), 1);
        assert_eq!(
            activity.was_associated_with[0].agent.as_str(),
            "did:key:z6MkContributor"
        );
    }

    #[test]
    fn test_from_session_creates_multiple_braids() {
        let factory = make_factory();
        let session = SessionContribution {
            session_id: "session-batch".to_string(),
            source_primal: "rhizoCrypt".to_string(),
            niche: Some("chemistry".to_string()),
            contributions: vec![
                ContributionRecord {
                    agent: Did::new("did:key:z6MkAgent1"),
                    role: AgentRole::Creator,
                    content_hash: "sha256:hash1".to_string(),
                    mime_type: "application/json".to_string(),
                    size: 100,
                    timestamp: 0,
                    description: None,
                    source_primal: None,
                    session_id: None,
                    domain: std::collections::HashMap::new(),
                },
                ContributionRecord {
                    agent: Did::new("did:key:z6MkAgent2"),
                    role: AgentRole::Contributor,
                    content_hash: "sha256:hash2".to_string(),
                    mime_type: "text/plain".to_string(),
                    size: 200,
                    timestamp: 0,
                    description: None,
                    source_primal: None,
                    session_id: None,
                    domain: std::collections::HashMap::new(),
                },
            ],
            session_start: None,
            session_end: None,
            loam_entry: None,
            domain: std::collections::HashMap::new(),
        };

        let braids = factory.from_session(&session).expect("should create");

        assert_eq!(braids.len(), 2);
        assert_eq!(braids[0].data_hash, "sha256:hash1");
        assert_eq!(braids[1].data_hash, "sha256:hash2");
        assert_eq!(braids[0].ecop.niche, Some("chemistry".to_string()));
        assert_eq!(braids[1].ecop.niche, Some("chemistry".to_string()));
        assert_eq!(
            braids[0].ecop.rhizo_session,
            Some("session-batch".to_string())
        );
    }

    #[test]
    fn test_from_contribution_domain_metadata_preserved() {
        let factory = make_factory();
        let mut domain = std::collections::HashMap::new();
        domain.insert("chemistry.molecule".to_string(), serde_json::json!("H2O"));
        domain.insert(
            "chemistry.functional".to_string(),
            serde_json::json!("B3LYP"),
        );

        let record = ContributionRecord {
            agent: Did::new("did:key:z6MkChemist"),
            role: AgentRole::Creator,
            content_hash: "sha256:chem123".to_string(),
            mime_type: "application/json".to_string(),
            size: 0,
            timestamp: 0,
            description: None,
            source_primal: None,
            session_id: None,
            domain,
        };

        let braid = factory.from_contribution(&record).expect("should create");

        assert_eq!(
            braid.metadata.custom.get("chemistry.molecule"),
            Some(&serde_json::json!("H2O"))
        );
        assert_eq!(
            braid.metadata.custom.get("chemistry.functional"),
            Some(&serde_json::json!("B3LYP"))
        );
    }
}
