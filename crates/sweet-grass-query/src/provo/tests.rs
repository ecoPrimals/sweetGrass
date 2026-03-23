// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

#![cfg(test)]
#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "test module: expect/unwrap are standard in tests"
)]

use std::sync::Arc;

use super::*;
use sweet_grass_core::agent::Did;

fn make_test_braid(hash: &str, agent: &str) -> Braid {
    let did = Did::new(agent);
    Braid::builder()
        .data_hash(hash)
        .mime_type("application/json")
        .size(1024)
        .attributed_to(did)
        .build()
        .expect("should build")
}

#[test]
fn test_json_ld_document() {
    let doc = JsonLdDocument::new();
    assert!(doc.context.is_object());
    assert!(doc.graph.is_empty());
}

#[test]
fn test_export_single_braid() {
    let braid = make_test_braid("sha256:test123", "did:key:z6MkTest");
    let exporter = ProvoExport::new();

    let doc = exporter.export_braid(&braid).expect("should export");

    assert_eq!(doc.graph.len(), 1);

    let entity = &doc.graph[0];
    assert_eq!(entity["@type"], "Entity");
    assert_eq!(entity["dataHash"], "sha256:test123");
    assert_eq!(entity["wasAttributedTo"], "did:key:z6MkTest");
}

#[test]
fn test_export_with_derivation() {
    let mut braid = make_test_braid("sha256:derived", "did:key:z6MkTest");
    braid.was_derived_from = vec![
        sweet_grass_core::EntityReference::by_hash("sha256:source1"),
        sweet_grass_core::EntityReference::by_hash("sha256:source2"),
    ];

    let exporter = ProvoExport::new();
    let doc = exporter.export_braid(&braid).expect("should export");

    let entity = &doc.graph[0];
    assert!(entity["wasDerivedFrom"].is_array());
    assert_eq!(entity["wasDerivedFrom"].as_array().unwrap().len(), 2);
}

#[test]
fn test_json_output() {
    let braid = make_test_braid("sha256:test", "did:key:z6MkTest");
    let exporter = ProvoExport::new();
    let doc = exporter.export_braid(&braid).expect("should export");

    let json = doc.to_json().expect("should serialize");
    assert!(json.contains("@context"));
    assert!(json.contains("@graph"));
    assert!(json.contains("prov:Entity"));
}

#[test]
fn test_without_metadata() {
    let mut braid = make_test_braid("sha256:test", "did:key:z6MkTest");
    braid.metadata.title = Some("Test Title".to_string());

    let exporter = ProvoExport::new().include_metadata(false);
    let doc = exporter.export_braid(&braid).expect("should export");

    let entity = &doc.graph[0];
    assert!(entity.get("rdfs:label").is_none());
}

#[test]
fn test_without_ecop() {
    let mut braid = make_test_braid("sha256:test", "did:key:z6MkTest");
    braid.ecop.source_primal = Some(Arc::from("sweetGrass"));

    let exporter = ProvoExport::new().include_ecop(false);
    let doc = exporter.export_braid(&braid).expect("should export");

    let entity = &doc.graph[0];
    assert!(entity.get("sourcePrimal").is_none());
}

#[test]
fn test_timestamp_conversion() {
    let ts = 1_703_203_200_000_000_000_u64; // 2023-12-22 00:00:00 UTC
    let iso = timestamp_to_iso(ts);
    assert!(iso.starts_with("2023-12-22"));
}

#[test]
fn test_timestamp_conversion_zero() {
    let ts = 0_u64;
    let iso = timestamp_to_iso(ts);
    assert!(iso.starts_with("1970-01-01"));
}

#[test]
fn test_json_ld_document_new() {
    let doc = JsonLdDocument::new();
    assert!(doc.graph.is_empty());
    assert!(doc.context["@version"].is_number());
    assert!(doc.context["prov"].is_string());
}

#[test]
fn test_json_ld_document_default() {
    let doc = JsonLdDocument::default();
    assert!(doc.graph.is_empty());
}

#[test]
fn test_json_ld_document_add_node() {
    let mut doc = JsonLdDocument::new();
    doc.add_node(serde_json::json!({"@id": "test"}));
    assert_eq!(doc.graph.len(), 1);

    doc.add_node(serde_json::json!({"@id": "test2"}));
    assert_eq!(doc.graph.len(), 2);
}

