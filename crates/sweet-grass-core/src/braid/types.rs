// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Braid type definitions: `ContentHash`, `BraidId`, `BraidContext`, `BraidType`, etc.

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

use indexmap::IndexMap;
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use super::braid_type::{BraidType, SummaryType};
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

/// W3C PROV-O vocabulary namespace.
pub const PROV_VOCAB_URI: &str = "http://www.w3.org/ns/prov#";
/// W3C XML Schema namespace.
pub const XSD_VOCAB_URI: &str = "http://www.w3.org/2001/XMLSchema#";
/// W3C RDF Schema namespace.
pub const RDFS_VOCAB_URI: &str = "http://www.w3.org/2000/01/rdf-schema#";
/// Schema.org namespace.
pub const SCHEMA_VOCAB_URI: &str = "http://schema.org/";

/// Default ecoPrimals vocabulary namespace.
///
/// In production, override via `ECOP_VOCAB_URI` env var or capability-based
/// discovery. These defaults are safe for development and testing.
pub const DEFAULT_ECOP_VOCAB_URI: &str = "https://ecoprimals.io/vocab#";
/// Default ecoPrimals base URI.
///
/// In production, override via `ECOP_BASE_URI` env var or capability-based
/// discovery.
pub const DEFAULT_ECOP_BASE_URI: &str = "https://ecoprimals.io/";

/// Resolve the ecoPrimals vocabulary URI from environment or default.
#[must_use]
pub fn ecop_vocab_uri() -> String {
    std::env::var("ECOP_VOCAB_URI").unwrap_or_else(|_| DEFAULT_ECOP_VOCAB_URI.to_string())
}

/// Resolve the ecoPrimals base URI from environment or default.
#[must_use]
pub fn ecop_base_uri() -> String {
    std::env::var("ECOP_BASE_URI").unwrap_or_else(|_| DEFAULT_ECOP_BASE_URI.to_string())
}

/// JSON-LD context version — always 1.1 per W3C specification.
///
/// Avoids float representation issues by using a dedicated type that
/// serializes to the JSON number `1.1` and validates on deserialization.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsonLdVersion;

impl Serialize for JsonLdVersion {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f64(1.1)
    }
}

impl<'de> Deserialize<'de> for JsonLdVersion {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let v = f64::deserialize(deserializer)?;
        if (v - 1.1).abs() < 0.1 {
            Ok(Self)
        } else {
            Err(serde::de::Error::custom(format!(
                "unsupported JSON-LD version: {v} (expected 1.1)"
            )))
        }
    }
}

/// JSON-LD context for semantic interpretation.
///
/// Uses [`IndexMap`] for vocabulary imports to guarantee deterministic
/// serialization order — important for content-addressed hashing and
/// reproducible JSON-LD output.
///
/// **Serialization note**: `#[serde(flatten)]` is incompatible with binary
/// codecs such as bincode (tarpc). For human-readable formats (JSON), imports
/// remain flattened into the root object. For non-human-readable serializers,
/// `imports` is a normal nested field so lengths are known on the wire.
#[derive(Clone, Debug)]
pub struct BraidContext {
    /// Base context URL.
    pub base: String,

    /// JSON-LD version (always 1.1).
    pub version: JsonLdVersion,

    /// Vocabulary imports (insertion-ordered for deterministic serialization).
    pub imports: IndexMap<String, String>,
}

#[derive(Serialize)]
struct BraidContextSerBin<'a> {
    #[serde(rename = "@base")]
    base: &'a str,
    #[serde(rename = "@version")]
    version: JsonLdVersion,
    imports: &'a IndexMap<String, String>,
}

#[derive(Deserialize)]
struct BraidContextDeBin {
    #[serde(rename = "@base")]
    base: String,
    #[serde(rename = "@version")]
    version: JsonLdVersion,
    imports: IndexMap<String, String>,
}

#[derive(Deserialize)]
struct BraidContextFlat {
    #[serde(rename = "@base")]
    base: String,
    #[serde(rename = "@version")]
    version: JsonLdVersion,
    #[serde(flatten)]
    imports: IndexMap<String, String>,
}

