// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Privacy controls and data subject rights.
//!
//! `SweetGrass` respects human dignity through proper privacy controls:
//! - Configurable visibility levels
//! - Data subject rights (access, rectification, erasure)
//! - Retention policies with automatic cleanup
//! - Selective disclosure for provenance queries
//!
//! ## Design Principles
//!
//! 1. **Consent-based**: Data collection requires explicit consent
//! 2. **Minimal collection**: Only store what's needed for provenance
//! 3. **Transparent**: Users can see what's stored about them
//! 4. **Controllable**: Users can request modification or deletion
//!
//! ## Example
//!
//! ```rust
//! use sweet_grass_core::privacy::{DurationSecs, PrivacyLevel, RetentionPolicy, PrivacyMetadata};
//!
//! let privacy = PrivacyMetadata::builder()
//!     .visibility(PrivacyLevel::Private)
//!     .retention(RetentionPolicy::Duration(DurationSecs(86400 * 365)))
//!     .consent_obtained(true)
//!     .build();
//! ```

use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

use crate::agent::Did;

/// Privacy visibility level for Braids and metadata.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub enum PrivacyLevel {
    /// Fully public - visible to all.
    #[default]
    Public,

    /// Visible only to authenticated users.
    Authenticated,

    /// Visible only to the owner and explicitly granted parties.
    Private,

    /// Encrypted at rest, visible only with decryption key.
    Encrypted,

    /// Anonymized version available publicly.
    AnonymizedPublic {
        /// Fields that have been anonymized.
        anonymized_fields: Vec<String>,
    },
}

/// Retention policy for provenance data.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub enum RetentionPolicy {
    /// Keep indefinitely (default for immutable provenance).
    #[default]
    Indefinite,

    /// Keep for a specific duration from creation.
    Duration(DurationSecs),

    /// Keep until a specific date.
    Until(SystemTimeSecs),

    /// Delete after all derived data is deleted.
    UntilOrphaned,

    /// Subject to legal hold - cannot be deleted.
    LegalHold {
        /// Reason for the legal hold.
        reason: String,
        /// When the hold was placed.
        placed_at: SystemTimeSecs,
    },
}

/// Duration in seconds (serializable wrapper).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurationSecs(pub u64);

impl From<Duration> for DurationSecs {
    fn from(d: Duration) -> Self {
        Self(d.as_secs())
    }
}

impl From<DurationSecs> for Duration {
    fn from(d: DurationSecs) -> Self {
        Self::from_secs(d.0)
    }
}

/// System time in seconds (serializable wrapper).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemTimeSecs(pub u64);

impl From<SystemTime> for SystemTimeSecs {
    fn from(t: SystemTime) -> Self {
        let secs = t
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Self(secs)
    }
}

/// Data subject rights request types (GDPR-inspired).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DataSubjectRequest {
    /// Right to access - get all data about a subject.
    Access {
        /// The data subject requesting access.
        subject: Did,
    },

    /// Right to rectification - correct inaccurate data.
    Rectification {
        /// The data subject.
        subject: Did,
        /// The Braid ID to correct.
        braid_id: String,
        /// Corrections to apply.
        corrections: Vec<(String, String)>,
    },

    /// Right to erasure ("right to be forgotten").
    Erasure {
        /// The data subject requesting erasure.
        subject: Did,
        /// Specific Braids to erase (empty = all).
        braid_ids: Vec<String>,
        /// Reason for erasure request.
        reason: ErasureReason,
    },

    /// Right to data portability - export in standard format.
    Portability {
        /// The data subject.
        subject: Did,
        /// Desired export format.
        format: ExportFormat,
    },

    /// Right to object - opt out of processing.
    Objection {
        /// The data subject.
        subject: Did,
        /// Processing type being objected to.
        processing_type: ProcessingType,
    },
}

/// Reasons for erasure requests.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ErasureReason {
    /// Consent withdrawn.
    ConsentWithdrawn,
    /// Data no longer necessary.
    NoLongerNecessary,
    /// Unlawful processing.
    UnlawfulProcessing,
    /// Legal obligation.
    LegalObligation,
    /// Other reason (with description).
    Other(String),
}

/// Export formats for data portability.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub enum ExportFormat {
    /// JSON-LD format (W3C PROV-O compatible).
    #[default]
    JsonLd,
    /// JSON format.
    Json,
    /// CSV format.
    Csv,
    /// RDF/XML format.
    RdfXml,
}

