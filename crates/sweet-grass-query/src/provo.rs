//! PROV-O JSON-LD export.
//!
//! This module provides W3C PROV-O compliant export of provenance data.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sweet_grass_core::{Activity, Braid};

use crate::traversal::ProvenanceGraph;
use crate::Result;

/// JSON-LD document representation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JsonLdDocument {
    /// JSON-LD context.
    #[serde(rename = "@context")]
    pub context: Value,

    /// Document graph.
    #[serde(rename = "@graph")]
    pub graph: Vec<Value>,
}

impl JsonLdDocument {
    /// Create a new JSON-LD document with PROV-O context.
    #[must_use]
    pub fn new() -> Self {
        Self {
            context: Self::prov_o_context(),
            graph: Vec::new(),
        }
    }

    /// Get the standard PROV-O context.
    fn prov_o_context() -> Value {
        json!({
            "@version": 1.1,
            "prov": "http://www.w3.org/ns/prov#",
            "xsd": "http://www.w3.org/2001/XMLSchema#",
            "rdfs": "http://www.w3.org/2000/01/rdf-schema#",
            "schema": "http://schema.org/",
            "ecop": "https://ecoprimals.io/vocab#",

            // PROV-O Classes
            "Entity": "prov:Entity",
            "Activity": "prov:Activity",
            "Agent": "prov:Agent",

            // PROV-O Properties
            "wasGeneratedBy": {"@id": "prov:wasGeneratedBy", "@type": "@id"},
            "wasDerivedFrom": {"@id": "prov:wasDerivedFrom", "@type": "@id"},
            "wasAttributedTo": {"@id": "prov:wasAttributedTo", "@type": "@id"},
            "wasAssociatedWith": {"@id": "prov:wasAssociatedWith", "@type": "@id"},
            "used": {"@id": "prov:used", "@type": "@id"},
            "startedAtTime": {"@id": "prov:startedAtTime", "@type": "xsd:dateTime"},
            "endedAtTime": {"@id": "prov:endedAtTime", "@type": "xsd:dateTime"},
            "generatedAtTime": {"@id": "prov:generatedAtTime", "@type": "xsd:dateTime"},
            "hadRole": {"@id": "prov:hadRole", "@type": "@id"},
            "actedOnBehalfOf": {"@id": "prov:actedOnBehalfOf", "@type": "@id"},

            // ecoPrimals extensions
            "dataHash": "ecop:dataHash",
            "mimeType": "ecop:mimeType",
            "size": "ecop:size",
            "computeUnits": "ecop:computeUnits",
            "sourcePrimal": "ecop:sourcePrimal",
            "niche": "ecop:niche"
        })
    }

    /// Add a node to the graph.
    pub fn add_node(&mut self, node: Value) {
        self.graph.push(node);
    }

    /// Convert to JSON string.
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Convert to compact JSON string.
    pub fn to_json_compact(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}

impl Default for JsonLdDocument {
    fn default() -> Self {
        Self::new()
    }
}

/// Exporter for PROV-O format.
pub struct ProvoExport {
    include_metadata: bool,
    include_ecop: bool,
}

impl ProvoExport {
    /// Create a new exporter.
    #[must_use]
    pub fn new() -> Self {
        Self {
            include_metadata: true,
            include_ecop: true,
        }
    }

    /// Set whether to include metadata.
    #[must_use]
    pub fn include_metadata(mut self, include: bool) -> Self {
        self.include_metadata = include;
        self
    }

    /// Set whether to include ecoPrimals extensions.
    #[must_use]
    pub fn include_ecop(mut self, include: bool) -> Self {
        self.include_ecop = include;
        self
    }

    /// Export a single Braid as PROV-O JSON-LD.
    pub fn export_braid(&self, braid: &Braid) -> Result<JsonLdDocument> {
        let mut doc = JsonLdDocument::new();

        // Add Entity node
        doc.add_node(self.braid_to_entity(braid));

        // Add Activity node if present
        if let Some(activity) = &braid.was_generated_by {
            doc.add_node(self.activity_to_prov(activity));
        }

        Ok(doc)
    }

    /// Export a provenance graph as PROV-O JSON-LD.
    pub fn export_graph(&self, graph: &ProvenanceGraph) -> Result<JsonLdDocument> {
        let mut doc = JsonLdDocument::new();

        // Add all entities
        for braid in graph.entities.values() {
            doc.add_node(self.braid_to_entity(braid));
        }

        // Add all activities
        for activity in graph.activities.values() {
            doc.add_node(self.activity_to_prov(activity));
        }

        Ok(doc)
    }