impl Serialize for BraidContext {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if !serializer.is_human_readable() {
            return BraidContextSerBin {
                base: &self.base,
                version: self.version,
                imports: &self.imports,
            }
            .serialize(serializer);
        }

        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("@base", &self.base)?;
        map.serialize_entry("@version", &self.version)?;
        for (k, v) in &self.imports {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for BraidContext {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if !deserializer.is_human_readable() {
            let b = BraidContextDeBin::deserialize(deserializer)?;
            return Ok(Self {
                base: b.base,
                version: b.version,
                imports: b.imports,
            });
        }

        let f = BraidContextFlat::deserialize(deserializer)?;
        Ok(Self {
            base: f.base,
            version: f.version,
            imports: f.imports,
        })
    }
}

impl Default for BraidContext {
    fn default() -> Self {
        let mut imports = IndexMap::new();
        imports.insert("prov".to_string(), PROV_VOCAB_URI.to_string());
        imports.insert("xsd".to_string(), XSD_VOCAB_URI.to_string());
        imports.insert("schema".to_string(), SCHEMA_VOCAB_URI.to_string());
        imports.insert("ecop".to_string(), ecop_vocab_uri());

        Self {
            base: ecop_base_uri(),
            version: JsonLdVersion,
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

/// Braid signature (W3C Data Integrity format) — **deprecated**.
///
/// Superseded by [`crate::dehydration::Witness`] (`WireWitnessRef` vocabulary).
/// Retained for one release cycle so that persisted JSONB rows can
/// still be deserialized.
#[deprecated(
    since = "0.7.28",
    note = "use crate::dehydration::Witness (WireWitnessRef)"
)]
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

#[expect(
    deprecated,
    reason = "impl for the deprecated type itself; remove with BraidSignature in v0.7.29"
)]
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
///
/// String fields use `Arc<str>` for O(1) clone — these values are shared
/// across all Braids created by the same factory/engine instance.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EcoPrimalsAttributes {
    /// Source primal that created this Braid.
    pub source_primal: Option<Arc<str>>,

    /// Niche context.
    pub niche: Option<Arc<str>>,

    /// `RhizoCrypt` session reference.
    pub rhizo_session: Option<String>,

    /// `LoamSpine` commit reference.
    pub loam_commit: Option<LoamCommitRef>,

    /// Certificate reference.
    pub certificate: Option<String>,

    /// Compression metadata.
    pub compression: Option<CompressionMeta>,

    /// Witnesses carried from the dehydration event (signatures, hashes,
    /// checkpoints, markers). The trio never interprets evidence —
    /// verification is delegated to `BearDog` or an external verifier.
    #[serde(default)]
    pub witnesses: Vec<crate::dehydration::Witness>,
}

/// `LoamSpine` commit reference.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoamCommitRef {
    /// Spine ID.
    pub spine_id: Arc<str>,
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

#[cfg(test)]
#[expect(clippy::expect_used, reason = "bincode roundtrip test setup")]
mod tests {
    use std::borrow::Borrow;
    use std::sync::Arc;

    use super::{
        BraidContext, BraidId, BraidMetadata, BraidType, ContentHash, DEFAULT_ECOP_BASE_URI,
        DEFAULT_ECOP_VOCAB_URI, JsonLdVersion, SummaryType, ecop_base_uri, ecop_vocab_uri,
    };
    use crate::agent::Did;

    #[test]
    fn braid_type_bincode_roundtrip_entity() {
        let bt = BraidType::Entity;
        let bytes = bincode::serialize(&bt).expect("serialize");
        let decoded: BraidType = bincode::deserialize(&bytes).expect("deserialize");
        assert_eq!(decoded, BraidType::Entity);
    }

    #[test]
    fn braid_type_bincode_roundtrip_activity() {
        let bt = BraidType::Activity;
        let bytes = bincode::serialize(&bt).expect("serialize");
        let decoded: BraidType = bincode::deserialize(&bytes).expect("deserialize");
        assert_eq!(decoded, BraidType::Activity);
    }

    #[test]
    fn braid_type_bincode_roundtrip_agent() {
        let bt = BraidType::Agent;
        let bytes = bincode::serialize(&bt).expect("serialize");
        let decoded: BraidType = bincode::deserialize(&bytes).expect("deserialize");
        assert_eq!(decoded, BraidType::Agent);
    }

