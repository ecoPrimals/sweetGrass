// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Braid type definitions: `ContentHash`, `BraidId`, `BraidContext`, `BraidType`, etc.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use super::braid_type::{BraidType, SummaryType};
pub use super::context::{
    BraidContext, DEFAULT_ECOP_BASE_URI, DEFAULT_ECOP_VOCAB_URI, JsonLdVersion, PROV_VOCAB_URI,
    RDFS_VOCAB_URI, SCHEMA_VOCAB_URI, XSD_VOCAB_URI, ecop_base_uri, ecop_base_uri_with_reader,
    ecop_vocab_uri, ecop_vocab_uri_with_reader,
};
pub use super::cross_gate::{CrossGateAttribution, CrossGateTrustEvent};
use crate::hash::hex_decode;
use crate::privacy::PrivacyMetadata;

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
///
/// Wraps `u64` for type safety — prevents accidental mixing of nanosecond
/// timestamps with arbitrary integers or second-precision values.
/// Wire-compatible with plain `u64` via `#[serde(transparent)]`.
#[derive(
    Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct Timestamp(u64);

impl Timestamp {
    /// Zero timestamp (Unix epoch).
    pub const ZERO: Self = Self(0);

    /// Create a timestamp from raw nanoseconds.
    #[must_use]
    pub const fn new(nanos: u64) -> Self {
        Self(nanos)
    }

    /// Get the raw nanosecond value.
    #[must_use]
    pub const fn nanos(self) -> u64 {
        self.0
    }

    /// Current wall-clock time as nanoseconds since epoch.
    #[must_use]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "u128->u64 truncation only occurs for dates beyond ~year 2554"
    )]
    pub fn now() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        Self(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |d| d.as_nanos() as u64),
        )
    }
}

impl From<u64> for Timestamp {
    fn from(nanos: u64) -> Self {
        Self(nanos)
    }
}

impl std::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

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

    /// Return a UUID for this braid — either the embedded UUID (for UUID-based
    /// IDs) or a deterministic v5 UUID derived from the `braid_id` string (for
    /// hash-based IDs). Always returns a valid UUID suitable for cross-primal
    /// correlation (loamSpine ledger entries, nestGate CAS keys, etc.).
    #[must_use]
    pub fn to_uuid(&self) -> Uuid {
        self.extract_uuid()
            .unwrap_or_else(|| Uuid::new_v5(&namespace_braid(), self.0.as_bytes()))
    }
}

/// UUID v5 namespace for deterministic `braid_id` → UUID derivation.
fn namespace_braid() -> Uuid {
    Uuid::new_v5(&Uuid::NAMESPACE_URL, b"urn:ecoPrimals:braid")
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

/// Anchoring provider anchor information.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoamAnchor {
    /// Spine where anchored.
    pub spine_id: Arc<str>,

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
///
/// String fields use `Arc<str>` for O(1) clone — these values are shared
/// across all Braids created by the same factory/engine instance.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EcoPrimalsAttributes {
    /// Source primal that created this Braid.
    pub source_primal: Option<Arc<str>>,

    /// Gate that originated this Braid (e.g. "strandGate", "ironGate").
    #[serde(default)]
    pub source_gate: Option<Arc<str>>,

    /// Niche context.
    pub niche: Option<Arc<str>>,

    /// Session events provider session reference (capability-based).
    #[serde(alias = "rhizo_session")]
    pub session_ref: Option<Arc<str>>,

    /// Permanent ledger commit reference (capability-based).
    #[serde(alias = "loam_commit")]
    pub ledger_commit: Option<LedgerCommitRef>,

    /// Certificate reference linking this braid to a loamSpine certificate.
    ///
    /// Structured as `CertificateRef` for cross-gate provenance chains.
    /// Deserializes from a plain string (legacy) or structured object.
    #[serde(default)]
    pub certificate: Option<CertificateRef>,

    /// Compression metadata.
    pub compression: Option<CompressionMeta>,

    /// Witnesses carried from the dehydration event (signatures, hashes,
    /// checkpoints, markers). The trio never interprets evidence —
    /// verification is delegated to `BearDog` or an external verifier.
    #[serde(default)]
    pub witnesses: Vec<crate::dehydration::Witness>,
}

/// Permanent ledger commit reference (capability-based).
///
/// Represents a commit to the permanent ledger provider, discovered at runtime.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LedgerCommitRef {
    /// Ledger spine/partition identifier.
    pub spine_id: Arc<str>,
    /// Entry content hash.
    pub entry_hash: ContentHash,
    /// Entry index in the ledger.
    pub index: u64,
}

/// Backward-compatible type alias.
pub type LoamCommitRef = LedgerCommitRef;

