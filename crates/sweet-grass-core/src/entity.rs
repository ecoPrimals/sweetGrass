// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Entity reference data structures - links to data artifacts.
//!
//! Entity references allow Braids to link to other data without
//! embedding it directly.

use serde::{Deserialize, Serialize};

use crate::braid::{BraidId, ContentHash};

/// Reference to a PROV entity.
///
/// **Serialization**: JSON uses the legacy untagged shape (`EntityReferenceHuman`);
/// binary codecs (e.g. bincode/tarpc) use an externally tagged enum so `serde` never
/// relies on `deserialize_any`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum EntityReference {
    /// Reference by Braid ID.
    ById {
        /// The Braid ID.
        braid_id: BraidId,
    },

    /// Reference by content hash.
    ByHash {
        /// The content hash.
        data_hash: ContentHash,
        /// Optional MIME type.
        mime_type: Option<String>,
    },

    /// Reference by anchoring provider location.
    ByLoamEntry {
        /// The spine ID.
        spine_id: String,
        /// The entry hash.
        entry_hash: ContentHash,
    },

    /// External reference (URL).
    External {
        /// The URL.
        url: String,
        /// Optional content hash for verification.
        hash: Option<ContentHash>,
    },

    /// Inline entity (for small data).
    Inline(InlineEntity),
}

impl EntityReference {
    /// Create a reference by Braid ID.
    #[must_use]
    pub const fn by_id(braid_id: BraidId) -> Self {
        Self::ById { braid_id }
    }

    /// Create a reference by content hash.
    #[must_use]
    pub fn by_hash(hash: impl Into<ContentHash>) -> Self {
        Self::ByHash {
            data_hash: hash.into(),
            mime_type: None,
        }
    }

    /// Create a reference by content hash with MIME type.
    #[must_use]
    pub fn by_hash_typed(hash: impl Into<ContentHash>, mime_type: impl Into<String>) -> Self {
        Self::ByHash {
            data_hash: hash.into(),
            mime_type: Some(mime_type.into()),
        }
    }

    /// Create a reference by permanent ledger entry (capability-based).
    #[must_use]
    pub fn by_ledger_entry(
        spine_id: impl Into<String>,
        entry_hash: impl Into<ContentHash>,
    ) -> Self {
        Self::ByLoamEntry {
            spine_id: spine_id.into(),
            entry_hash: entry_hash.into(),
        }
    }

    /// Backward-compatible alias for [`by_ledger_entry`](Self::by_ledger_entry).
    #[deprecated(
        since = "0.7.28",
        note = "use by_ledger_entry (capability-based naming)"
    )]
    #[must_use]
    pub fn by_loam_entry(spine_id: impl Into<String>, entry_hash: impl Into<ContentHash>) -> Self {
        Self::by_ledger_entry(spine_id, entry_hash)
    }

    /// Create an external reference.
    #[must_use]
    pub fn external(url: impl Into<String>) -> Self {
        Self::External {
            url: url.into(),
            hash: None,
        }
    }

    /// Create an external reference with hash verification.
    #[must_use]
    pub fn external_verified(url: impl Into<String>, hash: impl Into<ContentHash>) -> Self {
        Self::External {
            url: url.into(),
            hash: Some(hash.into()),
        }
    }

    /// Create an inline entity reference.
    #[must_use]
    pub const fn inline(entity: InlineEntity) -> Self {
        Self::Inline(entity)
    }

    /// Get the content hash if available.
    #[must_use]
    pub const fn content_hash(&self) -> Option<&ContentHash> {
        match self {
            Self::ById { .. } => None,
            Self::ByHash { data_hash, .. } => Some(data_hash),
            Self::ByLoamEntry { entry_hash, .. } => Some(entry_hash),
            Self::External { hash, .. } => hash.as_ref(),
            Self::Inline(entity) => Some(&entity.hash),
        }
    }

    /// Check if this is an inline reference.
    #[must_use]
    pub const fn is_inline(&self) -> bool {
        matches!(self, Self::Inline(_))
    }

    /// Check if this is an external reference.
    #[must_use]
    pub const fn is_external(&self) -> bool {
        matches!(self, Self::External { .. })
    }
}