#[test]
fn test_json_ld_document_to_json_compact() {
    let braid = make_test_braid("sha256:compact_test", "did:key:z6MkTest");
    let exporter = ProvoExport::new();
    let doc = exporter.export_braid(&braid).expect("should export");

    let compact = doc.to_json_compact().expect("should serialize");
    assert!(!compact.contains('\n'));
    assert!(compact.contains("@context"));
}

#[test]
fn test_provo_export_new() {
    let exporter = ProvoExport::new();
    let braid = make_test_braid("sha256:test", "did:key:z6MkTest");
    let doc = exporter.export_braid(&braid).expect("should export");
    assert!(!doc.graph.is_empty());
}

#[test]
fn test_provo_export_default() {
    let exporter = ProvoExport::default();
    let braid = make_test_braid("sha256:test", "did:key:z6MkTest");
    let doc = exporter.export_braid(&braid).expect("should export");
    assert!(!doc.graph.is_empty());
}

#[test]
fn test_export_with_activity() {
    use sweet_grass_core::Activity;
    use sweet_grass_core::activity::ActivityType;

    let mut braid = make_test_braid("sha256:with_activity", "did:key:z6MkTest");
    braid.was_generated_by = Some(Activity::builder(ActivityType::Creation).build());

    let exporter = ProvoExport::new();
    let doc = exporter.export_braid(&braid).expect("should export");

    assert_eq!(doc.graph.len(), 2);
}

#[test]
fn test_export_with_metadata_title() {
    let mut braid = make_test_braid("sha256:titled", "did:key:z6MkTest");
    braid.metadata.title = Some("Test Document".to_string());

    let exporter = ProvoExport::new().include_metadata(true);
    let doc = exporter.export_braid(&braid).expect("should export");

    let entity = &doc.graph[0];
    assert_eq!(entity["rdfs:label"], "Test Document");
}

#[test]
fn test_export_with_ecop_extensions() {
    let mut braid = make_test_braid("sha256:ecop_test", "did:key:z6MkTest");
    braid.ecop.source_primal = Some(Arc::from("sweetGrass"));
    braid.ecop.niche = Some(Arc::from("attribution"));

    let exporter = ProvoExport::new().include_ecop(true);
    let doc = exporter.export_braid(&braid).expect("should export");

    let entity = &doc.graph[0];
    assert_eq!(entity["sourcePrimal"], "sweetGrass");
    assert_eq!(entity["niche"], "attribution");
}

#[test]
fn test_export_with_both_disabled() {
    let mut braid = make_test_braid("sha256:minimal", "did:key:z6MkTest");
    braid.metadata.title = Some("Should Not Appear".to_string());
    braid.ecop.source_primal = Some(Arc::from("shouldNotAppear"));

    let exporter = ProvoExport::new()
        .include_metadata(false)
        .include_ecop(false);
    let doc = exporter.export_braid(&braid).expect("should export");

    let entity = &doc.graph[0];
    assert!(entity.get("rdfs:label").is_none());
    assert!(entity.get("sourcePrimal").is_none());
}

#[test]
fn test_export_graph_empty() {
    use std::collections::HashMap;

    let graph = crate::traversal::ProvenanceGraph {
        root: sweet_grass_core::EntityReference::by_hash("sha256:root"),
        entities: HashMap::new(),
        activities: HashMap::new(),
        derivation_edges: HashMap::new(),
        generation_edges: HashMap::new(),
        depth: 0,
        truncated: false,
    };
    let exporter = ProvoExport::new();
    let doc = exporter.export_graph(&graph).expect("should export");

    assert!(doc.graph.is_empty());
}

#[test]
fn test_export_preserves_data_hash() {
    let hash = "sha256:preserve_test_hash";
    let braid = make_test_braid(hash, "did:key:z6MkTest");

    let exporter = ProvoExport::new();
    let doc = exporter.export_braid(&braid).expect("should export");

    let entity = &doc.graph[0];
    assert_eq!(entity["dataHash"], hash);
}

