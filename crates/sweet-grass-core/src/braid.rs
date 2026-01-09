//! Braid data structures - the core provenance record.
//!
//! A Braid is a PROV-O compatible provenance record that describes:
//! - What data was created (content hash, MIME type, size)
//! - How it was generated (activity)
//! - Who contributed (agents with roles)
//! - Where it came from (derivation chain)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::activity::Activity;
use crate::agent::Did;
use crate::entity::EntityReference;

/// Content-addressed hash (e.g., "sha256:abc123...")
pub type ContentHash = String;

/// Timestamp in nanoseconds since Unix epoch.
pub type Timestamp = u64;

/// Braid identifier (URN format: "urn:braid:uuid:...")
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BraidId(String);

impl BraidId {
    /// Create a new random Braid ID.
    #[must_use]
    pub fn new() -> Self {
        Self(format!("urn:braid:uuid:{}", Uuid::new_v4()))
    }

    /// Create a Braid ID from a content hash.
    #[must_use]
    pub fn from_hash(hash: &ContentHash) -> Self {
        Self(format!("urn:braid:{hash}"))
    }

    /// Get the inner string representation.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Create a Braid ID from an existing string.
    #[must_use]
    pub const fn from_string(s: String) -> Self {
        Self(s)
    }
}

impl Default for BraidId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for BraidId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Types of Braids.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BraidType {
    /// Standard entity Braid (most common).
    #[default]
    Entity,

    /// Activity Braid.
    Activity,

    /// Agent Braid.
    Agent,

    /// Meta-Braid (summary of other Braids).
    Collection {
        /// Number of Braids summarized.
        member_count: u64,
        /// Type of summary.
        summary_type: SummaryType,
    },

    /// Delegation Braid (agent acting for another).
    Delegation {
        /// The delegate agent.
        delegate: Did,
        /// The principal agent.
        on_behalf_of: Did,
    },

    /// Slice provenance Braid.
    Slice {
        /// Slice operation mode.
        slice_mode: String,
        /// Origin spine ID.
        origin_spine: String,
    },
}

/// Summary types for meta-Braids.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SummaryType {
    /// Session summary.
    Session {
        /// The session ID being summarized.
        session_id: String,
    },
    /// Time period summary.
    Temporal {
        /// Start timestamp.
        start: Timestamp,
        /// End timestamp.
        end: Timestamp,
    },
    /// Activity type summary.
    ActivityGroup {
        /// The activity type being summarized.
        activity_type: String,
    },
    /// Agent contribution summary.
    AgentContributions {
        /// The agent being summarized.
        agent: Did,
    },
    /// Custom grouping.
    Custom {
        /// Criteria description.
        criteria: String,
    },
}

/// JSON-LD context for semantic interpretation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BraidContext {
    /// Base context URL.
    #[serde(rename = "@base")]
    pub base: String,

    /// JSON-LD version.
    #[serde(rename = "@version")]
    pub version: f32,

    /// Vocabulary imports.
    #[serde(flatten)]
    pub imports: HashMap<String, String>,
}

impl Default for BraidContext {
    fn default() -> Self {
        let mut imports = HashMap::new();
        imports.insert("prov".to_string(), "http://www.w3.org/ns/prov#".to_string());
        imports.insert(
            "xsd".to_string(),
            "http://www.w3.org/2001/XMLSchema#".to_string(),
        );
        imports.insert("schema".to_string(), "http://schema.org/".to_string());
        imports.insert(
            "ecop".to_string(),
            "https://ecoprimals.io/vocab#".to_string(),
        );

        Self {
            base: "https://ecoprimals.io/".to_string(),
            version: 1.1,
            imports,
        }
    }
}

/// Braid signature (W3C Data Integrity format).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BraidSignature {
    /// Signature type (e.g., `Ed25519Signature2020`).
    #[serde(rename = "type")]
    pub sig_type: String,

    /// When the signature was created.
    pub created: Timestamp,

    /// Verification method (key reference).
    pub verification_method: String,

    /// Proof purpose.
    pub proof_purpose: String,

    /// Base64-encoded signature value.
    pub proof_value: String,
}