    #[test]
    fn braid_type_bincode_roundtrip_collection() {
        let bt = BraidType::Collection {
            member_count: 5,
            summary_type: SummaryType::Session {
                session_id: "s1".into(),
            },
        };
        let bytes = bincode::serialize(&bt).expect("serialize");
        let decoded: BraidType = bincode::deserialize(&bytes).expect("deserialize");
        assert_eq!(decoded, bt);
    }

    #[test]
    fn braid_type_bincode_roundtrip_delegation() {
        let bt = BraidType::Delegation {
            delegate: Did::new("did:key:delegate"),
            on_behalf_of: Did::new("did:key:principal"),
        };
        let bytes = bincode::serialize(&bt).expect("serialize");
        let decoded: BraidType = bincode::deserialize(&bytes).expect("deserialize");
        assert_eq!(decoded, bt);
    }

    #[test]
    fn braid_type_bincode_roundtrip_slice() {
        let bt = BraidType::Slice {
            slice_mode: "window".into(),
            origin_spine: "spine-001".into(),
        };
        let bytes = bincode::serialize(&bt).expect("serialize");
        let decoded: BraidType = bincode::deserialize(&bytes).expect("deserialize");
        assert_eq!(decoded, bt);
    }

    #[test]
    fn braid_context_bincode_roundtrip() {
        let ctx = BraidContext::default();
        let bytes = bincode::serialize(&ctx).expect("serialize");
        let decoded: BraidContext = bincode::deserialize(&bytes).expect("deserialize");
        assert_eq!(decoded.base, ctx.base);
        assert_eq!(decoded.imports.len(), ctx.imports.len());
    }

    #[test]
    fn braid_metadata_bincode_roundtrip_with_custom() {
        let meta = BraidMetadata {
            title: Some(Arc::from("test")),
            tags: vec![Arc::from("tag1"), Arc::from("tag2")],
            custom: [
                ("key".to_string(), serde_json::json!(42)),
                ("nested".to_string(), serde_json::json!({"a": 1})),
            ]
            .into_iter()
            .collect(),
            ..Default::default()
        };

        let bytes = bincode::serialize(&meta).expect("serialize");
        let decoded: BraidMetadata = bincode::deserialize(&bytes).expect("deserialize");
        assert_eq!(decoded.title.as_deref(), Some("test"));
        assert_eq!(decoded.tags.len(), 2);
        assert_eq!(decoded.custom["key"], serde_json::json!(42));
    }

    #[test]
    fn content_hash_from_str_ref() {
        let h = ContentHash::from("sha256:abc");
        assert_eq!(h.as_str(), "sha256:abc");
    }

    #[test]
    fn content_hash_from_string_ref() {
        let s = String::from("sha256:xyz");
        let h = ContentHash::from(&s);
        assert_eq!(h.as_str(), "sha256:xyz");
    }

    #[test]
    fn content_hash_from_self_ref() {
        let h1 = ContentHash::new("sha256:test");
        let h2 = ContentHash::from(&h1);
        assert_eq!(h1, h2);
    }

    #[test]
    fn content_hash_partial_eq_str() {
        let h = ContentHash::new("sha256:cmp");
        assert!(h.eq("sha256:cmp"));
        assert!(!h.eq("sha256:other"));
    }

    #[test]
    fn content_hash_borrow_str() {
        let h = ContentHash::new("sha256:borrow");
        let s: &str = h.borrow();
        assert_eq!(s, "sha256:borrow");
    }

    #[test]
    fn content_hash_as_ref_str() {
        let h = ContentHash::new("sha256:asref");
        let s: &str = h.as_ref();
        assert_eq!(s, "sha256:asref");
    }

    #[test]
    fn content_hash_display() {
        let h = ContentHash::new("sha256:display");
        assert_eq!(format!("{h}"), "sha256:display");
    }

