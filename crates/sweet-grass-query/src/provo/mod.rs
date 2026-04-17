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
            context: Self::prov_o_context(),
            graph: Vec::new(),
        }
    }

    /// Get the standard PROV-O context.
    ///
    /// Namespace URIs are sourced from `sweet_grass_core::braid::types` constants.
    /// The ecoPrimals namespace honours the `ECOP_VOCAB_URI` env var at runtime.
    fn prov_o_context() -> Value {
        let ecop_ns = ecop_vocab_uri();

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
}

impl ProvoExport {
    /// Create a new exporter.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            include_metadata: true,
            include_ecop: true,
        }
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

    /// Export a single Braid as PROV-O JSON-LD.
    ///
    /// # Errors
    ///
    /// Returns an error if the export operation fails.
    pub fn export_braid(&self, braid: &Braid) -> Result<JsonLdDocument> {
        let mut doc = JsonLdDocument::new();

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
        let mut doc = JsonLdDocument::new();

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
        entity.insert("@type".to_string(), json!("Entity"));
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

        if let Some(activity) = &braid.was_generated_by {
            entity.insert("wasGeneratedBy".to_string(), json!(activity.id.as_str()));
        }

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
            if let Some(niche) = &braid.ecop.niche {
                entity.insert("niche".to_string(), json!(niche));
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
                    json!({
                        "prov:agent": a.agent.as_str(),
                        "prov:hadRole": format!("ecop:{:?}", a.role)
                    })
                })
                .collect();
            act.insert("wasAssociatedWith".to_string(), json!(agents));
        }

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

/// Convert nanosecond timestamp to ISO 8601 string.
fn timestamp_to_iso(nanos: u64) -> String {
    use chrono::{TimeZone, Utc};

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