impl BraidSignature {
    /// Create a new Ed25519 signature.
    #[must_use]
    pub fn new_ed25519(did: &Did, key_id: &str, signature_bytes: &[u8]) -> Self {
        use base64::Engine;
        Self {
            sig_type: "Ed25519Signature2020".to_string(),
            created: current_timestamp_nanos(),
            verification_method: format!("{}#{key_id}", did.as_str()),
            proof_purpose: "assertionMethod".to_string(),
            proof_value: base64::engine::general_purpose::STANDARD.encode(signature_bytes),
        }
    }

    /// Create an unsigned placeholder signature.
    #[must_use]
    pub fn unsigned() -> Self {
        Self {
            sig_type: "Unsigned".to_string(),
            created: current_timestamp_nanos(),
            verification_method: String::new(),
            proof_purpose: "pending".to_string(),
            proof_value: String::new(),
        }
    }
}

/// Anchoring provider anchor information.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoamAnchor {
    /// Spine where anchored.
    pub spine_id: String,

    /// Entry hash in the spine.
    pub entry_hash: ContentHash,

    /// Entry index.
    pub index: u64,

    /// When anchored.
    pub anchored_at: Timestamp,

    /// Whether the anchor has been verified.
    pub verified: bool,
}

/// ecoPrimals-specific Braid attributes.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EcoPrimalsAttributes {
    /// Source primal that created this Braid.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_primal: Option<String>,

    /// Niche context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub niche: Option<String>,

    /// `RhizoCrypt` session reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rhizo_session: Option<String>,

    /// `LoamSpine` commit reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loam_commit: Option<LoamCommitRef>,

    /// Certificate reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate: Option<String>,

    /// Compression metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compression: Option<CompressionMeta>,
}

/// `LoamSpine` commit reference.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoamCommitRef {
    /// Spine ID.
    pub spine_id: String,
    /// Entry hash.
    pub entry_hash: ContentHash,
    /// Entry index.
    pub index: u64,
}

/// Compression metadata for summarized Braids.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompressionMeta {
    /// Original vertex count.
    pub vertex_count: u64,
    /// Branches explored.
    pub branch_count: u64,
    /// Compression ratio.
    pub ratio: f64,
    /// Parent Braids summarized.
    pub summarizes: Vec<BraidId>,
}

/// Domain-specific metadata.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BraidMetadata {
    /// Title or name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Tags/keywords.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Custom key-value metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom: HashMap<String, serde_json::Value>,
}

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
    pub mime_type: String,

    /// Size of the data in bytes.
    pub size: u64,

    /// How this data was generated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub was_generated_by: Option<Activity>,

    /// What entities this was derived from.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
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

    /// Cryptographic signature.
    pub signature: BraidSignature,

    /// Anchoring provider anchor (if committed).
    #[serde(skip_serializing_if = "Option::is_none")]
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

    /// Check if this Braid is signed.
    #[must_use]
    pub fn is_signed(&self) -> bool {
        self.signature.sig_type != "Unsigned"
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

        // Create a copy without signature for hashing
        let mut hasher = Sha256::new();
        hasher.update(self.id.as_str().as_bytes());
        hasher.update(self.data_hash.as_bytes());
        hasher.update(self.mime_type.as_bytes());
        hasher.update(self.size.to_le_bytes());
        hasher.update(self.was_attributed_to.as_str().as_bytes());
        hasher.update(self.generated_at_time.to_le_bytes());

        let result = hasher.finalize();
        format!("sha256:{}", hex::encode(result))
    }
}

/// Builder for creating Braids.
#[derive(Default)]
pub struct BraidBuilder {
    data_hash: Option<ContentHash>,
    mime_type: Option<String>,
    size: Option<u64>,
    braid_type: BraidType,
    was_generated_by: Option<Activity>,
    was_derived_from: Vec<EntityReference>,
    was_attributed_to: Option<Did>,
    metadata: BraidMetadata,
    ecop: EcoPrimalsAttributes,
}

impl BraidBuilder {
    /// Set the data hash.
    #[must_use]
    pub fn data_hash(mut self, hash: impl Into<ContentHash>) -> Self {
        self.data_hash = Some(hash.into());
        self
    }

