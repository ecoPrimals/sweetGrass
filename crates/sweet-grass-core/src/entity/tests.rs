// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Tests for entity reference types, serialization, and inline data.

#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "test module: unwrap/expect are standard in tests"
)]

use super::*;

// ── Bincode roundtrip ───────────────────────────────────────────────

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

// ── Constructor and accessor coverage ───────────────────────────────

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

// ── InlineEntity ────────────────────────────────────────────────────

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

// ── Serialization roundtrips ────────────────────────────────────────

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
fn test_inline_entity_serialization() {
    let entity = InlineEntity::text("test", "text/plain");
    let ref_entity = EntityReference::inline(entity);

    let json = serde_json::to_string(&ref_entity).expect("should serialize");
    let parsed: EntityReference = serde_json::from_str(&json).expect("should deserialize");

    assert!(parsed.is_inline());
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

// ── Error display ───────────────────────────────────────────────────

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