/// JSON (human-readable) wire shape for [`EntityReference`] — untagged, matches historical API.
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum EntityReferenceHuman {
    /// Reference by Braid ID.
    ById {
        /// The Braid ID.
        braid_id: BraidId,
    },
    /// Reference by content hash.
    ByHash {
        /// The content hash.
        data_hash: ContentHash,
        /// Optional MIME type.
        #[serde(skip_serializing_if = "Option::is_none")]
        mime_type: Option<String>,
    },
    /// Reference by anchoring provider location.
    ByLoamEntry {
        /// The spine ID.
        spine_id: String,
        /// The entry hash.
        entry_hash: ContentHash,
    },
    /// External reference (URL).
    External {
        /// The URL.
        url: String,
        /// Optional content hash for verification.
        #[serde(skip_serializing_if = "Option::is_none")]
        hash: Option<ContentHash>,
    },
    /// Inline entity (for small data).
    Inline(InlineEntity),
}

/// Binary wire shape for [`EntityReference`] — externally tagged for bincode compatibility.
///
/// Optional fields are always serialized as `Option` values (no `skip_serializing_if`) so
/// non–self-describing codecs like bincode can roundtrip every variant.
#[derive(Serialize, Deserialize)]
enum EntityReferenceBin {
    /// Reference by Braid ID.
    ById {
        /// The Braid ID.
        braid_id: BraidId,
    },
    /// Reference by content hash.
    ByHash {
        /// The content hash.
        data_hash: ContentHash,
        /// Optional MIME type.
        mime_type: Option<String>,
    },
    /// Reference by anchoring provider location.
    ByLoamEntry {
        /// The spine ID.
        spine_id: String,
        /// The entry hash.
        entry_hash: ContentHash,
    },
    /// External reference (URL).
    External {
        /// The URL.
        url: String,
        /// Optional content hash for verification.
        hash: Option<ContentHash>,
    },
    /// Inline entity (for small data).
    Inline(InlineEntity),
}

impl From<EntityReference> for EntityReferenceHuman {
    fn from(r: EntityReference) -> Self {
        match r {
            EntityReference::ById { braid_id } => Self::ById { braid_id },
            EntityReference::ByHash {
                data_hash,
                mime_type,
            } => Self::ByHash {
                data_hash,
                mime_type,
            },
            EntityReference::ByLoamEntry {
                spine_id,
                entry_hash,
            } => Self::ByLoamEntry {
                spine_id,
                entry_hash,
            },
            EntityReference::External { url, hash } => Self::External { url, hash },
            EntityReference::Inline(entity) => Self::Inline(entity),
        }
    }
}

impl From<EntityReferenceHuman> for EntityReference {
    fn from(r: EntityReferenceHuman) -> Self {
        match r {
            EntityReferenceHuman::ById { braid_id } => Self::ById { braid_id },
            EntityReferenceHuman::ByHash {
                data_hash,
                mime_type,
            } => Self::ByHash {
                data_hash,
                mime_type,
            },
            EntityReferenceHuman::ByLoamEntry {
                spine_id,
                entry_hash,
            } => Self::ByLoamEntry {
                spine_id,
                entry_hash,
            },
            EntityReferenceHuman::External { url, hash } => Self::External { url, hash },
            EntityReferenceHuman::Inline(entity) => Self::Inline(entity),
        }
    }
}

impl From<&EntityReference> for EntityReferenceBin {
    fn from(r: &EntityReference) -> Self {
        match r {
            EntityReference::ById { braid_id } => Self::ById {
                braid_id: braid_id.clone(),
            },
            EntityReference::ByHash {
                data_hash,
                mime_type,
            } => Self::ByHash {
                data_hash: data_hash.clone(),
                mime_type: mime_type.clone(),
            },
            EntityReference::ByLoamEntry {
                spine_id,
                entry_hash,
            } => Self::ByLoamEntry {
                spine_id: spine_id.clone(),
                entry_hash: entry_hash.clone(),
            },
            EntityReference::External { url, hash } => Self::External {
                url: url.clone(),
                hash: hash.clone(),
            },
            EntityReference::Inline(entity) => Self::Inline(entity.clone()),
        }
    }
}

