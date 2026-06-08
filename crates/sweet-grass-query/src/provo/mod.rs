// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! PROV-O JSON-LD export.
//!
//! This module provides W3C PROV-O compliant export of provenance data.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sweet_grass_core::braid::types::{
    PROV_VOCAB_URI, RDFS_VOCAB_URI, SCHEMA_VOCAB_URI, XSD_VOCAB_URI, ecop_vocab_uri,
};
use sweet_grass_core::braid::{BraidId, BraidType};
use sweet_grass_core::entity::EntityReference;
use sweet_grass_core::{Activity, Braid};

use crate::Result;
use crate::traversal::ProvenanceGraph;

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
            context: Self::prov_o_context_with(&ecop_vocab_uri()),
            graph: Vec::new(),
        }
    }

    /// Create with a pre-resolved ecoPrimals vocabulary URI (avoids `env::var`).
    #[must_use]
    pub fn with_ecop_vocab(ecop_ns: &str) -> Self {
        Self {
            context: Self::prov_o_context_with(ecop_ns),
            graph: Vec::new(),
        }
    }

    /// Build the standard PROV-O context with a given ecoPrimals namespace.
    fn prov_o_context_with(ecop_ns: &str) -> Value {
        let ecop_ns = ecop_ns.to_string();

        json!({
            "@version": 1.1,
            "prov": PROV_VOCAB_URI,
            "xsd": XSD_VOCAB_URI,
            "rdfs": RDFS_VOCAB_URI,
            "schema": SCHEMA_VOCAB_URI,
            "ecop": ecop_ns,

            "Entity": "prov:Entity",
            "Activity": "prov:Activity",
            "Agent": "prov:Agent",
            "Collection": "prov:Collection",

            "wasGeneratedBy": {"@id": "prov:wasGeneratedBy", "@type": "@id"},
            "wasDerivedFrom": {"@id": "prov:wasDerivedFrom", "@type": "@id"},
            "wasAttributedTo": {"@id": "prov:wasAttributedTo", "@type": "@id"},
            "wasAssociatedWith": {"@id": "prov:wasAssociatedWith", "@type": "@id"},
            "used": {"@id": "prov:used", "@type": "@id"},
            "startedAtTime": {"@id": "prov:startedAtTime", "@type": "xsd:dateTime"},
            "endedAtTime": {"@id": "prov:endedAtTime", "@type": "xsd:dateTime"},
            "generatedAtTime": {"@id": "prov:generatedAtTime", "@type": "xsd:dateTime"},
            "invalidatedAtTime": {"@id": "prov:invalidatedAtTime", "@type": "xsd:dateTime"},
            "alternateOf": {"@id": "prov:alternateOf", "@type": "@id"},
            "hadRole": {"@id": "prov:hadRole", "@type": "@id"},
            "actedOnBehalfOf": {"@id": "prov:actedOnBehalfOf", "@type": "@id"},

            "dataHash": "ecop:dataHash",
            "mimeType": "ecop:mimeType",
            "size": "ecop:size",
            "computeUnits": "ecop:computeUnits",
            "sourcePrimal": "ecop:sourcePrimal",
            "sourceGate": "ecop:sourceGate",
            "niche": "ecop:niche",

            "crossGateAttribution": "ecop:crossGateAttribution",
            "originGate": "ecop:originGate",
            "targetGate": "ecop:targetGate",
            "trustEvent": "ecop:trustEvent",
            "originAgent": {"@id": "ecop:originAgent", "@type": "@id"},
            "targetAgent": {"@id": "ecop:targetAgent", "@type": "@id"},
            "familyId": "ecop:familyId"
        })
    }

    /// Add a node to the graph.
    pub fn add_node(&mut self, node: Value) {
        self.graph.push(node);
    }

    /// Convert to JSON string.
    ///
    /// # Errors
    ///
    /// Returns an error if JSON serialization fails.
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Convert to compact JSON string.
    ///
    /// # Errors
    ///
    /// Returns an error if JSON serialization fails.
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
    ecop_vocab: Option<String>,
}

