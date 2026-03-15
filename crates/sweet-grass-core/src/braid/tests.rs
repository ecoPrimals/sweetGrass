// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Braid module tests.

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "test module: expect/unwrap are standard in tests"
)]
mod unit_tests {
    use super::super::*;
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
        let hash = ContentHash::new("sha256:abc123");
        let id = BraidId::from_hash(&hash);
        assert_eq!(id.as_str(), "urn:braid:sha256:abc123");
    }

    #[test]
    fn test_braid_id_extract_uuid() {
        let id = BraidId::new();
        let uuid = id.extract_uuid();
        assert!(
            uuid.is_some(),
            "random BraidId should have extractable UUID"
        );

        let hash_id = BraidId::from_hash(&ContentHash::new("sha256:abc123"));
        assert!(
            hash_id.extract_uuid().is_none(),
            "hash-based BraidId should not extract as UUID"
        );
    }

    #[test]
    fn test_content_hash_to_bytes32() {
        let hex_64 = "a".repeat(64);
        let hash = ContentHash::new(format!("sha256:{hex_64}"));
        let bytes = hash.to_bytes32();
        assert!(bytes.is_some());
        assert_eq!(bytes.unwrap(), [0xaa; 32]);

        let short = ContentHash::new("sha256:abcd");
        assert!(short.to_bytes32().is_none(), "too short should be None");

        let no_prefix = ContentHash::new("nocolon");
        assert!(no_prefix.to_bytes32().is_none());
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

        assert_eq!(braid.data_hash.as_str(), "sha256:abc123");
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
#[expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "test module: expect/unwrap are standard in tests"
)]
mod proptests {
    use super::super::*;
    use crate::agent::Did;
    use crate::hash::{hex_decode, hex_encode};
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
            let ch = ContentHash::new(hash);
            let id1 = BraidId::from_hash(&ch);
            let id2 = BraidId::from_hash(&ch);
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
            let ch = ContentHash::new(hash);
            let id = BraidId::from_hash(&ch);
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

