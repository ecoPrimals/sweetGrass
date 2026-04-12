// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Braid type classification and serialization.
//!
//! Defines the [`BraidType`] enum and [`SummaryType`] for classifying
//! provenance records.  Serialization uses internally-tagged JSON for
//! human-readable formats and externally-tagged enum for bincode/tarpc.

use serde::{Deserialize, Serialize};

use crate::agent::Did;

use super::types::Timestamp;

/// Types of Braids.
///
/// **Serialization**: JSON uses `type` as an internal tag (see [`BraidTypeJson`]);
/// binary codecs use an externally tagged enum for bincode/tarpc compatibility.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum BraidType {
    /// Standard entity Braid (most common).
    #[default]
    Entity,

    /// Activity Braid.
    Activity,

    /// Agent Braid.
    Agent,

    /// Meta-Braid (summary of other Braids).
    Collection {
        /// Number of Braids summarized.
        member_count: u64,
        /// Type of summary.
        summary_type: SummaryType,
    },

    /// Delegation Braid (agent acting for another).
    Delegation {
        /// The delegate agent.
        delegate: Did,
        /// The principal agent.
        on_behalf_of: Did,
    },

    /// Slice provenance Braid.
    Slice {
        /// Slice operation mode.
        slice_mode: String,
        /// Origin spine ID.
        origin_spine: String,
    },
}

/// Summary types for meta-Braids.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SummaryType {
    /// Session summary.
    Session {
        /// The session ID being summarized.
        session_id: String,
    },
    /// Time period summary.
    Temporal {
        /// Start timestamp.
        start: Timestamp,
        /// End timestamp.
        end: Timestamp,
    },
    /// Activity type summary.
    ActivityGroup {
        /// The activity type being summarized.
        activity_type: String,
    },
    /// Agent contribution summary.
    AgentContributions {
        /// The agent being summarized.
        agent: Did,
    },
    /// Custom grouping.
    Custom {
        /// Criteria description.
        criteria: String,
    },
}

// ---------------------------------------------------------------------------
// Dual-format serde: JSON (internally tagged) vs bincode (externally tagged)
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum BraidTypeJson {
    Entity,
    Activity,
    Agent,
    Collection {
        member_count: u64,
        summary_type: SummaryType,
    },
    Delegation {
        delegate: Did,
        on_behalf_of: Did,
    },
    Slice {
        slice_mode: String,
        origin_spine: String,
    },
}

#[derive(Serialize, Deserialize)]
enum BraidTypeBin {
    Entity,
    Activity,
    Agent,
    Collection {
        member_count: u64,
        summary_type: SummaryType,
    },
    Delegation {
        delegate: Did,
        on_behalf_of: Did,
    },
    Slice {
        slice_mode: String,
        origin_spine: String,
    },
}

impl From<BraidType> for BraidTypeJson {
    fn from(t: BraidType) -> Self {
        match t {
            BraidType::Entity => Self::Entity,
            BraidType::Activity => Self::Activity,
            BraidType::Agent => Self::Agent,
            BraidType::Collection {
                member_count,
                summary_type,
            } => Self::Collection {
                member_count,
                summary_type,
            },
            BraidType::Delegation {
                delegate,
                on_behalf_of,
            } => Self::Delegation {
                delegate,
                on_behalf_of,
            },
            BraidType::Slice {
                slice_mode,
                origin_spine,
            } => Self::Slice {
                slice_mode,
                origin_spine,
            },
        }
    }
}

impl From<BraidTypeJson> for BraidType {
    fn from(t: BraidTypeJson) -> Self {
        match t {
            BraidTypeJson::Entity => Self::Entity,
            BraidTypeJson::Activity => Self::Activity,
            BraidTypeJson::Agent => Self::Agent,
            BraidTypeJson::Collection {
                member_count,
                summary_type,
            } => Self::Collection {
                member_count,
                summary_type,
            },
            BraidTypeJson::Delegation {
                delegate,
                on_behalf_of,
            } => Self::Delegation {
                delegate,
                on_behalf_of,
            },
            BraidTypeJson::Slice {
                slice_mode,
                origin_spine,
            } => Self::Slice {
                slice_mode,
                origin_spine,
            },
        }
    }
}

impl From<&BraidType> for BraidTypeBin {
    fn from(t: &BraidType) -> Self {
        match t {
            BraidType::Entity => Self::Entity,
            BraidType::Activity => Self::Activity,
            BraidType::Agent => Self::Agent,
            BraidType::Collection {
                member_count,
                summary_type,
            } => Self::Collection {
                member_count: *member_count,
                summary_type: summary_type.clone(),
            },
            BraidType::Delegation {
                delegate,
                on_behalf_of,
            } => Self::Delegation {
                delegate: delegate.clone(),
                on_behalf_of: on_behalf_of.clone(),
            },
            BraidType::Slice {
                slice_mode,
                origin_spine,
            } => Self::Slice {
                slice_mode: slice_mode.clone(),
                origin_spine: origin_spine.clone(),
            },
        }
    }
}

impl From<BraidTypeBin> for BraidType {
    fn from(t: BraidTypeBin) -> Self {
        match t {
            BraidTypeBin::Entity => Self::Entity,
            BraidTypeBin::Activity => Self::Activity,
            BraidTypeBin::Agent => Self::Agent,
            BraidTypeBin::Collection {
                member_count,
                summary_type,
            } => Self::Collection {
                member_count,
                summary_type,
            },
            BraidTypeBin::Delegation {
                delegate,
                on_behalf_of,
            } => Self::Delegation {
                delegate,
                on_behalf_of,
            },
            BraidTypeBin::Slice {
                slice_mode,
                origin_spine,
            } => Self::Slice {
                slice_mode,
                origin_spine,
            },
        }
    }
}

impl Serialize for BraidType {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            BraidTypeJson::from(self.clone()).serialize(serializer)
        } else {
            BraidTypeBin::from(self).serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for BraidType {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            BraidTypeJson::deserialize(deserializer).map(Into::into)
        } else {
            BraidTypeBin::deserialize(deserializer).map(Into::into)
        }
    }
}