impl ProvoExport {
    /// Create a new exporter.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            include_metadata: true,
            include_ecop: true,
            ecop_vocab: None,
        }
    }

    /// Set a pre-resolved ecoPrimals vocabulary URI (avoids `env::var` on export).
    #[must_use]
    pub fn with_ecop_vocab(mut self, uri: impl Into<String>) -> Self {
        self.ecop_vocab = Some(uri.into());
        self
    }

    /// Set whether to include metadata.
    #[must_use]
    pub const fn include_metadata(mut self, include: bool) -> Self {
        self.include_metadata = include;
        self
    }

    /// Set whether to include ecoPrimals extensions.
    #[must_use]
    pub const fn include_ecop(mut self, include: bool) -> Self {
        self.include_ecop = include;
        self
    }

    /// Build a `JsonLdDocument` using the pre-resolved ecoPrimals URI if set.
    fn make_doc(&self) -> JsonLdDocument {
        self.ecop_vocab
            .as_ref()
            .map_or_else(JsonLdDocument::new, |uri| {
                JsonLdDocument::with_ecop_vocab(uri)
            })
    }

    /// Export a single Braid as PROV-O JSON-LD.
    ///
    /// # Errors
    ///
    /// Returns an error if the export operation fails.
    pub fn export_braid(&self, braid: &Braid) -> Result<JsonLdDocument> {
        let mut doc = self.make_doc();

        doc.add_node(self.braid_to_entity(braid));

        if let Some(activity) = &braid.was_generated_by {
            doc.add_node(self.activity_to_prov(activity));
        }

        Ok(doc)
    }

    /// Export a provenance graph as PROV-O JSON-LD.
    ///
    /// # Errors
    ///
    /// Returns an error if the export operation fails.
    pub fn export_graph(&self, graph: &ProvenanceGraph) -> Result<JsonLdDocument> {
        let mut doc = self.make_doc();

        for braid in graph.entities.values() {
            doc.add_node(self.braid_to_entity(braid));
        }

        for activity in graph.activities.values() {
            doc.add_node(self.activity_to_prov(activity));
        }

        Ok(doc)
    }

    /// Convert a Braid to a PROV-O Entity node.
    fn braid_to_entity(&self, braid: &Braid) -> Value {
        let mut entity = IndexMap::new();

        entity.insert("@id".to_string(), json!(braid.id.as_str()));
        entity.insert(
            "@type".to_string(),
            json!(braid_type_to_prov_type(&braid.braid_type)),
        );
        entity.insert("dataHash".to_string(), json!(braid.data_hash));
        entity.insert("mimeType".to_string(), json!(braid.mime_type));
        entity.insert("size".to_string(), json!(braid.size));

        entity.insert(
            "wasAttributedTo".to_string(),
            json!(braid.was_attributed_to.as_str()),
        );

        entity.insert(
            "generatedAtTime".to_string(),
            json!(timestamp_to_iso(braid.generated_at_time)),
        );

        if let Some(invalidated) = braid.invalidated_at_time {
            entity.insert(
                "invalidatedAtTime".to_string(),
                json!(timestamp_to_iso(invalidated)),
            );
        }

        if let Some(activity) = &braid.was_generated_by {
            entity.insert("wasGeneratedBy".to_string(), json!(activity.id.as_str()));
        }

        if !braid.was_derived_from.is_empty() {
            let derived: Vec<Value> = braid
                .was_derived_from
                .iter()
                .filter_map(entity_reference_to_prov_id)
                .map(|id| json!(id))
                .collect();
            if !derived.is_empty() {
                entity.insert("wasDerivedFrom".to_string(), json!(derived));
            }
        }

        if !braid.alternate_of.is_empty() {
            let alternates: Vec<Value> = braid
                .alternate_of
                .iter()
                .filter_map(entity_reference_to_prov_id)
                .map(|id| json!(id))
                .collect();
            if !alternates.is_empty() {
                entity.insert("alternateOf".to_string(), json!(alternates));
            }
        }

        if self.include_metadata {
            if let Some(title) = &braid.metadata.title {
                entity.insert("rdfs:label".to_string(), json!(title));
            }
            if let Some(desc) = &braid.metadata.description {
                entity.insert("rdfs:comment".to_string(), json!(desc));
            }
        }

        if self.include_ecop {
            if let Some(primal) = &braid.ecop.source_primal {
                entity.insert("sourcePrimal".to_string(), json!(primal));
            }
            if let Some(gate) = &braid.ecop.source_gate {
                entity.insert("sourceGate".to_string(), json!(gate));
            }
            if let Some(niche) = &braid.ecop.niche {
                entity.insert("niche".to_string(), json!(niche));
            }
            if let Some(cga) = &braid.metadata.cross_gate {
                let mut cg = IndexMap::new();
                cg.insert("originGate".to_string(), json!(cga.origin_gate));
                cg.insert("targetGate".to_string(), json!(cga.target_gate));
                cg.insert(
                    "trustEvent".to_string(),
                    serde_json::to_value(&cga.trust_event).unwrap_or_else(|_| json!("unknown")),
                );
                cg.insert("originAgent".to_string(), json!(cga.origin_agent.as_str()));
                if let Some(target) = &cga.target_agent {
                    cg.insert("targetAgent".to_string(), json!(target.as_str()));
                }
                if let Some(fid) = &cga.family_id {
                    cg.insert("familyId".to_string(), json!(fid));
                }
                entity.insert("crossGateAttribution".to_string(), json!(cg));
            }
        }

        json!(entity)
    }

    /// Convert an Activity to a PROV-O Activity node.
    fn activity_to_prov(&self, activity: &Activity) -> Value {
        let mut act = IndexMap::new();

        act.insert("@id".to_string(), json!(activity.id.as_str()));
        act.insert("@type".to_string(), json!("Activity"));

        act.insert(
            "ecop:activityType".to_string(),
            json!(activity.activity_type.to_string()),
        );

        act.insert(
            "startedAtTime".to_string(),
            json!(timestamp_to_iso(activity.started_at_time)),
        );
        if let Some(end) = activity.ended_at_time {
            act.insert("endedAtTime".to_string(), json!(timestamp_to_iso(end)));
        }

        if !activity.was_associated_with.is_empty() {
            let agents: Vec<Value> = activity
                .was_associated_with
                .iter()
                .map(|a| {
                    let mut association = json!({
                        "prov:agent": a.agent.as_str(),
                        "prov:hadRole": format!("ecop:{:?}", a.role)
                    });
                    if let Some(principal) = &a.on_behalf_of {
                        association["prov:actedOnBehalfOf"] = json!(principal.as_str());
                    }
                    association
                })
                .collect();
            act.insert("wasAssociatedWith".to_string(), json!(agents));
        }

        if !activity.used.is_empty() {
            let used: Vec<Value> = activity
                .used
                .iter()
                .filter_map(|u| entity_reference_to_prov_id(&u.entity))
                .map(|id| json!(id))
                .collect();
            if !used.is_empty() {
                act.insert("used".to_string(), json!(used));
            }
        }

        if self.include_ecop
            && let Some(compute) = activity.ecop.compute_units
        {
            act.insert("computeUnits".to_string(), json!(compute));
        }

        json!(act)
    }
}