impl From<EntityReferenceBin> for EntityReference {
    fn from(r: EntityReferenceBin) -> Self {
        match r {
            EntityReferenceBin::ById { braid_id } => Self::ById { braid_id },
            EntityReferenceBin::ByHash {
                data_hash,
                mime_type,
            } => Self::ByHash {
                data_hash,
                mime_type,
            },
            EntityReferenceBin::ByLoamEntry {
                spine_id,
                entry_hash,
            } => Self::ByLoamEntry {
                spine_id,
                entry_hash,
            },
            EntityReferenceBin::External { url, hash } => Self::External { url, hash },
            EntityReferenceBin::Inline(entity) => Self::Inline(entity),
        }
    }
}

impl Serialize for EntityReference {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            EntityReferenceHuman::from(self.clone()).serialize(serializer)
        } else {
            EntityReferenceBin::from(self).serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for EntityReference {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            EntityReferenceHuman::deserialize(deserializer).map(Into::into)
        } else {
            EntityReferenceBin::deserialize(deserializer).map(Into::into)
        }
    }
}

/// Encoding for inline entity data.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum Encoding {
    /// Base64 encoding.
    #[default]
    Base64,
    /// UTF-8 text.
    Utf8,
    /// Hexadecimal encoding.
    Hex,
}

/// Inline entity for small data.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InlineEntity {
    /// Content type (MIME type).
    pub content_type: String,

    /// Encoding of the data.
    pub encoding: Encoding,

    /// The data (encoded according to `encoding`).
    pub data: String,

    /// Hash for verification.
    pub hash: ContentHash,
}

impl InlineEntity {
    /// Create a new inline entity from UTF-8 text.
    #[must_use]
    pub fn text(content: impl AsRef<str>, content_type: impl Into<String>) -> Self {
        let content = content.as_ref();
        let hash = compute_sha256(content.as_bytes());
        Self {
            content_type: content_type.into(),
            encoding: Encoding::Utf8,
            data: content.to_string(),
            hash,
        }
    }

    /// Create a new inline entity from bytes (Base64 encoded).
    #[must_use]
    pub fn bytes(content: &[u8], content_type: impl Into<String>) -> Self {
        use base64::Engine;
        let hash = compute_sha256(content);
        Self {
            content_type: content_type.into(),
            encoding: Encoding::Base64,
            data: base64::engine::general_purpose::STANDARD.encode(content),
            hash,
        }
    }

    /// Create a new inline entity from JSON-serializable data.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn json<T: Serialize>(value: &T) -> Result<Self, serde_json::Error> {
        let json = serde_json::to_string(value)?;
        Ok(Self::text(&json, "application/json"))
    }

    /// Get the decoded data as bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if decoding fails.
    pub fn decode(&self) -> Result<Vec<u8>, DecodeError> {
        match self.encoding {
            Encoding::Utf8 => Ok(self.data.as_bytes().to_vec()),
            Encoding::Base64 => {
                use base64::Engine;
                base64::engine::general_purpose::STANDARD
                    .decode(&self.data)
                    .map_err(|e| DecodeError::Base64(e.to_string()))
            },
            Encoding::Hex => Ok(hex_decode_strict(&self.data)?),
        }
    }

    /// Get decoded data as `Cow<[u8]>` - zero-copy for UTF-8 data.
    ///
    /// This returns borrowed bytes for UTF-8 encoded data, avoiding
    /// allocation. For Base64 and Hex encoded data, decoding is required
    /// so an owned `Vec<u8>` is returned.
    ///
    /// # Errors
    ///
    /// Returns an error if decoding fails.
    pub fn decode_cow(&self) -> Result<std::borrow::Cow<'_, [u8]>, DecodeError> {
        use std::borrow::Cow;
        match self.encoding {
            Encoding::Utf8 => Ok(Cow::Borrowed(self.data.as_bytes())),
            Encoding::Base64 => {
                use base64::Engine;
                let decoded = base64::engine::general_purpose::STANDARD
                    .decode(&self.data)
                    .map_err(|e| DecodeError::Base64(e.to_string()))?;
                Ok(Cow::Owned(decoded))
            },
            Encoding::Hex => {
                let decoded = hex_decode_strict(&self.data)?;
                Ok(Cow::Owned(decoded))
            },
        }
    }

    /// Verify the hash matches the content.
    ///
    /// # Errors
    ///
    /// Returns an error if decoding fails.
    pub fn verify(&self) -> Result<bool, DecodeError> {
        let decoded = self.decode()?;
        let computed = compute_sha256(&decoded);
        Ok(computed == self.hash)
    }
}

