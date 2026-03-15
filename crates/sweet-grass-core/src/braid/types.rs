// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Braid type definitions: `ContentHash`, `BraidId`, `BraidContext`, `BraidType`, etc.

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::agent::Did;
use crate::hash::hex_decode;

/// Content-addressed hash (e.g., "sha256:abc123...").
///
/// Uses `Arc<str>` internally so `.clone()` is O(1) (atomic refcount increment),
/// matching the zero-copy strategy used by `BraidId` and `Did`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct ContentHash(Arc<str>);

impl ContentHash {
    /// Create from any string-like value.
    #[must_use]
    pub fn new(s: impl Into<String>) -> Self {
        let s = s.into();
        Self(Arc::from(s.into_boxed_str()))
    }

    /// View as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Extract the raw hash bytes from a prefixed hash (e.g., `"sha256:abcdef..."`).
    ///
    /// Returns `None` if the hash is not in `{algorithm}:{hex}` format or
    /// the hex portion doesn't decode to exactly 32 bytes.
    /// This is used for `LoamSpine` anchoring which expects `[u8; 32]`.
    #[must_use]
    pub fn to_bytes32(&self) -> Option<[u8; 32]> {
        let hex_str = self.0.split_once(':').map(|(_, h)| h)?;
        let bytes = hex_decode(hex_str)?;
        <[u8; 32]>::try_from(bytes.as_slice()).ok()
    }
}

impl std::fmt::Display for ContentHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for ContentHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self(Arc::from(s.into_boxed_str())))
    }
}

impl From<&str> for ContentHash {
    fn from(s: &str) -> Self {
        Self(Arc::from(s))
    }
}

impl From<String> for ContentHash {
    fn from(s: String) -> Self {
        Self(Arc::from(s.into_boxed_str()))
    }
}

impl From<&Self> for ContentHash {
    fn from(s: &Self) -> Self {
        s.clone()
    }
}

impl From<&String> for ContentHash {
    fn from(s: &String) -> Self {
        Self(Arc::from(s.as_str()))
    }
}

impl PartialEq<str> for ContentHash {
    fn eq(&self, other: &str) -> bool {
        self.0.as_ref() == other
    }
}

impl AsRef<str> for ContentHash {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::borrow::Borrow<str> for ContentHash {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl Default for ContentHash {
    fn default() -> Self {
        Self(Arc::from(""))
    }
}

/// Timestamp in nanoseconds since Unix epoch.
pub type Timestamp = u64;

/// Braid identifier (URN format: "urn:braid:uuid:...")
///
/// Uses `Arc<str>` internally so `.clone()` is O(1) (atomic refcount increment).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct BraidId(Arc<str>);

impl BraidId {
    /// Create a new random Braid ID.
    #[must_use]
    pub fn new() -> Self {
        Self(format!("urn:braid:uuid:{}", Uuid::new_v4()).into())
    }

    /// Create a Braid ID from a content hash.
    #[must_use]
    pub fn from_hash(hash: &ContentHash) -> Self {
        Self(format!("urn:braid:{hash}").into())
    }

    /// Get the inner string representation.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Create a Braid ID from an existing string.
    #[must_use]
    pub fn from_string(s: impl Into<String>) -> Self {
        let s = s.into();
        Self(Arc::from(s.into_boxed_str()))
    }

    /// Extract the UUID from a `urn:braid:uuid:{uuid}` format `BraidId`.
    ///
    /// Returns `None` if the `BraidId` is not in UUID format (e.g., hash-based IDs).
    #[must_use]
    pub fn extract_uuid(&self) -> Option<Uuid> {
        self.0
            .strip_prefix("urn:braid:uuid:")
            .and_then(|s| s.parse::<Uuid>().ok())
    }
}

impl<'de> Deserialize<'de> for BraidId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self(Arc::from(s)))
    }
}

impl Default for BraidId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for BraidId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
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

