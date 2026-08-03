// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Unit tests for braid core types (`ContentHash`, `BraidId`, `BraidType`,
//! `BraidMetadata`, `CertificateRef`).

#![expect(clippy::expect_used, clippy::unwrap_used, reason = "test module")]

use std::borrow::Borrow;
use std::sync::Arc;

use super::{
    BraidId, BraidMetadata, BraidType, ContentHash, CrossGateAttribution, CrossGateTrustEvent,
    SummaryType, Timestamp,
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
fn braid_metadata_bincode_roundtrip_with_cross_gate() {
    let meta = BraidMetadata {
        cross_gate: Some(CrossGateAttribution {
            origin_gate: Arc::from("strandGate"),
            target_gate: Arc::from("ironGate"),
            trust_event: CrossGateTrustEvent::KeyExchange,
            origin_agent: Did::new("did:key:z6MkOrigin"),
            target_agent: Some(Did::new("did:key:z6MkTarget")),
            family_id: Some("family-42".to_string()),
        }),
        ..Default::default()
    };

    let bytes = bincode::serialize(&meta).expect("serialize");
    let decoded: BraidMetadata = bincode::deserialize(&bytes).expect("deserialize");
    assert_eq!(decoded.cross_gate, meta.cross_gate);
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
            start: Timestamp::new(100),
            end: Timestamp::new(999),
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

#[test]
fn braid_id_to_uuid_returns_embedded_for_uuid_ids() {
    let id = BraidId::new();
    let extracted = id.extract_uuid().unwrap();
    assert_eq!(id.to_uuid(), extracted);
}

#[test]
fn braid_id_to_uuid_derives_deterministic_v5_for_hash_ids() {
    let hash_id = BraidId::from_hash(&ContentHash::new("sha256:abc123"));
    let uuid1 = hash_id.to_uuid();
    let uuid2 = hash_id.to_uuid();
    assert_eq!(uuid1, uuid2, "derivation must be deterministic");
    assert_eq!(uuid1.get_version_num(), 5);
}

#[test]
fn braid_id_to_uuid_different_hashes_produce_different_uuids() {
    let id_a = BraidId::from_hash(&ContentHash::new("sha256:aaa"));
    let id_b = BraidId::from_hash(&ContentHash::new("sha256:bbb"));
    assert_ne!(id_a.to_uuid(), id_b.to_uuid());
}