/// Error decoding inline entity data.
#[derive(Debug, Clone, thiserror::Error)]
#[non_exhaustive]
pub enum DecodeError {
    /// Base64 decoding error.
    #[error("base64 decode error: {0}")]
    Base64(String),

    /// Hex decoding error.
    #[error("hex decode error: {0}")]
    Hex(#[from] crate::hash::HexDecodeError),
}

use crate::hash::{hex_decode_strict, sha256 as compute_sha256};

#[cfg(test)]
#[expect(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    fn assert_entity_reference_bincode_roundtrip(original: &EntityReference) {
        let bytes = bincode::serialize(original).unwrap();
        let decoded: EntityReference = bincode::deserialize(&bytes).unwrap();
        assert_eq!(&decoded, original);
    }

    #[test]
    fn entity_reference_bincode_roundtrip_by_id() {
        let braid_id = BraidId::from_string("urn:braid:uuid:bincode-by-id");
        assert_entity_reference_bincode_roundtrip(&EntityReference::by_id(braid_id));
    }

    #[test]
    fn entity_reference_bincode_roundtrip_by_hash() {
        assert_entity_reference_bincode_roundtrip(&EntityReference::by_hash("sha256:abc123"));
        assert_entity_reference_bincode_roundtrip(&EntityReference::by_hash_typed(
            "sha256:typed",
            "application/json",
        ));
    }

    #[test]
    fn entity_reference_bincode_roundtrip_by_ledger_entry() {
        assert_entity_reference_bincode_roundtrip(&EntityReference::by_ledger_entry(
            "spine-bincode",
            "sha256:ledgerhash",
        ));
    }

    #[test]
    fn entity_reference_bincode_roundtrip_external() {
        assert_entity_reference_bincode_roundtrip(&EntityReference::external(
            "https://example.com/bincode.json",
        ));
        assert_entity_reference_bincode_roundtrip(&EntityReference::external_verified(
            "https://example.com/verified.bin",
            "sha256:extverify",
        ));
    }

    #[test]
    fn entity_reference_bincode_roundtrip_inline() {
        let inline = InlineEntity::text("bincode payload", "text/plain");
        assert_entity_reference_bincode_roundtrip(&EntityReference::inline(inline));
    }

    #[test]
    fn test_entity_reference_by_hash() {
        let entity = EntityReference::by_hash("sha256:abc123");
        assert_eq!(
            entity.content_hash().map(ContentHash::as_str),
            Some("sha256:abc123")
        );
        assert!(!entity.is_inline());
        assert!(!entity.is_external());
    }

    #[test]
    fn test_entity_reference_external() {
        let entity = EntityReference::external("https://example.com/data.json");
        assert!(entity.is_external());
        assert!(entity.content_hash().is_none());
    }

    #[test]
    fn test_entity_reference_external_verified() {
        let entity =
            EntityReference::external_verified("https://example.com/data.json", "sha256:abc123");
        assert!(entity.is_external());
        assert_eq!(
            entity.content_hash().map(ContentHash::as_str),
            Some("sha256:abc123")
        );
    }

    #[test]
    fn test_inline_entity_text() {
        let entity = InlineEntity::text("Hello, World!", "text/plain");
        assert_eq!(entity.encoding, Encoding::Utf8);
        assert_eq!(entity.data, "Hello, World!");
        assert!(entity.hash.as_str().starts_with("sha256:"));
        assert!(entity.verify().expect("should verify"));
    }

    #[test]
    fn test_inline_entity_bytes() {
        let data = b"binary data";
        let entity = InlineEntity::bytes(data, "application/octet-stream");
        assert_eq!(entity.encoding, Encoding::Base64);

        let decoded = entity.decode().expect("should decode");
        assert_eq!(decoded, data);
        assert!(entity.verify().expect("should verify"));
    }

    #[test]
    fn test_inline_entity_json() {
        #[derive(Serialize)]
        struct Data {
            value: i32,
        }

        let entity = InlineEntity::json(&Data { value: 42 }).expect("should create");
        assert_eq!(entity.content_type, "application/json");
        assert!(entity.data.contains("42"));
    }

    #[test]
    fn test_entity_reference_serialization() {
        let entity = EntityReference::by_hash_typed("sha256:abc123", "application/json");
        let json = serde_json::to_string(&entity).expect("should serialize");
        assert!(json.contains("sha256:abc123"));
        assert!(json.contains("application/json"));

        let parsed: EntityReference = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(parsed.content_hash(), entity.content_hash());
    }

    #[test]
    fn test_inline_entity_decode_cow_utf8() {
        use std::borrow::Cow;
        let entity = InlineEntity::text("Hello, World!", "text/plain");

        let decoded = entity.decode_cow().expect("should decode");
        assert!(
            matches!(decoded, Cow::Borrowed(_)),
            "UTF-8 should be borrowed"
        );
        assert_eq!(decoded.as_ref(), b"Hello, World!");
    }

    #[test]
    fn test_inline_entity_decode_cow_base64() {
        use std::borrow::Cow;
        let entity = InlineEntity::bytes(b"binary data", "application/octet-stream");

        let decoded = entity.decode_cow().expect("should decode");
        assert!(matches!(decoded, Cow::Owned(_)), "Base64 should be owned");
        assert_eq!(decoded.as_ref(), b"binary data");
    }

    #[test]
    fn test_inline_entity_serialization() {
        let entity = InlineEntity::text("test", "text/plain");
        let ref_entity = EntityReference::inline(entity);

        let json = serde_json::to_string(&ref_entity).expect("should serialize");
        let parsed: EntityReference = serde_json::from_str(&json).expect("should deserialize");

        assert!(parsed.is_inline());
    }

    #[test]
    fn test_entity_reference_by_id() {
        let braid_id = BraidId::from_string("urn:braid:uuid:test-123");
        let entity = EntityReference::by_id(braid_id.clone());
        assert!(matches!(&entity, EntityReference::ById { braid_id: id } if *id == braid_id));
        assert!(entity.content_hash().is_none());
        assert!(!entity.is_inline());
        assert!(!entity.is_external());
    }

    #[test]
    fn test_entity_reference_by_ledger_entry() {
        let entity = EntityReference::by_ledger_entry("spine-1", "sha256:entryhash123");
        assert_eq!(
            entity.content_hash().map(ContentHash::as_str),
            Some("sha256:entryhash123")
        );
        assert!(!entity.is_inline());
        assert!(!entity.is_external());

        let json = serde_json::to_string(&entity).expect("should serialize");
        assert!(json.contains("spine-1"));
        assert!(json.contains("sha256:entryhash123"));
    }

    #[test]
    fn test_inline_entity_decode_hex() {
        use crate::hash::hex_encode;
        let data = b"hex encoded data";
        let hex_data = hex_encode(data);
        let hash = crate::hash::sha256(data);
        let entity = InlineEntity {
            content_type: "application/octet-stream".to_string(),
            encoding: Encoding::Hex,
            data: hex_data,
            hash,
        };

        let decoded = entity.decode().expect("should decode hex");
        assert_eq!(decoded, data);
        assert!(entity.verify().expect("should verify"));
    }

    #[test]
    fn test_inline_entity_decode_cow_hex() {
        use crate::hash::hex_encode;
        use std::borrow::Cow;
        let data = b"hex cow test";
        let hex_data = hex_encode(data);
        let hash = crate::hash::sha256(data);
        let entity = InlineEntity {
            content_type: "application/octet-stream".to_string(),
            encoding: Encoding::Hex,
            data: hex_data,
            hash,
        };

        let decoded = entity.decode_cow().expect("should decode hex");
        assert!(matches!(decoded, Cow::Owned(_)), "Hex should be owned");
        assert_eq!(decoded.as_ref(), data);
    }

    #[test]
    fn test_decode_error_display() {
        use crate::hash::HexDecodeError;
        let odd_err = DecodeError::Hex(HexDecodeError::OddLength(5));
        assert!(odd_err.to_string().contains("hex"));
        assert!(odd_err.to_string().contains('5'));

        let invalid_err = DecodeError::Hex(HexDecodeError::InvalidChar { position: 2 });
        assert!(invalid_err.to_string().contains("hex"));
        assert!(invalid_err.to_string().contains('2'));

        let base64_err = DecodeError::Base64("invalid padding".to_string());
        assert!(base64_err.to_string().contains("base64"));
        assert!(base64_err.to_string().contains("invalid padding"));
    }

    #[test]
    fn test_encoding_hex_variant() {
        let hex = Encoding::Hex;
        assert_eq!(hex, Encoding::Hex);
        assert_ne!(hex, Encoding::Base64);
        assert_ne!(hex, Encoding::Utf8);

        let json = serde_json::to_string(&hex).expect("serialize");
        assert_eq!(json, "\"hex\"");
        let parsed: Encoding = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, Encoding::Hex);
    }

    #[test]
    fn test_entity_reference_inline_with_verification() {
        let entity = InlineEntity::text("verified content", "text/plain");
        assert!(entity.verify().expect("verify"));

        let ref_entity = EntityReference::inline(entity);
        assert!(ref_entity.is_inline());
        let hash = ref_entity.content_hash().expect("inline has content_hash");
        assert!(hash.as_str().starts_with("sha256:"));

        let EntityReference::Inline(inline_entity) = &ref_entity else {
            panic!("expected Inline variant")
        };
        assert!(inline_entity.verify().expect("inline verify"));
    }

    #[test]
    fn test_inline_entity_hex_decode_invalid() {
        let entity = InlineEntity {
            content_type: "application/octet-stream".to_string(),
            encoding: Encoding::Hex,
            data: "not-valid-hex!".to_string(),
            hash: ContentHash::new("sha256:placeholder"),
        };
        assert!(entity.decode().is_err());
        assert!(entity.decode_cow().is_err());
    }

    #[test]
    fn entity_reference_json_roundtrip_by_id() {
        let r = EntityReference::by_id(BraidId::from_string("urn:braid:uuid:test"));
        let json = serde_json::to_string(&r).expect("serialize");
        let decoded: EntityReference = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, r);
    }

    #[test]
    fn entity_reference_json_roundtrip_by_ledger_entry() {
        let r = EntityReference::ByLoamEntry {
            spine_id: "spine-42".to_string(),
            entry_hash: ContentHash::new("sha256:ledger_entry_hash"),
        };
        let json = serde_json::to_string(&r).expect("serialize");
        let decoded: EntityReference = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, r);
    }

    #[test]
    fn entity_reference_json_roundtrip_external() {
        let r = EntityReference::external_verified(
            "https://example.com/data",
            ContentHash::new("sha256:ext_hash"),
        );
        let json = serde_json::to_string(&r).expect("serialize");
        let decoded: EntityReference = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, r);
    }

    #[test]
    fn entity_reference_json_roundtrip_external_no_hash() {
        let r = EntityReference::external("https://example.com/no-hash");
        let json = serde_json::to_string(&r).expect("serialize");
        let decoded: EntityReference = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, r);
    }

    #[test]
    fn entity_reference_json_roundtrip_by_hash_typed() {
        let r = EntityReference::by_hash_typed("sha256:typed_hash", "application/json");
        let json = serde_json::to_string(&r).expect("serialize");
        let decoded: EntityReference = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, r);
    }
}
