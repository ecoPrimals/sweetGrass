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
/// **Serialization**: JSON uses `type` as an internal tag (`BraidTypeJson`);
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
//
// JSON needs `#[serde(tag = "type")]` for human-readable output; bincode
// needs external tagging (serde default). The macro below generates both
// helper enums and their bidirectional From impls from a single variant list,
// eliminating the triple-definition boilerplate.
// ---------------------------------------------------------------------------

macro_rules! braid_type_serde_helpers {
    (
        $(
            $variant:ident $( { $($field:ident : $ty:ty),+ $(,)? } )?
        ),+ $(,)?
    ) => {
        #[derive(Serialize, Deserialize)]
        #[serde(tag = "type")]
        enum BraidTypeJson {
            $( $variant $( { $($field: $ty),+ } )? ),+
        }

        #[derive(Serialize, Deserialize)]
        enum BraidTypeBin {
            $( $variant $( { $($field: $ty),+ } )? ),+
        }

        impl From<BraidType> for BraidTypeJson {
            fn from(t: BraidType) -> Self {
                match t {
                    $( BraidType::$variant $( { $($field),+ } )? =>
                        Self::$variant $( { $($field),+ } )? ),+
                }
            }
        }

        impl From<BraidTypeJson> for BraidType {
            fn from(t: BraidTypeJson) -> Self {
                match t {
                    $( BraidTypeJson::$variant $( { $($field),+ } )? =>
                        Self::$variant $( { $($field),+ } )? ),+
                }
            }
        }

        impl From<&BraidType> for BraidTypeBin {
            fn from(t: &BraidType) -> Self {
                match *t {
                    $( BraidType::$variant $( { $(ref $field),+ } )? =>
                        Self::$variant $( { $($field: $field.clone()),+ } )? ),+
                }
            }
        }

        impl From<BraidTypeBin> for BraidType {
            fn from(t: BraidTypeBin) -> Self {
                match t {
                    $( BraidTypeBin::$variant $( { $($field),+ } )? =>
                        Self::$variant $( { $($field),+ } )? ),+
                }
            }
        }
    };
}

braid_type_serde_helpers! {
    Entity,
    Activity,
    Agent,
    Collection { member_count: u64, summary_type: SummaryType },
    Delegation { delegate: Did, on_behalf_of: Did },
    Slice { slice_mode: String, origin_spine: String },
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
