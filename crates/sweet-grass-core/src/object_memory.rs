// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Object Memory — event timeline tracking for memory-bound digital objects.
//!
//! Provides `append_object_event` and `get_object_timeline` on top of braids.
//! Each event creates a derived braid linked to the previous one, forming
//! a verifiable chain of custody and history for any digital object.
//!
//! This module powers the fermenting system: digital objects whose value
//! accumulates through use. Every trade, loan, achievement, and cosmetic
//! change is a braid in the object's memory.

use std::collections::HashMap;

use crate::activity::{Activity, ActivityMetadata};
use crate::agent::{AgentAssociation, AgentRole, Did};
use crate::braid::{Braid, BraidBuilder, BraidId, EcoPrimalsAttributes, current_timestamp_nanos};
use crate::entity::EntityReference;
use crate::error::SweetGrassError;
use crate::{ActivityId, ActivityType};

/// An event in an object's memory timeline.
#[derive(Debug, Clone)]
pub struct ObjectEvent {
    /// Type of event (e.g., "mint", "trade", "achievement").
    pub event_type: String,
    /// Human-readable description.
    pub description: String,
    /// Actor who performed the event.
    pub actor: Did,
    /// Optional metadata (key-value pairs).
    pub metadata: HashMap<String, String>,
}

impl ObjectEvent {
    /// Create a new object event.
    #[must_use]
    pub fn new(event_type: impl Into<String>, description: impl Into<String>, actor: Did) -> Self {
        Self {
            event_type: event_type.into(),
            description: description.into(),
            actor,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the event.
    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Object memory: tracks a chain of braids for a single digital object.
///
/// Each object is identified by a stable ID (typically a loamSpine certificate ID).
/// Events form a linked list via `was_derived_from` references.
pub struct ObjectMemory {
    braids: Vec<Braid>,
    object_heads: HashMap<String, BraidId>,
}

impl ObjectMemory {
    /// Create a new empty object memory store.
    #[must_use]
    pub fn new() -> Self {
        Self {
            braids: Vec::new(),
            object_heads: HashMap::new(),
        }
    }

    /// Append an event to an object's memory timeline.
    ///
    /// Creates a new braid derived from the object's most recent braid
    /// (if any), forming a linked derivation chain.
    ///
    /// # Errors
    ///
    /// Returns an error if braid construction fails.
    pub fn append_object_event(
        &mut self,
        object_id: &str,
        event: &ObjectEvent,
    ) -> Result<BraidId, SweetGrassError> {
        let activity = Activity {
            id: ActivityId::from_task(&event.event_type),
            activity_type: ActivityType::Creation,
            used: Vec::new(),
            was_associated_with: vec![AgentAssociation {
                agent: event.actor.clone(),
                role: AgentRole::Creator,
                on_behalf_of: None,
                had_plan: Some("object_memory".into()),
            }],
            started_at_time: current_timestamp_nanos(),
            ended_at_time: Some(current_timestamp_nanos()),
            metadata: ActivityMetadata::default(),
            ecop: crate::activity::ActivityEcoPrimals::default(),
        };

        let data_hash = format!("sha256:{object_id}:{}", event.event_type);
        let ecop = EcoPrimalsAttributes {
            source_primal: Some(crate::identity::PRIMAL_NAME.into()),
            ..Default::default()
        };

        let mut builder = BraidBuilder::default();
        builder = builder
            .data_hash(&data_hash)
            .mime_type("application/x-object-event")
            .size(event.description.len() as u64)
            .attributed_to(event.actor.clone())
            .generated_by(activity)
            .ecop(ecop);

        if let Some(prev_head) = self.object_heads.get(object_id) {
            builder = builder.derived_from(EntityReference::by_id(prev_head.clone()));
        }

        let mut braid = builder.build()?;
        braid.metadata.description = Some(event.description.clone().into());

        let braid_id = braid.id.clone();
        self.object_heads.insert(object_id.into(), braid_id.clone());
        self.braids.push(braid);

        Ok(braid_id)
    }

    /// Get the full event timeline for an object, oldest first.
    ///
    /// Walks the derivation chain backwards from the head, then reverses.
    #[must_use]
    pub fn get_object_timeline(&self, object_id: &str) -> Vec<&Braid> {
        let Some(head_id) = self.object_heads.get(object_id) else {
            return Vec::new();
        };

        let mut timeline = Vec::new();
        let mut current_id = Some(head_id.clone());

        while let Some(id) = current_id {
            if let Some(braid) = self.braids.iter().find(|b| b.id == id) {
                timeline.push(braid);
                current_id = braid.was_derived_from.first().and_then(|e| match e {
                    EntityReference::ById { braid_id } => Some(braid_id.clone()),
                    _ => None,
                });
            } else {
                break;
            }
        }

        timeline.reverse();
        timeline
    }

    /// Export an object's timeline as a PROV-O compatible text report.
    #[must_use]
    pub fn export_prov_timeline(&self, object_id: &str) -> String {
        use std::fmt::Write;

        let timeline = self.get_object_timeline(object_id);
        if timeline.is_empty() {
            return format!("No history for object {object_id}");
        }

        let mut report = format!("=== Object Memory: {object_id} ===\n");
        let _ = writeln!(report, "Events: {}\n", timeline.len());

        for (idx, braid) in timeline.iter().enumerate() {
            let desc = braid
                .metadata
                .description
                .as_deref()
                .unwrap_or("(no description)");
            let agent = braid.was_attributed_to.as_str();
            let derived = if braid.was_derived_from.is_empty() {
                "(genesis)".to_string()
            } else {
                format!("derived from {} parent(s)", braid.was_derived_from.len())
            };

            let _ = write!(
                report,
                "[{idx}] {desc}\n    agent: {agent}\n    braid: {}\n    {derived}\n\n",
                braid.id.as_str()
            );
        }

        report
    }

    /// Get the number of events for an object.
    #[must_use]
    pub fn event_count(&self, object_id: &str) -> usize {
        self.get_object_timeline(object_id).len()
    }

    /// Get the total number of braids across all objects.
    #[must_use]
    pub const fn total_braids(&self) -> usize {
        self.braids.len()
    }

    /// Get the list of object IDs being tracked.
    #[must_use]
    pub fn tracked_objects(&self) -> Vec<&str> {
        self.object_heads.keys().map(String::as_str).collect()
    }
}

impl Default for ObjectMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test module: unwrap is standard in tests"
)]
mod tests {
    use super::*;