/// Certificate reference linking a braid to a loamSpine certificate.
///
/// Provides the structured linkage between attribution braids and the
/// certificate lifecycle (Nest Atomic G3 convergence gap #2).
///
/// Deserializes from either:
/// - A plain string (`"cert-001"`) → `CertificateRef { id, .. defaults }`
/// - A structured object with full metadata
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "CertificateRefWire")]
pub struct CertificateRef {
    /// Unique certificate identifier (loamSpine-issued).
    pub id: Arc<str>,
    /// Gate that issued/minted this certificate.
    #[serde(default)]
    pub issuing_gate: Option<Arc<str>>,
    /// Whether the certificate has been sealed in the ledger.
    #[serde(default)]
    pub sealed: bool,
    /// DID of the minting authority.
    #[serde(default)]
    pub minting_authority: Option<Arc<str>>,
    /// Content hash of the certificate body (for CAS lookup).
    #[serde(default)]
    pub content_hash: Option<ContentHash>,
}

impl CertificateRef {
    /// Create a simple certificate reference from an ID string.
    #[must_use]
    pub fn new(id: impl AsRef<str>) -> Self {
        Self {
            id: Arc::from(id.as_ref()),
            issuing_gate: None,
            sealed: false,
            minting_authority: None,
            content_hash: None,
        }
    }

    /// Create a fully-specified certificate reference for cross-gate attestation.
    #[must_use]
    pub fn cross_gate(
        id: impl AsRef<str>,
        issuing_gate: impl AsRef<str>,
        minting_authority: impl AsRef<str>,
    ) -> Self {
        Self {
            id: Arc::from(id.as_ref()),
            issuing_gate: Some(Arc::from(issuing_gate.as_ref())),
            sealed: false,
            minting_authority: Some(Arc::from(minting_authority.as_ref())),
            content_hash: None,
        }
    }

    /// Mark the certificate as sealed (ledger-committed).
    #[must_use]
    pub const fn with_sealed(mut self) -> Self {
        self.sealed = true;
        self
    }

    /// Attach a CAS content hash for the certificate body.
    #[must_use]
    pub fn with_content_hash(mut self, hash: ContentHash) -> Self {
        self.content_hash = Some(hash);
        self
    }

    /// The certificate identifier.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
}

/// Wire format for backward-compatible deserialization.
#[derive(Deserialize)]
#[serde(untagged)]
enum CertificateRefWire {
    /// Plain string (legacy format).
    Plain(String),
    /// Structured object (G3 format).
    Structured {
        id: Arc<str>,
        #[serde(default)]
        issuing_gate: Option<Arc<str>>,
        #[serde(default)]
        sealed: bool,
        #[serde(default)]
        minting_authority: Option<Arc<str>>,
        #[serde(default)]
        content_hash: Option<ContentHash>,
    },
}

impl From<CertificateRefWire> for CertificateRef {
    fn from(wire: CertificateRefWire) -> Self {
        match wire {
            CertificateRefWire::Plain(id) => Self::new(id),
            CertificateRefWire::Structured {
                id,
                issuing_gate,
                sealed,
                minting_authority,
                content_hash,
            } => Self {
                id,
                issuing_gate,
                sealed,
                minting_authority,
                content_hash,
            },
        }
    }
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

fn serialize_json_value_map<S>(
    map: &HashMap<String, serde_json::Value>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if !serializer.is_human_readable() {
        let string_map: HashMap<String, String> = map
            .iter()
            .map(|(k, v)| {
                serde_json::to_string(v)
                    .map(|s| (k.clone(), s))
                    .map_err(serde::ser::Error::custom)
            })
            .collect::<Result<_, _>>()?;
        return string_map.serialize(serializer);
    }
    map.serialize(serializer)
}

fn deserialize_json_value_map<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, serde_json::Value>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    if deserializer.is_human_readable() {
        HashMap::deserialize(deserializer)
    } else {
        let string_map: HashMap<String, String> = HashMap::deserialize(deserializer)?;
        string_map
            .into_iter()
            .map(|(k, s)| {
                serde_json::from_str(&s)
                    .map(|v| (k, v))
                    .map_err(serde::de::Error::custom)
            })
            .collect()
    }
}

/// Domain-specific metadata.
///
/// String fields use `Arc<str>` for O(1) clone — metadata is shared across
/// query results and response serialization without per-field allocation.
///
/// The `custom` map uses JSON values on human-readable transports; for binary
/// codecs (e.g. bincode/tarpc), each value is stored as a UTF-8 JSON string.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BraidMetadata {
    /// Title or name.
    pub title: Option<Arc<str>>,

    /// Description.
    pub description: Option<Arc<str>>,

    /// Tags/keywords.
    #[serde(default)]
    pub tags: Vec<Arc<str>>,

    /// Custom key-value metadata.
    #[serde(
        default,
        serialize_with = "serialize_json_value_map",
        deserialize_with = "deserialize_json_value_map"
    )]
    pub custom: HashMap<String, serde_json::Value>,

    /// Privacy controls for this braid.
    #[serde(default)]
    pub privacy: Option<PrivacyMetadata>,

    /// Cross-gate trust attribution (bearDog w135 mesh events).
    #[serde(default)]
    pub cross_gate: Option<CrossGateAttribution>,
}

/// Get current timestamp in nanoseconds since Unix epoch.
///
/// Convenience wrapper around [`Timestamp::now()`].
#[must_use]
pub fn current_timestamp_nanos() -> Timestamp {
    Timestamp::now()
}