#[test]
fn test_export_preserves_mime_type() {
    let mut braid = make_test_braid("sha256:mime_test", "did:key:z6MkTest");
    braid.mime_type = "application/json".into();

    let exporter = ProvoExport::new();
    let doc = exporter.export_braid(&braid).expect("should export");

    let entity = &doc.graph[0];
    assert_eq!(entity["mimeType"], "application/json");
}

#[test]
fn test_export_preserves_size() {
    let mut braid = make_test_braid("sha256:size_test", "did:key:z6MkTest");
    braid.size = 12345;

    let exporter = ProvoExport::new();
    let doc = exporter.export_braid(&braid).expect("should export");

    let entity = &doc.graph[0];
    assert_eq!(entity["size"], 12345);
}

#[test]
fn test_context_contains_required_prefixes() {
    let doc = JsonLdDocument::new();

    assert!(doc.context["prov"].is_string());
    assert!(doc.context["xsd"].is_string());
    assert!(doc.context["rdfs"].is_string());
    assert!(doc.context["schema"].is_string());
    assert!(doc.context["ecop"].is_string());
}

#[test]
fn test_context_contains_prov_classes() {
    let doc = JsonLdDocument::new();

    assert_eq!(doc.context["Entity"], "prov:Entity");
    assert_eq!(doc.context["Activity"], "prov:Activity");
    assert_eq!(doc.context["Agent"], "prov:Agent");
}

#[test]
fn test_provo_export_include_metadata() {
    let exporter = ProvoExport::new().include_metadata(false);
    let braid = make_test_braid("sha256:builder_test", "did:key:z6MkTest");
    let doc = exporter.export_braid(&braid).expect("should export");
    assert!(!doc.graph.is_empty());
}

#[test]
fn test_provo_export_include_ecop() {
    let exporter = ProvoExport::new().include_ecop(false);
    let braid = make_test_braid("sha256:ecop_builder", "did:key:z6MkTest");
    let doc = exporter.export_braid(&braid).expect("should export");
    assert!(!doc.graph.is_empty());
}

#[test]
fn test_provo_export_builder_chain() {
    let exporter = ProvoExport::new()
        .include_metadata(false)
        .include_ecop(true);
    let braid = make_test_braid("sha256:chain", "did:key:z6MkTest");
    let doc = exporter.export_braid(&braid).expect("should export");
    assert_eq!(doc.graph.len(), 1);
}

#[test]
fn test_export_graph_with_entities() {
    use std::collections::HashMap;

    let braid1 = make_test_braid("sha256:entity1", "did:key:z6MkTest");
    let braid2 = make_test_braid("sha256:entity2", "did:key:z6MkTest");

    let mut entities = HashMap::new();
    entities.insert("sha256:entity1".to_string(), braid1);
    entities.insert("sha256:entity2".to_string(), braid2);

    let graph = crate::traversal::ProvenanceGraph {
        root: sweet_grass_core::EntityReference::by_hash("sha256:entity1"),
        entities,
        activities: HashMap::new(),
        derivation_edges: HashMap::new(),
        generation_edges: HashMap::new(),
        depth: 0,
        truncated: false,
    };

    let exporter = ProvoExport::new();
    let doc = exporter.export_graph(&graph).expect("should export");

    assert_eq!(doc.graph.len(), 2);
}

#[test]
fn test_export_graph_with_activities() {
    use std::collections::HashMap;
    use sweet_grass_core::{
        Activity,
        activity::ActivityType,
        agent::{AgentAssociation, AgentRole},
    };

    let activity = Activity::builder(ActivityType::Derivation)
        .associated_with(AgentAssociation::new(
            Did::new("did:key:z6MkAgent"),
            AgentRole::Creator,
        ))
        .compute_units(2.5)
        .started_at(1000)
        .ended_at(2000)
        .build();

    let mut braid = make_test_braid("sha256:with_act", "did:key:z6MkTest");
    braid.was_generated_by = Some(activity.clone());

    let mut entities = HashMap::new();
    entities.insert("sha256:with_act".to_string(), braid);

    let mut activities = HashMap::new();
    activities.insert(activity.id.as_str().to_string(), activity);

    let graph = crate::traversal::ProvenanceGraph {
        root: sweet_grass_core::EntityReference::by_hash("sha256:with_act"),
        entities,
        activities,
        derivation_edges: HashMap::new(),
        generation_edges: HashMap::new(),
        depth: 0,
        truncated: false,
    };

    let exporter = ProvoExport::new();
    let doc = exporter.export_graph(&graph).expect("should export");

    assert_eq!(doc.graph.len(), 2);
}