    /// Convert a Braid to a PROV-O Entity node.
    fn braid_to_entity(&self, braid: &Braid) -> Value {
        let mut entity = IndexMap::new();

        // Core PROV properties
        entity.insert("@id".to_string(), json!(braid.id.as_str()));
        entity.insert("@type".to_string(), json!("Entity"));
        entity.insert("dataHash".to_string(), json!(braid.data_hash));
        entity.insert("mimeType".to_string(), json!(braid.mime_type));
        entity.insert("size".to_string(), json!(braid.size));

        // Attribution
        entity.insert(
            "wasAttributedTo".to_string(),
            json!(braid.was_attributed_to.as_str()),
        );

        // Generation time
        entity.insert(
            "generatedAtTime".to_string(),
            json!(timestamp_to_iso(braid.generated_at_time)),
        );

        // Generation activity
        if let Some(activity) = &braid.was_generated_by {
            entity.insert("wasGeneratedBy".to_string(), json!(activity.id.as_str()));
        }

        // Derivation
        if !braid.was_derived_from.is_empty() {
            let derived: Vec<Value> = braid
                .was_derived_from
                .iter()
                .filter_map(|e| e.content_hash().map(|h| json!(h)))
                .collect();
            if !derived.is_empty() {
                entity.insert("wasDerivedFrom".to_string(), json!(derived));
            }
        }

        // Metadata
        if self.include_metadata {
            if let Some(title) = &braid.metadata.title {
                entity.insert("rdfs:label".to_string(), json!(title));
            }
            if let Some(desc) = &braid.metadata.description {
                entity.insert("rdfs:comment".to_string(), json!(desc));
            }
        }

        // ecoPrimals extensions
        if self.include_ecop {
            if let Some(primal) = &braid.ecop.source_primal {
                entity.insert("sourcePrimal".to_string(), json!(primal));
            }
            if let Some(niche) = &braid.ecop.niche {
                entity.insert("niche".to_string(), json!(niche));
            }
        }

        json!(entity)
    }

    /// Convert an Activity to a PROV-O Activity node.
    fn activity_to_prov(&self, activity: &Activity) -> Value {
        let mut act = IndexMap::new();

        // Core PROV properties
        act.insert("@id".to_string(), json!(activity.id.as_str()));
        act.insert("@type".to_string(), json!("Activity"));

        // Activity type as additional type
        act.insert(
            "ecop:activityType".to_string(),
            json!(activity.activity_type.to_string()),
        );

        // Time bounds
        act.insert(
            "startedAtTime".to_string(),
            json!(timestamp_to_iso(activity.started_at_time)),
        );
        if let Some(end) = activity.ended_at_time {
            act.insert("endedAtTime".to_string(), json!(timestamp_to_iso(end)));
        }

        // Associations
        if !activity.was_associated_with.is_empty() {
            let agents: Vec<Value> = activity
                .was_associated_with
                .iter()
                .map(|a| {
                    json!({
                        "prov:agent": a.agent.as_str(),
                        "prov:hadRole": format!("ecop:{:?}", a.role)
                    })
                })
                .collect();
            act.insert("wasAssociatedWith".to_string(), json!(agents));
        }

        // Used entities
        if !activity.used.is_empty() {
            let used: Vec<Value> = activity
                .used
                .iter()
                .filter_map(|u| u.entity.content_hash().map(|h| json!(h)))
                .collect();
            if !used.is_empty() {
                act.insert("used".to_string(), json!(used));
            }
        }

        // ecoPrimals extensions
        if self.include_ecop {
            if let Some(compute) = activity.ecop.compute_units {
                act.insert("computeUnits".to_string(), json!(compute));
            }
        }

        json!(act)
    }
}

impl Default for ProvoExport {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert nanosecond timestamp to ISO 8601 string.
fn timestamp_to_iso(nanos: u64) -> String {
    use chrono::{TimeZone, Utc};

    #[allow(clippy::cast_possible_wrap)]
    let secs = (nanos / 1_000_000_000) as i64;
    let nsecs = (nanos % 1_000_000_000) as u32;

    match Utc.timestamp_opt(secs, nsecs) {
        chrono::LocalResult::Single(dt) => dt.to_rfc3339(),
        _ => format!("{nanos}"),
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::expect_used, clippy::unwrap_used)]
mod tests {
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
        braid.ecop.source_primal = Some("sweetGrass".to_string());

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
        // Defaults should be true
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
        use sweet_grass_core::activity::ActivityType;
        use sweet_grass_core::Activity;

        let mut braid = make_test_braid("sha256:with_activity", "did:key:z6MkTest");
        braid.was_generated_by = Some(Activity::builder(ActivityType::Creation).build());

        let exporter = ProvoExport::new();
        let doc = exporter.export_braid(&braid).expect("should export");

        // Should have entity and activity nodes
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
        braid.ecop.source_primal = Some("sweetGrass".to_string());
        braid.ecop.niche = Some("attribution".to_string());

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
        braid.ecop.source_primal = Some("shouldNotAppear".to_string());

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
        braid.mime_type = "application/json".to_string();

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
}