/// Types of processing that can be objected to.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ProcessingType {
    /// Attribution tracking.
    Attribution,
    /// Reward calculation.
    RewardCalculation,
    /// Analytics.
    Analytics,
    /// Third-party sharing.
    ThirdPartySharing,
    /// All processing.
    All,
}

/// Privacy metadata attached to Braids.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PrivacyMetadata {
    /// Visibility level.
    pub visibility: PrivacyLevel,

    /// Data retention policy.
    pub retention: RetentionPolicy,

    /// Whether consent was obtained for storing this data.
    pub consent_obtained: bool,

    /// Consent details if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consent_details: Option<ConsentDetails>,

    /// Parties granted access (for Private visibility).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub granted_access: Vec<Did>,

    /// Processing restrictions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub processing_restrictions: Vec<ProcessingType>,

    /// Whether this data can be included in derived works.
    pub derivation_allowed: bool,
}

impl PrivacyMetadata {
    /// Create a new privacy metadata builder.
    #[must_use]
    pub const fn builder() -> PrivacyMetadataBuilder {
        PrivacyMetadataBuilder::new()
    }

    /// Check if a party has access.
    #[must_use]
    pub fn has_access(&self, requester: &Did, owner: &Did) -> bool {
        match &self.visibility {
            // Public, authenticated, and anonymized are all accessible
            // (Caller must verify authentication for Authenticated level)
            PrivacyLevel::Public
            | PrivacyLevel::Authenticated
            | PrivacyLevel::AnonymizedPublic { .. } => true,
            // Private and encrypted require ownership or explicit access grant
            PrivacyLevel::Private | PrivacyLevel::Encrypted => {
                requester == owner || self.granted_access.contains(requester)
            },
        }
    }

    /// Check if a processing type is restricted.
    #[must_use]
    pub fn is_processing_restricted(&self, processing: &ProcessingType) -> bool {
        self.processing_restrictions.contains(processing)
            || self.processing_restrictions.contains(&ProcessingType::All)
    }
}

/// Consent details for privacy compliance.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConsentDetails {
    /// When consent was obtained.
    pub obtained_at: SystemTimeSecs,

    /// How consent was obtained.
    pub mechanism: ConsentMechanism,

    /// Version of privacy policy consented to.
    pub policy_version: String,

    /// Specific purposes consented to.
    pub purposes: Vec<String>,
}

/// Mechanisms for obtaining consent.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ConsentMechanism {
    /// Explicit opt-in checkbox/button.
    ExplicitOptIn,
    /// Implicit through action.
    ImplicitAction,
    /// Contract necessity.
    ContractNecessity,
    /// Legal obligation.
    LegalObligation,
}

/// Builder for privacy metadata.
pub struct PrivacyMetadataBuilder {
    metadata: PrivacyMetadata,
}

impl PrivacyMetadataBuilder {
    /// Create a new builder with defaults.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            metadata: PrivacyMetadata {
                visibility: PrivacyLevel::Public,
                retention: RetentionPolicy::Indefinite,
                consent_obtained: false,
                consent_details: None,
                granted_access: Vec::new(),
                processing_restrictions: Vec::new(),
                derivation_allowed: true,
            },
        }
    }

    /// Set visibility level.
    #[must_use]
    pub fn visibility(mut self, level: PrivacyLevel) -> Self {
        self.metadata.visibility = level;
        self
    }

    /// Set retention policy.
    #[must_use]
    pub fn retention(mut self, policy: RetentionPolicy) -> Self {
        self.metadata.retention = policy;
        self
    }

    /// Mark consent as obtained.
    #[must_use]
    pub const fn consent_obtained(mut self, obtained: bool) -> Self {
        self.metadata.consent_obtained = obtained;
        self
    }

    /// Set consent details.
    #[must_use]
    pub fn consent_details(mut self, details: ConsentDetails) -> Self {
        self.metadata.consent_details = Some(details);
        self.metadata.consent_obtained = true;
        self
    }

    /// Grant access to a party.
    #[must_use]
    pub fn grant_access(mut self, party: Did) -> Self {
        self.metadata.granted_access.push(party);
        self
    }

    /// Add a processing restriction.
    #[must_use]
    pub fn restrict_processing(mut self, processing: ProcessingType) -> Self {
        self.metadata.processing_restrictions.push(processing);
        self
    }

    /// Set whether derivation is allowed.
    #[must_use]
    pub const fn allow_derivation(mut self, allowed: bool) -> Self {
        self.metadata.derivation_allowed = allowed;
        self
    }

    /// Build the privacy metadata.
    #[must_use]
    pub fn build(self) -> PrivacyMetadata {
        self.metadata
    }
}

impl Default for PrivacyMetadataBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