#[test]
fn test_export_activity_with_associations_and_used() {
    use sweet_grass_core::agent::{AgentAssociation, AgentRole};
    use sweet_grass_core::{
        Activity,
        activity::{ActivityType, UsedEntity},
        entity::EntityReference,
    };

    let activity = Activity::builder(ActivityType::Derivation)
        .associated_with(AgentAssociation::new(
            Did::new("did:key:z6MkAgent1"),
            AgentRole::Creator,
        ))
        .associated_with(AgentAssociation::new(
            Did::new("did:key:z6MkAgent2"),
            AgentRole::Contributor,
        ))
        .uses(UsedEntity::new(EntityReference::by_hash("sha256:input")))
        .compute_units(1.5)
        .started_at(1000)
        .ended_at(5000)
        .build();

    let mut braid = make_test_braid("sha256:full_activity", "did:key:z6MkTest");
    braid.was_generated_by = Some(activity);

    let exporter = ProvoExport::new();
    let doc = exporter.export_braid(&braid).expect("should export");

    assert_eq!(doc.graph.len(), 2);
    let activity_node = doc
        .graph
        .iter()
        .find(|n| n["@type"] == "Activity")
        .expect("activity node");
    assert!(activity_node["wasAssociatedWith"].is_array());
    assert_eq!(
        activity_node["wasAssociatedWith"].as_array().unwrap().len(),
        2
    );
    assert!(activity_node["used"].is_array());
    assert_eq!(activity_node["computeUnits"], 1.5);
    assert!(activity_node.get("endedAtTime").is_some());
}

#[test]
fn test_json_ld_document_serialization_roundtrip() {
    let mut doc = JsonLdDocument::new();
    doc.add_node(serde_json::json!({
        "@id": "test:entity1",
        "@type": "Entity",
        "dataHash": "sha256:abc123"
    }));
    doc.add_node(serde_json::json!({
        "@id": "test:entity2",
        "@type": "Entity",
        "dataHash": "sha256:def456"
    }));

    let json = doc.to_json().expect("serialize");
    let parsed: JsonLdDocument = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(parsed.graph.len(), 2);
    assert!(parsed.context.is_object());
}

#[test]
fn test_timestamp_conversion_far_future() {
    let ts = 18_000_000_000_000_000_000_u64;
    let iso = timestamp_to_iso(ts);
    assert!(iso.contains("2540"));
}

#[test]
fn test_to_json_success() {
    let doc = JsonLdDocument::new();
    let result = doc.to_json();
    assert!(result.is_ok());
}

#[test]
fn test_to_json_compact_success() {
    let doc = JsonLdDocument::new();
    let result = doc.to_json_compact();
    assert!(result.is_ok());
}

#[test]
fn test_export_with_metadata_description() {
    let mut braid = make_test_braid("sha256:desc_test", "did:key:z6MkTest");
    braid.metadata.description = Some("A test description".to_string());

    let exporter = ProvoExport::new().include_metadata(true);
    let doc = exporter.export_braid(&braid).expect("should export");

    let entity = &doc.graph[0];
    assert_eq!(entity["rdfs:comment"], "A test description");
}

#[test]
fn test_export_derivation_with_external_ref() {
    let mut braid = make_test_braid("sha256:derived", "did:key:z6MkTest");
    braid.was_derived_from = vec![
        sweet_grass_core::EntityReference::by_hash("sha256:has_hash"),
        sweet_grass_core::EntityReference::external("https://example.com/no-hash"),
    ];

    let exporter = ProvoExport::new();
    let doc = exporter.export_braid(&braid).expect("should export");

    let entity = &doc.graph[0];
    let derived = entity["wasDerivedFrom"].as_array().unwrap();
    assert_eq!(derived.len(), 1);
    assert_eq!(derived[0], "sha256:has_hash");
}