/// W3C PROV-O vocabulary namespace.
pub const PROV_VOCAB_URI: &str = "http://www.w3.org/ns/prov#";
/// W3C XML Schema namespace.
pub const XSD_VOCAB_URI: &str = "http://www.w3.org/2001/XMLSchema#";
/// Schema.org namespace.
pub const SCHEMA_VOCAB_URI: &str = "http://schema.org/";
/// ecoPrimals vocabulary namespace (discovered at runtime in production).
pub const ECOP_VOCAB_URI: &str = "https://ecoprimals.io/vocab#";
/// ecoPrimals base URI (discovered at runtime in production).
pub const ECOP_BASE_URI: &str = "https://ecoprimals.io/";

/// JSON-LD context for semantic interpretation.
///
/// Uses [`IndexMap`] for vocabulary imports to guarantee deterministic
/// serialization order — important for content-addressed hashing and
/// reproducible JSON-LD output.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BraidContext {
    /// Base context URL.
    #[serde(rename = "@base")]
    pub base: String,

    /// JSON-LD version.
    #[serde(rename = "@version")]
    pub version: f32,

    /// Vocabulary imports (insertion-ordered for deterministic serialization).
    #[serde(flatten)]
    pub imports: IndexMap<String, String>,
}

impl Default for BraidContext {
    fn default() -> Self {
        let mut imports = IndexMap::new();
        imports.insert("prov".to_string(), PROV_VOCAB_URI.to_string());
        imports.insert("xsd".to_string(), XSD_VOCAB_URI.to_string());
        imports.insert("schema".to_string(), SCHEMA_VOCAB_URI.to_string());
        imports.insert("ecop".to_string(), ECOP_VOCAB_URI.to_string());

        Self {
            base: ECOP_BASE_URI.to_string(),
            version: 1.1,
            imports,
        }
    }
}

/// Well-known signature type for Ed25519.
const SIG_TYPE_ED25519: &str = "Ed25519Signature2020";
/// Signature type for unsigned placeholders.
const SIG_TYPE_UNSIGNED: &str = "Unsigned";
/// Standard proof purpose for assertion signatures.
const PROOF_PURPOSE_ASSERTION: &str = "assertionMethod";
/// Proof purpose for pending/unsigned signatures.
const PROOF_PURPOSE_PENDING: &str = "pending";

/// Braid signature (W3C Data Integrity format).
///
/// Uses `Cow<'static, str>` for fields with well-known static values
/// (`sig_type`, `proof_purpose`) to avoid unnecessary heap allocations
/// while still supporting dynamic values.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BraidSignature {
    /// Signature type (e.g., `Ed25519Signature2020`).
    #[serde(rename = "type")]
    pub sig_type: Cow<'static, str>,

    /// When the signature was created.
    pub created: Timestamp,

    /// Verification method (key reference).
    pub verification_method: Cow<'static, str>,

    /// Proof purpose.
    pub proof_purpose: Cow<'static, str>,

    /// Base64-encoded signature value.
    pub proof_value: Cow<'static, str>,
}

impl BraidSignature {
    /// Create a new Ed25519 signature.
    #[must_use]
    pub fn new_ed25519(did: &Did, key_id: &str, signature_bytes: &[u8]) -> Self {
        use base64::Engine;
        Self {
            sig_type: Cow::Borrowed(SIG_TYPE_ED25519),
            created: current_timestamp_nanos(),
            verification_method: Cow::Owned(format!("{}#{key_id}", did.as_str())),
            proof_purpose: Cow::Borrowed(PROOF_PURPOSE_ASSERTION),
            proof_value: Cow::Owned(
                base64::engine::general_purpose::STANDARD.encode(signature_bytes),
            ),
        }
    }

    /// Create an unsigned placeholder signature.
    #[must_use]
    pub fn unsigned() -> Self {
        Self {
            sig_type: Cow::Borrowed(SIG_TYPE_UNSIGNED),
            created: current_timestamp_nanos(),
            verification_method: Cow::Borrowed(""),
            proof_purpose: Cow::Borrowed(PROOF_PURPOSE_PENDING),
            proof_value: Cow::Borrowed(""),
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

/// Get current timestamp in nanoseconds since Unix epoch.
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    reason = "u128->u64 truncation only occurs for dates beyond ~year 2554; acceptable for timestamp"
)]
pub fn current_timestamp_nanos() -> Timestamp {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}