            prop_assert_eq!(braid.data_hash.as_str(), hash.as_str());
        }

        /// BraidId roundtrip: any non-empty string -> BraidId -> as_str matches original
        #[test]
        fn prop_braid_id_roundtrip(s in "[a-zA-Z0-9_-]{1,64}") {
            let id = BraidId::from_string(&s);
            prop_assert_eq!(id.as_str(), s.as_str());
        }

        /// ContentHash roundtrip: any non-empty string -> ContentHash -> as_str matches original
        #[test]
        fn prop_content_hash_roundtrip(s in "[a-zA-Z0-9:._-]{1,128}") {
            let hash = ContentHash::new(&s);
            prop_assert_eq!(hash.as_str(), s.as_str());
        }

        /// Did roundtrip: any string starting with "did:" -> Did -> as_str matches original
        #[test]
        fn prop_did_roundtrip(s in "did:[a-zA-Z0-9:._-]{1,100}") {
            let did = Did::new(&s);
            prop_assert_eq!(did.as_str(), s.as_str());
        }

        /// Hex encode-decode roundtrip: any Vec<u8> -> hex_encode -> hex_decode == original
        #[test]
        fn prop_hex_encode_decode_roundtrip(bytes in proptest::collection::vec(any::<u8>(), 0..256)) {
            let encoded = hex_encode(&bytes);
            let decoded = hex_decode(&encoded);
            prop_assert_eq!(decoded, Some(bytes));
        }

        /// Braid builder always produces valid braid with correct fields
        #[test]
        fn prop_braid_builder_correct_fields(
            hash in "[a-f0-9]{64}",
            mime in "(application|text)/(json|plain|octet-stream)",
            size in 0u64..10_000_000u64,
            agent in "did:key:z6Mk[a-zA-Z0-9]{10,44}",
        ) {
            let data_hash = format!("sha256:{hash}");
            let did = Did::new(&agent);
            let braid = Braid::builder()
                .data_hash(&data_hash)
                .mime_type(&mime)
                .size(size)
                .attributed_to(did)
                .build()
                .expect("should build");

            prop_assert_eq!(braid.data_hash.as_str(), data_hash.as_str());
            prop_assert_eq!(braid.mime_type, mime);
            prop_assert_eq!(braid.size, size);
            prop_assert_eq!(braid.was_attributed_to.as_str(), agent.as_str());
        }

        /// BraidId clone produces eq value (Arc<str> invariant)
        #[test]
        fn prop_braid_id_clone_eq(s in "[a-zA-Z0-9_-]{1,64}") {
            let id = BraidId::from_string(&s);
            prop_assert_eq!(id.clone(), id);
        }

        /// ContentHash clone produces eq value (Arc<str> invariant)
        #[test]
        fn prop_content_hash_clone_eq(s in "[a-zA-Z0-9:._-]{1,128}") {
            let hash = ContentHash::new(&s);
            prop_assert_eq!(hash.clone(), hash);
        }

        /// Did clone produces eq value (Arc<str> invariant)
        #[test]
        fn prop_did_clone_eq(s in "did:[a-zA-Z0-9:._-]{1,100}") {
            let did = Did::new(&s);
            prop_assert_eq!(did.clone(), did);
        }
    }

    #[test]
    fn test_braid_builder_generated_by() {
        use crate::activity::{Activity, ActivityType};
        use crate::agent::{AgentAssociation, AgentRole};

        let did = Did::new("did:key:z6MkBuilderGen");
        let activity = Activity::builder(ActivityType::Derivation)
            .associated_with(AgentAssociation::new(did.clone(), AgentRole::Creator))
            .build();

        let braid = Braid::builder()
            .data_hash("sha256:gen-by-test")
            .mime_type("text/plain")
            .size(42)
            .attributed_to(did)
            .generated_by(activity)
            .build()
            .expect("should build");

        assert!(braid.was_generated_by.is_some());
    }

    #[test]
    fn test_braid_builder_derived_from() {
        let did = Did::new("did:key:z6MkBuilderDeriv");
        let entity = EntityReference::by_hash("sha256:parent-input");
        let braid = Braid::builder()
            .data_hash("sha256:derived-test")
            .mime_type("text/plain")
            .size(10)
            .attributed_to(did)
            .derived_from(entity)
            .build()
            .expect("should build");

        assert_eq!(braid.was_derived_from.len(), 1);
    }

    #[test]
    fn test_braid_builder_ecop() {
        let did = Did::new("did:key:z6MkBuilderEcop");
        let ecop = EcoPrimalsAttributes {
            source_primal: Some("test-primal".to_string()),
            ..EcoPrimalsAttributes::default()
        };
        let braid = Braid::builder()
            .data_hash("sha256:ecop-test")
            .mime_type("text/plain")
            .size(0)
            .attributed_to(did)
            .ecop(ecop)
            .build()
            .expect("should build");

        assert_eq!(braid.ecop.source_primal, Some("test-primal".to_string()));
    }

    #[test]
    fn test_braid_builder_metadata() {
        let did = Did::new("did:key:z6MkBuilderMeta");
        let meta = BraidMetadata {
            tags: vec!["important".to_string()],
            ..BraidMetadata::default()
        };
        let braid = Braid::builder()
            .data_hash("sha256:meta-test")
            .mime_type("text/plain")
            .size(0)
            .attributed_to(did)
            .metadata(meta)
            .build()
            .expect("should build");

        assert_eq!(braid.metadata.tags, vec!["important"]);
    }

    #[test]
    fn test_braid_builder_braid_type() {
        use crate::braid::BraidType;

        let did = Did::new("did:key:z6MkBuilderType");

        let entity = Braid::builder()
            .data_hash("sha256:entity")
            .mime_type("text/plain")
            .size(0)
            .attributed_to(did.clone())
            .braid_type(BraidType::Entity)
            .build()
            .expect("should build");
        assert!(matches!(entity.braid_type, BraidType::Entity));

        let activity = Braid::builder()
            .data_hash("sha256:activity")
            .mime_type("text/plain")
            .size(0)
            .attributed_to(did.clone())
            .braid_type(BraidType::Activity)
            .build()
            .expect("should build");
        assert!(matches!(activity.braid_type, BraidType::Activity));

        let agent = Braid::builder()
            .data_hash("sha256:agent")
            .mime_type("text/plain")
            .size(0)
            .attributed_to(did)
            .braid_type(BraidType::Agent)
            .build()
            .expect("should build");
        assert!(matches!(agent.braid_type, BraidType::Agent));
    }

    #[test]
    fn test_braid_builder_derived_from_multiple() {
        let did = Did::new("did:key:z6MkBuilderDeriv");
        let e1 = EntityReference::by_hash("sha256:parent1");
        let e2 = EntityReference::by_hash("sha256:parent2");

        let braid = Braid::builder()
            .data_hash("sha256:derived-multi")
            .mime_type("text/plain")
            .size(10)
            .attributed_to(did)
            .derived_from(e1)
            .derived_from(e2)
            .build()
            .expect("should build");

        assert_eq!(braid.was_derived_from.len(), 2);
    }

    #[test]
    fn test_braid_builder_missing_data_hash() {
        let did = Did::new("did:key:z6MkBuilder");
        let result = Braid::builder()
            .mime_type("text/plain")
            .size(100)
            .attributed_to(did)
            .build();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("data_hash"));
    }

    #[test]
    fn test_braid_builder_missing_mime_type() {
        let did = Did::new("did:key:z6MkBuilder");
        let result = Braid::builder()
            .data_hash("sha256:abc")
            .size(100)
            .attributed_to(did)
            .build();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("mime_type"));
    }

    #[test]
    fn test_braid_builder_missing_size() {
        let did = Did::new("did:key:z6MkBuilder");
        let result = Braid::builder()
            .data_hash("sha256:abc")
            .mime_type("text/plain")
            .attributed_to(did)
            .build();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("size"));
    }

    #[test]
    fn test_braid_builder_missing_attributed_to() {
        let result = Braid::builder()
            .data_hash("sha256:abc")
            .mime_type("text/plain")
            .size(100)
            .build();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("was_attributed_to"));
    }
}