    /// Set the MIME type.
    #[must_use]
    pub fn mime_type(mut self, mime: impl Into<String>) -> Self {
        self.mime_type = Some(mime.into());
        self
    }

    /// Set the size.
    #[must_use]
    pub const fn size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    /// Set the Braid type.
    #[must_use]
    pub fn braid_type(mut self, braid_type: BraidType) -> Self {
        self.braid_type = braid_type;
        self
    }

    /// Set the generating activity.
    #[must_use]
    pub fn generated_by(mut self, activity: Activity) -> Self {
        self.was_generated_by = Some(activity);
        self
    }

    /// Add a derivation source.
    #[must_use]
    pub fn derived_from(mut self, entity: EntityReference) -> Self {
        self.was_derived_from.push(entity);
        self
    }

    /// Set the attribution.
    #[must_use]
    pub fn attributed_to(mut self, did: Did) -> Self {
        self.was_attributed_to = Some(did);
        self
    }

    /// Set metadata.
    #[must_use]
    pub fn metadata(mut self, metadata: BraidMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Set ecoPrimals attributes.
    #[must_use]
    pub fn ecop(mut self, ecop: EcoPrimalsAttributes) -> Self {
        self.ecop = ecop;
        self
    }

    /// Build the Braid.
    ///
    /// # Errors
    ///
    /// Returns an error if required fields are missing.
    pub fn build(self) -> crate::Result<Braid> {
        let data_hash = self
            .data_hash
            .ok_or_else(|| crate::SweetGrassError::Validation("data_hash is required".into()))?;
        let mime_type = self
            .mime_type
            .ok_or_else(|| crate::SweetGrassError::Validation("mime_type is required".into()))?;
        let size = self
            .size
            .ok_or_else(|| crate::SweetGrassError::Validation("size is required".into()))?;
        let was_attributed_to = self.was_attributed_to.ok_or_else(|| {
            crate::SweetGrassError::Validation("was_attributed_to is required".into())
        })?;

        Ok(Braid {
            context: BraidContext::default(),
            id: BraidId::from_hash(&data_hash),
            braid_type: self.braid_type,
            data_hash,
            mime_type,
            size,
            was_generated_by: self.was_generated_by,
            was_derived_from: self.was_derived_from,
            was_attributed_to,
            generated_at_time: current_timestamp_nanos(),
            metadata: self.metadata,
            ecop: self.ecop,
            signature: BraidSignature::unsigned(),
            loam_anchor: None,
        })
    }
}

/// Get current timestamp in nanoseconds since Unix epoch.
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn current_timestamp_nanos() -> Timestamp {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

// Hex encoding helper (avoiding additional dependency)
mod hex {
    use std::fmt::Write;

    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes.as_ref().iter().fold(String::new(), |mut output, b| {
            let _ = write!(output, "{b:02x}");
            output
        })
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::agent::Did;

    #[test]
    fn test_braid_id_generation() {
        let id1 = BraidId::new();
        let id2 = BraidId::new();
        assert_ne!(id1, id2);
        assert!(id1.as_str().starts_with("urn:braid:uuid:"));
    }

    #[test]
    fn test_braid_id_from_hash() {
        let id = BraidId::from_hash(&"sha256:abc123".to_string());
        assert_eq!(id.as_str(), "urn:braid:sha256:abc123");
    }

    #[test]
    fn test_braid_builder() {
        let did = Did::new("did:key:z6MkTest123");
        let braid = Braid::builder()
            .data_hash("sha256:abc123")
            .mime_type("application/json")
            .size(1024)
            .attributed_to(did.clone())
            .build()
            .expect("should build");

        assert_eq!(braid.data_hash, "sha256:abc123");
        assert_eq!(braid.mime_type, "application/json");
        assert_eq!(braid.size, 1024);
        assert_eq!(braid.was_attributed_to, did);
        assert!(!braid.is_signed());
        assert!(!braid.is_anchored());
    }

    #[test]
    fn test_braid_builder_missing_required() {
        let result = Braid::builder().build();
        assert!(result.is_err());
    }

    #[test]
    fn test_braid_context_default() {
        let ctx = BraidContext::default();
        assert!((ctx.version - 1.1).abs() < f32::EPSILON);
        assert!(ctx.imports.contains_key("prov"));
        assert!(ctx.imports.contains_key("ecop"));
    }

    #[test]
    fn test_braid_serialization() {
        let did = Did::new("did:key:z6MkTest123");
        let braid = Braid::builder()
            .data_hash("sha256:abc123")
            .mime_type("text/plain")
            .size(100)
            .attributed_to(did)
            .build()
            .expect("should build");

        let json = serde_json::to_string_pretty(&braid).expect("should serialize");
        assert!(json.contains("@context"));
        assert!(json.contains("@id"));
        assert!(json.contains("sha256:abc123"));

        let parsed: Braid = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(parsed.data_hash, braid.data_hash);
    }
}

/// Property-based tests using proptest
#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod proptests {
    use super::*;
    use crate::agent::Did;
    use proptest::prelude::*;

    /// Generate arbitrary valid SHA256 hashes
    fn arb_sha256_hash() -> impl Strategy<Value = String> {
        "[a-f0-9]{64}".prop_map(|s| format!("sha256:{s}"))
    }

    /// Generate arbitrary valid DID strings
    fn arb_did() -> impl Strategy<Value = Did> {
        "[a-zA-Z0-9]{10,50}".prop_map(|s| Did::new(format!("did:key:z6Mk{s}")))
    }

    /// Generate arbitrary MIME types
    fn arb_mime_type() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("text/plain".to_string()),
            Just("application/json".to_string()),
            Just("application/octet-stream".to_string()),
            Just("text/csv".to_string()),
            Just("image/png".to_string()),
        ]
    }

    proptest! {
        /// BraidId uniqueness property: new IDs should always be unique
        #[test]
        fn prop_braid_id_unique(_seed: u64) {
            let id1 = BraidId::new();
            let id2 = BraidId::new();
            prop_assert_ne!(id1, id2);
        }

        /// BraidId from hash is deterministic
        #[test]
        fn prop_braid_id_from_hash_deterministic(hash in arb_sha256_hash()) {
            let id1 = BraidId::from_hash(&hash);
            let id2 = BraidId::from_hash(&hash);
            prop_assert_eq!(id1, id2);
        }

        /// Braid builder with valid inputs always succeeds
        #[test]
        fn prop_braid_builder_valid_inputs(
            hash in arb_sha256_hash(),
            mime in arb_mime_type(),
            size in 0u64..10_000_000,
            did in arb_did(),
        ) {
            let result = Braid::builder()
                .data_hash(&hash)
                .mime_type(&mime)
                .size(size)
                .attributed_to(did)
                .build();
            prop_assert!(result.is_ok());
        }

        /// Braid serialization roundtrip preserves data
        #[test]
        fn prop_braid_serialization_roundtrip(
            hash in arb_sha256_hash(),
            mime in arb_mime_type(),
            size in 0u64..10_000_000,
            did in arb_did(),
        ) {
            let braid = Braid::builder()
                .data_hash(&hash)
                .mime_type(&mime)
                .size(size)
                .attributed_to(did)
                .build()
                .expect("should build");

            let json = serde_json::to_string(&braid).expect("should serialize");
            let parsed: Braid = serde_json::from_str(&json).expect("should deserialize");

            prop_assert_eq!(braid.data_hash, parsed.data_hash);
            prop_assert_eq!(braid.mime_type, parsed.mime_type);
            prop_assert_eq!(braid.size, parsed.size);
            prop_assert_eq!(braid.was_attributed_to, parsed.was_attributed_to);
        }

        /// BraidId string format is always valid
        #[test]
        fn prop_braid_id_format(hash in arb_sha256_hash()) {
            let id = BraidId::from_hash(&hash);
            let id_str = id.as_str();
            prop_assert!(id_str.starts_with("urn:braid:"));
            prop_assert!(id_str.contains("sha256:"));
        }

        /// Content hash format is preserved in braid
        #[test]
        fn prop_content_hash_preserved(hash in arb_sha256_hash(), did in arb_did()) {
            let braid = Braid::builder()
                .data_hash(&hash)
                .mime_type("text/plain")
                .size(100)
                .attributed_to(did)
                .build()
                .expect("should build");

            prop_assert_eq!(braid.data_hash, hash);
        }
    }
}
