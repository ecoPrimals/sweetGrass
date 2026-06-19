// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! PROV agent type hierarchy with dual-format serialization.
//!
//! JSON uses an internal `type` tag for human readability; binary codecs
//! (bincode/tarpc) use an externally tagged enum for compact wire format.

use serde::{Deserialize, Serialize};

/// Agent types in the PROV model.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum AgentType {
    /// Human person.
    Person {
        /// Optional display name.
        name: Option<String>,
    },

    /// Software agent (AI, bot, service).
    SoftwareAgent {
        /// Software name.
        software_name: String,
        /// Software version.
        version: String,
    },

    /// Organization.
    Organization {
        /// Organization name.
        name: String,
        /// Organization type.
        org_type: Option<String>,
    },

    /// Hardware device.
    Device {
        /// Device type.
        device_type: String,
        /// Device identifier.
        device_id: Option<String>,
    },
}

impl Default for AgentType {
    fn default() -> Self {
        Self::Person { name: None }
    }
}

impl Serialize for AgentType {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            AgentTypeJson::from(self.clone()).serialize(serializer)
        } else {
            AgentTypeBin::from(self).serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for AgentType {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            AgentTypeJson::deserialize(deserializer).map(Into::into)
        } else {
            AgentTypeBin::deserialize(deserializer).map(Into::into)
        }
    }
}

// --- Private serde adapters ---

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum AgentTypeJson {
    Person {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    SoftwareAgent {
        software_name: String,
        version: String,
    },
    Organization {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        org_type: Option<String>,
    },
    Device {
        device_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        device_id: Option<String>,
    },
}

#[derive(Serialize, Deserialize)]
enum AgentTypeBin {
    Person {
        name: Option<String>,
    },
    SoftwareAgent {
        software_name: String,
        version: String,
    },
    Organization {
        name: String,
        org_type: Option<String>,
    },
    Device {
        device_type: String,
        device_id: Option<String>,
    },
}

impl From<AgentType> for AgentTypeJson {
    fn from(t: AgentType) -> Self {
        match t {
            AgentType::Person { name } => Self::Person { name },
            AgentType::SoftwareAgent {
                software_name,
                version,
            } => Self::SoftwareAgent {
                software_name,
                version,
            },
            AgentType::Organization { name, org_type } => Self::Organization { name, org_type },
            AgentType::Device {
                device_type,
                device_id,
            } => Self::Device {
                device_type,
                device_id,
            },
        }
    }
}

impl From<AgentTypeJson> for AgentType {
    fn from(t: AgentTypeJson) -> Self {
        match t {
            AgentTypeJson::Person { name } => Self::Person { name },
            AgentTypeJson::SoftwareAgent {
                software_name,
                version,
            } => Self::SoftwareAgent {
                software_name,
                version,
            },
            AgentTypeJson::Organization { name, org_type } => Self::Organization { name, org_type },
            AgentTypeJson::Device {
                device_type,
                device_id,
            } => Self::Device {
                device_type,
                device_id,
            },
        }
    }
}

impl From<&AgentType> for AgentTypeBin {
    fn from(t: &AgentType) -> Self {
        match t {
            AgentType::Person { name } => Self::Person { name: name.clone() },
            AgentType::SoftwareAgent {
                software_name,
                version,
            } => Self::SoftwareAgent {
                software_name: software_name.clone(),
                version: version.clone(),
            },
            AgentType::Organization { name, org_type } => Self::Organization {
                name: name.clone(),
                org_type: org_type.clone(),
            },
            AgentType::Device {
                device_type,
                device_id,
            } => Self::Device {
                device_type: device_type.clone(),
                device_id: device_id.clone(),
            },
        }
    }
}

impl From<AgentTypeBin> for AgentType {
    fn from(t: AgentTypeBin) -> Self {
        match t {
            AgentTypeBin::Person { name } => Self::Person { name },
            AgentTypeBin::SoftwareAgent {
                software_name,
                version,
            } => Self::SoftwareAgent {
                software_name,
                version,
            },
            AgentTypeBin::Organization { name, org_type } => Self::Organization { name, org_type },
            AgentTypeBin::Device {
                device_type,
                device_id,
            } => Self::Device {
                device_type,
                device_id,
            },
        }
    }
}