    #[test]
    #[expect(deprecated, reason = "testing deprecated BraidSignature")]
    fn braid_signature_ed25519() {
        let did = Did::new("did:key:z6MkSigner");
        let sig = super::BraidSignature::new_ed25519(&did, "key-1", b"test-sig");
        assert_eq!(&*sig.sig_type, "Ed25519Signature2020");
        assert!(!sig.proof_value.is_empty());
        assert_eq!(&*sig.proof_purpose, "assertionMethod");
        assert!(sig.verification_method.contains("key-1"));
    }

    #[test]
    #[expect(deprecated, reason = "testing deprecated BraidSignature")]
    fn braid_signature_unsigned() {
        let sig = super::BraidSignature::unsigned();
        assert_eq!(&*sig.sig_type, "Unsigned");
        assert!(sig.proof_value.is_empty());
        assert_eq!(&*sig.proof_purpose, "pending");
    }

    #[test]
    fn json_ld_version_rejects_bad_version() {
        let result: Result<JsonLdVersion, _> = serde_json::from_str("2.0");
        let err = result
            .expect_err("2.0 must not deserialize as JSON-LD 1.1")
            .to_string();
        assert!(err.contains("unsupported JSON-LD version"), "{err}");
    }

    #[test]
    fn ecop_vocab_uri_default() {
        temp_env::with_vars([("ECOP_VOCAB_URI", None::<&str>)], || {
            assert_eq!(ecop_vocab_uri(), DEFAULT_ECOP_VOCAB_URI);
        });
    }

    #[test]
    fn ecop_vocab_uri_env_override() {
        temp_env::with_vars(
            [("ECOP_VOCAB_URI", Some("https://custom.io/vocab#"))],
            || {
                assert_eq!(ecop_vocab_uri(), "https://custom.io/vocab#");
            },
        );
    }

    #[test]
    fn ecop_base_uri_default() {
        temp_env::with_vars([("ECOP_BASE_URI", None::<&str>)], || {
            assert_eq!(ecop_base_uri(), DEFAULT_ECOP_BASE_URI);
        });
    }

    #[test]
    fn braid_id_display() {
        let id = BraidId::from_string("urn:braid:uuid:test");
        assert_eq!(format!("{id}"), "urn:braid:uuid:test");
    }

    #[test]
    fn braid_id_from_hash() {
        let h = ContentHash::new("sha256:abc");
        let id = BraidId::from_hash(&h);
        assert_eq!(id.as_str(), "urn:braid:sha256:abc");
    }

    #[test]
    fn content_hash_default() {
        let h = ContentHash::default();
        assert_eq!(h.as_str(), "");
    }

    #[test]
    fn braid_id_default() {
        let id = BraidId::default();
        assert!(id.as_str().starts_with("urn:braid:uuid:"));
    }

    #[test]
    fn braid_type_json_roundtrip_collection() {
        let bt = BraidType::Collection {
            member_count: 7,
            summary_type: SummaryType::Temporal {
                start: 100,
                end: 999,
            },
        };
        let json = serde_json::to_string(&bt).expect("serialize");
        let decoded: BraidType = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, bt);
    }

    #[test]
    fn braid_type_json_roundtrip_delegation() {
        let bt = BraidType::Delegation {
            delegate: Did::new("did:key:delegate"),
            on_behalf_of: Did::new("did:key:principal"),
        };
        let json = serde_json::to_string(&bt).expect("serialize");
        let decoded: BraidType = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, bt);
    }

    #[test]
    fn braid_type_json_roundtrip_slice() {
        let bt = BraidType::Slice {
            slice_mode: "window".into(),
            origin_spine: "spine-001".into(),
        };
        let json = serde_json::to_string(&bt).expect("serialize");
        let decoded: BraidType = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, bt);
    }

    #[test]
    fn braid_type_json_roundtrip_entity_activity_agent() {
        for bt in [BraidType::Entity, BraidType::Activity, BraidType::Agent] {
            let json = serde_json::to_string(&bt).expect("serialize");
            let decoded: BraidType = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(decoded, bt);
        }
    }

    #[test]
    fn braid_id_extract_uuid() {
        let id = BraidId::new();
        assert!(id.extract_uuid().is_some());

        let hash_id = BraidId::from_hash(&ContentHash::new("sha256:test"));
        assert!(hash_id.extract_uuid().is_none());
    }
}