impl Default for ProvoExport {
    fn default() -> Self {
        Self::new()
    }
}

/// Map a braid type to its PROV-O `@type` label.
const fn braid_type_to_prov_type(braid_type: &BraidType) -> &'static str {
    match braid_type {
        BraidType::Activity => "Activity",
        BraidType::Agent => "Agent",
        BraidType::Collection { .. } => "Collection",
        _ => "Entity",
    }
}

/// Resolve an entity reference to a PROV-O `@id` URI.
fn entity_reference_to_prov_id(reference: &EntityReference) -> Option<String> {
    match reference {
        EntityReference::ById { braid_id } => Some(braid_id.as_str().to_string()),
        EntityReference::ByHash { data_hash, .. } => {
            Some(BraidId::from_hash(data_hash).as_str().to_string())
        },
        EntityReference::ByLoamEntry { entry_hash, .. } => {
            Some(BraidId::from_hash(entry_hash).as_str().to_string())
        },
        EntityReference::External {
            hash: Some(hash), ..
        } => Some(BraidId::from_hash(hash).as_str().to_string()),
        EntityReference::External { hash: None, .. } => None,
        EntityReference::Inline(entity) => {
            Some(BraidId::from_hash(&entity.hash).as_str().to_string())
        },
        _ => reference
            .content_hash()
            .map(|hash| BraidId::from_hash(hash).as_str().to_string()),
    }
}

/// Convert nanosecond timestamp to ISO 8601 string.
fn timestamp_to_iso(ts: sweet_grass_core::Timestamp) -> String {
    use chrono::{TimeZone, Utc};

    let nanos = ts.nanos();
    #[expect(
        clippy::cast_possible_wrap,
        reason = "nanos/1e9 fits in i64 for timestamps until year 2554; chrono::Utc::timestamp_opt requires i64"
    )]
    let secs = (nanos / 1_000_000_000) as i64;
    let nsecs = (nanos % 1_000_000_000) as u32;

    match Utc.timestamp_opt(secs, nsecs) {
        chrono::LocalResult::Single(dt) => dt.to_rfc3339(),
        _ => format!("{nanos}"),
    }
}

#[cfg(test)]
mod tests;
