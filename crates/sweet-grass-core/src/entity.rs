// SPDX-License-Identifier: AGPL-3.0-only
//! Entity reference data structures - links to data artifacts.
//!
//! Entity references allow Braids to link to other data without
//! embedding it directly.

use serde::{Deserialize, Serialize};

use crate::braid::{BraidId, ContentHash};

/// Reference to a PROV entity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
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

    /// Create a reference by anchoring provider entry.
    #[must_use]
    pub fn by_loam_entry(spine_id: impl Into<String>, entry_hash: impl Into<ContentHash>) -> Self {
        Self::ByLoamEntry {
            spine_id: spine_id.into(),
            entry_hash: entry_hash.into(),
        }
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

/// Encoding for inline entity data.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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
            Encoding::Hex => hex_decode_strict(&self.data).map_err(DecodeError::Hex),
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
                let decoded = hex_decode_strict(&self.data).map_err(DecodeError::Hex)?;
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
pub enum DecodeError {
    /// Base64 decoding error.
    #[error("base64 decode error: {0}")]
    Base64(String),

    /// Hex decoding error.
    #[error("hex decode error: {0}")]
    Hex(String),
}

use crate::hash::{hex_decode_strict, sha256 as compute_sha256};

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "test module: expect is standard in tests"
)]
mod tests {
    use super::*;

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
}