    #[test]
    fn append_single_event() {
        let mut memory = ObjectMemory::new();
        let actor = Did::new("did:key:alice");
        let event = ObjectEvent::new("mint", "Minted sword", actor);

        let result = memory.append_object_event("cert-001", &event);
        assert!(result.is_ok());
        assert_eq!(memory.event_count("cert-001"), 1);
    }

    #[test]
    fn timeline_is_ordered() {
        let mut memory = ObjectMemory::new();
        let actor = Did::new("did:key:alice");

        memory
            .append_object_event(
                "cert-001",
                &ObjectEvent::new("mint", "Minted", actor.clone()),
            )
            .unwrap();
        memory
            .append_object_event(
                "cert-001",
                &ObjectEvent::new("trade", "Traded", actor.clone()),
            )
            .unwrap();
        memory
            .append_object_event(
                "cert-001",
                &ObjectEvent::new("achievement", "Boss kill", actor),
            )
            .unwrap();

        let timeline = memory.get_object_timeline("cert-001");
        assert_eq!(timeline.len(), 3);
        assert!(
            timeline[0]
                .metadata
                .description
                .as_deref()
                .unwrap()
                .contains("Minted")
        );
        assert!(
            timeline[2]
                .metadata
                .description
                .as_deref()
                .unwrap()
                .contains("Boss kill")
        );
    }

    #[test]
    fn derivation_chain_links() {
        let mut memory = ObjectMemory::new();
        let actor = Did::new("did:key:alice");

        memory
            .append_object_event(
                "cert-001",
                &ObjectEvent::new("mint", "Minted", actor.clone()),
            )
            .unwrap();
        memory
            .append_object_event("cert-001", &ObjectEvent::new("trade", "Traded", actor))
            .unwrap();

        let timeline = memory.get_object_timeline("cert-001");
        assert!(timeline[0].was_derived_from.is_empty());
        assert_eq!(timeline[1].was_derived_from.len(), 1);
    }

    #[test]
    fn separate_objects_have_separate_timelines() {
        let mut memory = ObjectMemory::new();
        let actor = Did::new("did:key:alice");

        memory
            .append_object_event(
                "sword-001",
                &ObjectEvent::new("mint", "Sword minted", actor.clone()),
            )
            .unwrap();
        memory
            .append_object_event(
                "potion-002",
                &ObjectEvent::new("mint", "Potion minted", actor),
            )
            .unwrap();

        assert_eq!(memory.event_count("sword-001"), 1);
        assert_eq!(memory.event_count("potion-002"), 1);
        assert_eq!(memory.total_braids(), 2);
        assert_eq!(memory.tracked_objects().len(), 2);
    }

    #[test]
    fn prov_export_contains_events() {
        let mut memory = ObjectMemory::new();
        let actor = Did::new("did:key:alice");

        memory
            .append_object_event(
                "cert-001",
                &ObjectEvent::new("mint", "Minted legendary sword", actor),
            )
            .unwrap();

        let report = memory.export_prov_timeline("cert-001");
        assert!(report.contains("Minted legendary sword"));
        assert!(report.contains("did:key:alice"));
        assert!(report.contains("Events: 1"));
    }

    #[test]
    fn empty_timeline_for_unknown_object() {
        let memory = ObjectMemory::new();
        let timeline = memory.get_object_timeline("nonexistent");
        assert!(timeline.is_empty());

        let report = memory.export_prov_timeline("nonexistent");
        assert!(report.contains("No history"));
    }
}
