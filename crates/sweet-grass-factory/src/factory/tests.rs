// SPDX-License-Identifier: AGPL-3.0-only
//! BraidFactory tests.

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::expect_used, clippy::unwrap_used)]
mod factory_tests {
    use super::super::*;
    use sweet_grass_core::test_fixtures::TEST_SOURCE_PRIMAL;
    use sweet_grass_core::{
        agent::AgentRole,
        braid::{BraidId, ContentHash, SummaryType},
        contribution::{ContributionRecord, SessionContribution},
        entity::EntityReference,
        ActivityType,
    };

    fn make_factory() -> BraidFactory {
        BraidFactory::new(sweet_grass_core::agent::Did::new("did:key:z6MkTestFactory"))
    }

    #[test]
    fn test_from_data() {
        let factory = make_factory();
        let data = b"Hello, World!";

        let braid = factory
            .from_data(data, "text/plain", None)
            .expect("should create");

        assert!(braid.data_hash.as_str().starts_with("sha256:"));
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
            sweet_grass_core::braid::BraidType::Collection { member_count, .. } => {
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
            sweet_grass_core::braid::BraidType::Collection {
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
            .from_loam_entry(&LoamEntryParams {
                spine_id: "spine-1".to_string(),
                entry_hash: ContentHash::new("sha256:entry123"),
                index: 42,
                data_hash: ContentHash::new("sha256:data456"),
                mime_type: "application/json".to_string(),
                size: 1024,
                metadata: None,
            })
            .expect("should create");

        assert!(braid.ecop.loam_commit.is_some());
        let commit = braid.ecop.loam_commit.unwrap();
        assert_eq!(commit.spine_id, "spine-1");
        assert_eq!(commit.index, 42);
    }

    #[test]
    fn test_certificate_mint() {
        let factory = make_factory();
        let recipient = sweet_grass_core::agent::Did::new("did:key:z6MkRecipient");

        let braid = factory
            .certificate_mint(
                "cert-001",
                ContentHash::new("sha256:certdata"),
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
        let factory = make_factory().with_source_primal(TEST_SOURCE_PRIMAL);

        let braid = factory
            .from_data(b"test", "text/plain", None)
            .expect("should create");

        assert_eq!(
            braid.ecop.source_primal,
            Some(TEST_SOURCE_PRIMAL.to_string())
        );
    }

    #[test]
    fn test_from_self_knowledge() {
        use sweet_grass_core::primal_info::SelfKnowledge;

        let self_knowledge = SelfKnowledge {
            name: "test-primal".to_string(),
            ..Default::default()
        };

        let factory = BraidFactory::from_self_knowledge(
            sweet_grass_core::agent::Did::new("did:key:z6MkTest"),
            &self_knowledge,
        );

        let braid = factory
            .from_data(b"test", "text/plain", None)
            .expect("should create");

        assert_eq!(braid.ecop.source_primal, Some("test-primal".to_string()));
    }

    #[test]
    fn test_from_contribution_creates_valid_braid() {
        let factory = make_factory();
        let record = ContributionRecord {
            agent: sweet_grass_core::agent::Did::new("did:key:z6MkContributor"),
            role: AgentRole::Creator,
            content_hash: "sha256:contrib123".to_string(),
            mime_type: "application/json".to_string(),
            size: 256,
            timestamp: 1_000_000_000,
            description: Some("Test contribution".to_string()),
            source_primal: Some(TEST_SOURCE_PRIMAL.to_string()),
            session_id: Some("session-xyz".to_string()),
            domain: std::collections::HashMap::new(),
        };

        let braid = factory.from_contribution(&record).expect("should create");

        assert_eq!(braid.data_hash.as_str(), "sha256:contrib123");
        assert_eq!(braid.was_attributed_to.as_str(), "did:key:z6MkContributor");
        assert_eq!(braid.generated_at_time, 1_000_000_000);
        assert_eq!(
            braid.ecop.source_primal,
            Some(TEST_SOURCE_PRIMAL.to_string())
        );
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
            source_primal: TEST_SOURCE_PRIMAL.to_string(),
            niche: Some("chemistry".to_string()),
            contributions: vec![
                ContributionRecord {
                    agent: sweet_grass_core::agent::Did::new("did:key:z6MkAgent1"),
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
                    agent: sweet_grass_core::agent::Did::new("did:key:z6MkAgent2"),
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
        assert_eq!(braids[0].data_hash.as_str(), "sha256:hash1");
        assert_eq!(braids[1].data_hash.as_str(), "sha256:hash2");
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
            agent: sweet_grass_core::agent::Did::new("did:key:z6MkChemist"),
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

    #[test]
    fn test_from_session_with_loam_entry() {
        let factory = make_factory();
        let session = SessionContribution {
            session_id: "session-loam".to_string(),
            source_primal: TEST_SOURCE_PRIMAL.to_string(),
            niche: None,
            contributions: vec![ContributionRecord {
                agent: sweet_grass_core::agent::Did::new("did:key:z6MkLoam"),
                role: AgentRole::Creator,
                content_hash: "sha256:loamhash".to_string(),
                mime_type: "text/plain".to_string(),
                size: 50,
                timestamp: 0,
                description: None,
                source_primal: None,
                session_id: None,
                domain: std::collections::HashMap::new(),
            }],
            session_start: None,
            session_end: None,
            loam_entry: Some("main|sha256:entry123|7".to_string()),
            domain: std::collections::HashMap::new(),
        };

        let braids = factory.from_session(&session).expect("should create");
        assert_eq!(braids.len(), 1);
        let loam = braids[0]
            .ecop
            .loam_commit
            .as_ref()
            .expect("loam_commit set");
        assert_eq!(loam.spine_id, "main");
        assert_eq!(loam.entry_hash.as_str(), "sha256:entry123");
        assert_eq!(loam.index, 7);
    }

    #[test]
    fn test_from_session_with_invalid_loam_entry() {
        let factory = make_factory();
        let session = SessionContribution {
            session_id: "session-bad-loam".to_string(),
            source_primal: TEST_SOURCE_PRIMAL.to_string(),
            niche: None,
            contributions: vec![ContributionRecord {
                agent: sweet_grass_core::agent::Did::new("did:key:z6MkBadLoam"),
                role: AgentRole::Creator,
                content_hash: "sha256:badloam".to_string(),
                mime_type: "text/plain".to_string(),
                size: 10,
                timestamp: 0,
                description: None,
                source_primal: None,
                session_id: None,
                domain: std::collections::HashMap::new(),
            }],
            session_start: None,
            session_end: None,
            loam_entry: Some("only-two|parts".to_string()),
            domain: std::collections::HashMap::new(),
        };

        let braids = factory.from_session(&session).expect("should create");
        assert_eq!(braids.len(), 1);
        assert!(braids[0].ecop.loam_commit.is_none());
    }
}
