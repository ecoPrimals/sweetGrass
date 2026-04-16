// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Activity data structures - processes that create or transform data.
//!
//! Activities represent the "how" of provenance - the processes that
//! consume inputs and produce outputs.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::agent::{AgentAssociation, Did};
use crate::braid::Timestamp;
use crate::entity::EntityReference;

/// Activity identifier (URN format).
///
/// Uses `Arc<str>` internally so `.clone()` is O(1) (atomic refcount increment),
/// matching the zero-copy strategy used by [`ContentHash`], [`BraidId`], and [`Did`].
///
/// [`ContentHash`]: crate::braid::ContentHash
/// [`BraidId`]: crate::braid::BraidId
/// [`Did`]: crate::agent::Did
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct ActivityId(Arc<str>);

impl ActivityId {
    /// Create a new random Activity ID.
    #[must_use]
    pub fn new() -> Self {
        Self(format!("urn:activity:uuid:{}", Uuid::new_v4()).into())
    }

    /// Create an Activity ID from a task ID.
    #[must_use]
    pub fn from_task(task_id: &str) -> Self {
        Self(format!("urn:activity:task:{task_id}").into())
    }

    /// Create an Activity ID from a string (for deserialization from storage).
    #[must_use]
    pub fn from_string(s: impl Into<String>) -> Self {
        let s = s.into();
        Self(Arc::from(s.into_boxed_str()))
    }

    /// Get the inner string representation.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for ActivityId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self(Arc::from(s.into_boxed_str())))
    }
}

impl Default for ActivityId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ActivityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for ActivityId {
    fn from(s: &str) -> Self {
        Self(Arc::from(s))
    }
}

impl From<String> for ActivityId {
    fn from(s: String) -> Self {
        Self(Arc::from(s.into_boxed_str()))
    }
}

/// Standard activity types following PROV-O.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ActivityType {
    // === Data Creation ===
    /// Original data creation.
    #[default]
    Creation,
    /// Import from external source.
    Import,
    /// Extraction from larger dataset.
    Extraction,
    /// Automatic generation.
    Generation,

    // === Transformation ===
    /// Generic transformation.
    Transformation,
    /// Derivation from source.
    Derivation,
    /// Aggregation of multiple sources.
    Aggregation,
    /// Filtering/selection.
    Filtering,
    /// Merging multiple inputs.
    Merge,
    /// Splitting into multiple outputs.
    Split,

    // === Analysis ===
    /// Generic analysis.
    Analysis,
    /// Computational processing.
    Computation,
    /// Simulation run.
    Simulation,
    /// Machine learning inference or training.
    MachineLearning,
    /// Inference from data.
    Inference,

    // === Scientific ===
    /// Scientific experiment.
    Experiment,
    /// Observation/measurement collection.
    Observation,
    /// Direct measurement.
    Measurement,
    /// Validation of results.
    Validation,

    // === Collaboration ===
    /// Editing/modification.
    Editing,
    /// Review process.
    Review,
    /// Approval decision.
    Approval,
    /// Publication/release.
    Publication,

    // === Session events provider-specific ===
    /// Session start.
    SessionStart,
    /// Session commit.
    SessionCommit,
    /// Session rollback.
    SessionRollback,
    /// Slice checkout.
    SliceCheckout,
    /// Slice return.
    SliceReturn,

    // === Anchoring provider-specific ===
    /// Certificate minting.
    CertificateMint,
    /// Certificate transfer.
    CertificateTransfer,
    /// Certificate loan.
    CertificateLoan,
    /// Certificate return.
    CertificateReturn,

    // === Custom ===
    /// Custom activity type with URI.
    Custom {
        /// The type URI.
        type_uri: String,
    },
}

impl std::fmt::Display for ActivityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Custom { type_uri } => write!(f, "{type_uri}"),
            other => write!(f, "{other:?}"),
        }
    }
}

/// Role an entity plays in an activity.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EntityRole {
    /// Primary input.
    #[default]
    Input,
    /// Template/pattern.
    Template,
    /// Configuration data.
    Configuration,
    /// Reference data.
    Reference,
    /// Training data (ML).
    Training,
    /// Validation data.
    Validation,
    /// Custom role.
    Custom(String),
}

/// Extent of entity usage.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum UsageExtent {
    /// Full entity used.
    Full,
    /// Partial usage by fraction.
    Partial {
        /// Fraction used (0.0 to 1.0).
        fraction: f64,
    },
    /// Byte range used.
    Bytes {
        /// Start offset.
        start: u64,
        /// End offset.
        end: u64,
    },
    /// Subset description.
    Subset {
        /// Description of subset.
        description: String,
    },
}

/// Entity used as input to an activity.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UsedEntity {
    /// Reference to the entity.
    pub entity: EntityReference,

    /// Role this entity played.
    pub role: EntityRole,

    /// When it was used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<Timestamp>,

    /// How much was used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extent: Option<UsageExtent>,
}

impl UsedEntity {
    /// Create a new used entity with default role.
    #[must_use]
    pub fn new(entity: EntityReference) -> Self {
        Self {
            entity,
            role: EntityRole::default(),
            time: None,
            extent: None,
        }
    }

    /// Set the role.
    #[must_use]
    pub fn with_role(mut self, role: EntityRole) -> Self {
        self.role = role;
        self
    }

    /// Set the timestamp.
    #[must_use]
    pub const fn with_time(mut self, time: Timestamp) -> Self {
        self.time = Some(time);
        self
    }
}

/// ecoPrimals-specific activity attributes.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ActivityEcoPrimals {
    /// Compute units consumed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compute_units: Option<f64>,

    /// Storage used (bytes).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_bytes: Option<u64>,

    /// Network transfer (bytes).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_bytes: Option<u64>,

    /// Duration (nanoseconds).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ns: Option<u64>,

    /// Session events provider session reference (capability-based, runtime-discovered).
    #[serde(skip_serializing_if = "Option::is_none", alias = "rhizo_session")]
    pub session_ref: Option<String>,

    /// Compute provider task ID (capability-based, runtime-discovered).
    #[serde(skip_serializing_if = "Option::is_none", alias = "toadstool_task")]
    pub compute_task: Option<String>,

    /// Ledger entry reference (capability-based, runtime-discovered).
    #[serde(skip_serializing_if = "Option::is_none", alias = "loam_entry")]
    pub ledger_entry: Option<String>,

    /// Niche context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub niche: Option<String>,
}

/// Activity metadata.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ActivityMetadata {
    /// Description of the activity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Software/tool used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software: Option<String>,

    /// Software version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_version: Option<String>,

    /// Custom parameters.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub parameters: std::collections::HashMap<String, serde_json::Value>,
}

/// A PROV-O Activity (process/action).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Activity {
    /// Activity identifier.
    #[serde(rename = "@id")]
    pub id: ActivityId,

    /// Activity type.
    #[serde(rename = "@type")]
    pub activity_type: ActivityType,

    /// Inputs used by this activity.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub used: Vec<UsedEntity>,

    /// Agent(s) who performed the activity.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub was_associated_with: Vec<AgentAssociation>,

    /// When the activity started.
    pub started_at_time: Timestamp,

    /// When the activity ended.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at_time: Option<Timestamp>,

    /// Activity metadata.
    #[serde(default)]
    pub metadata: ActivityMetadata,

    /// ecoPrimals-specific attributes.
    #[serde(default)]
    pub ecop: ActivityEcoPrimals,
}

impl Activity {
    /// Create a new Activity builder.
    #[must_use]
    pub fn builder(activity_type: ActivityType) -> ActivityBuilder {
        ActivityBuilder::new(activity_type)
    }

    /// Get the duration of the activity in nanoseconds.
    #[must_use]
    pub fn duration_ns(&self) -> Option<u64> {
        self.ended_at_time
            .map(|end| end.saturating_sub(self.started_at_time))
    }

    /// Check if the activity has completed.
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        self.ended_at_time.is_some()
    }

    /// Get the primary agent (first associated agent).
    #[must_use]
    pub fn primary_agent(&self) -> Option<&Did> {
        self.was_associated_with.first().map(|a| &a.agent)
    }
}

/// Builder for creating Activities.
pub struct ActivityBuilder {
    activity_type: ActivityType,
    used: Vec<UsedEntity>,
    was_associated_with: Vec<AgentAssociation>,
    started_at_time: Timestamp,
    ended_at_time: Option<Timestamp>,
    metadata: ActivityMetadata,
    ecop: ActivityEcoPrimals,
}

impl ActivityBuilder {
    /// Create a new Activity builder.
    #[must_use]
    pub fn new(activity_type: ActivityType) -> Self {
        Self {
            activity_type,
            used: Vec::new(),
            was_associated_with: Vec::new(),
            started_at_time: crate::braid::current_timestamp_nanos(),
            ended_at_time: None,
            metadata: ActivityMetadata::default(),
            ecop: ActivityEcoPrimals::default(),
        }
    }

    /// Add a used entity.
    #[must_use]
    pub fn uses(mut self, entity: UsedEntity) -> Self {
        self.used.push(entity);
        self
    }

    /// Add an associated agent.
    #[must_use]
    pub fn associated_with(mut self, assoc: AgentAssociation) -> Self {
        self.was_associated_with.push(assoc);
        self
    }

    /// Set the start time.
    #[must_use]
    pub const fn started_at(mut self, time: Timestamp) -> Self {
        self.started_at_time = time;
        self
    }

    /// Set the end time.
    #[must_use]
    pub const fn ended_at(mut self, time: Timestamp) -> Self {
        self.ended_at_time = Some(time);
        self
    }

    /// Set compute units.
    #[must_use]
    pub const fn compute_units(mut self, units: f64) -> Self {
        self.ecop.compute_units = Some(units);
        self
    }

    /// Set the session events provider session reference.
    #[must_use]
    pub fn session_ref(mut self, session_id: impl Into<String>) -> Self {
        self.ecop.session_ref = Some(session_id.into());
        self
    }

    /// Set the compute provider task.
    #[must_use]
    pub fn compute_task(mut self, task_id: impl Into<String>) -> Self {
        self.ecop.compute_task = Some(task_id.into());
        self
    }

    /// Set metadata.
    #[must_use]
    pub fn metadata(mut self, metadata: ActivityMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Build the Activity.
    #[must_use]
    pub fn build(self) -> Activity {
        Activity {
            id: ActivityId::new(),
            activity_type: self.activity_type,
            used: self.used,
            was_associated_with: self.was_associated_with,
            started_at_time: self.started_at_time,
            ended_at_time: self.ended_at_time,
            metadata: self.metadata,
            ecop: self.ecop,
        }
    }
}

#[cfg(test)]
mod tests;
