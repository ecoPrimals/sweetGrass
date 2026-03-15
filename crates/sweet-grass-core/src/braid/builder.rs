// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! `BraidBuilder` - builder pattern for constructing Braids.

use crate::activity::Activity;
use crate::agent::Did;
use crate::entity::EntityReference;

use super::types::{
    BraidContext, BraidId, BraidMetadata, BraidSignature, BraidType, ContentHash,
    EcoPrimalsAttributes,
};

/// Builder for creating Braids.
#[derive(Default)]
pub struct BraidBuilder {
    data_hash: Option<ContentHash>,
    mime_type: Option<String>,
    size: Option<u64>,
    braid_type: BraidType,
    was_generated_by: Option<Activity>,
    was_derived_from: Vec<EntityReference>,
    was_attributed_to: Option<Did>,
    metadata: BraidMetadata,
    ecop: EcoPrimalsAttributes,
}

impl BraidBuilder {
    /// Set the data hash.
    #[must_use]
    pub fn data_hash(mut self, hash: impl Into<ContentHash>) -> Self {
        self.data_hash = Some(hash.into());
        self
    }

    /// Set the MIME type.
    #[must_use]
    pub fn mime_type(mut self, mime: impl Into<String>) -> Self {
        self.mime_type = Some(mime.into());
        self
    }

    /// Set the size.
    #[must_use]
    pub const fn size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    /// Set the Braid type.
    #[must_use]
    pub fn braid_type(mut self, braid_type: BraidType) -> Self {
        self.braid_type = braid_type;
        self
    }

    /// Set the generating activity.
    #[must_use]
    pub fn generated_by(mut self, activity: Activity) -> Self {
        self.was_generated_by = Some(activity);
        self
    }

    /// Add a derivation source.
    #[must_use]
    pub fn derived_from(mut self, entity: EntityReference) -> Self {
        self.was_derived_from.push(entity);
        self
    }

    /// Set the attribution.
    #[must_use]
    pub fn attributed_to(mut self, did: Did) -> Self {
        self.was_attributed_to = Some(did);
        self
    }

    /// Set metadata.
    #[must_use]
    pub fn metadata(mut self, metadata: BraidMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Set ecoPrimals attributes.
    #[must_use]
    pub fn ecop(mut self, ecop: EcoPrimalsAttributes) -> Self {
        self.ecop = ecop;
        self
    }

    /// Build the Braid.
    ///
    /// # Errors
    ///
    /// Returns an error if required fields are missing.
    pub fn build(self) -> crate::Result<super::Braid> {
        let data_hash = self
            .data_hash
            .ok_or_else(|| crate::SweetGrassError::Validation("data_hash is required".into()))?;
        let mime_type = self
            .mime_type
            .ok_or_else(|| crate::SweetGrassError::Validation("mime_type is required".into()))?;
        let size = self
            .size
            .ok_or_else(|| crate::SweetGrassError::Validation("size is required".into()))?;
        let was_attributed_to = self.was_attributed_to.ok_or_else(|| {
            crate::SweetGrassError::Validation("was_attributed_to is required".into())
        })?;

        Ok(super::Braid {
            context: BraidContext::default(),
            id: BraidId::from_hash(&data_hash),
            braid_type: self.braid_type,
            data_hash,
            mime_type,
            size,
            was_generated_by: self.was_generated_by,
            was_derived_from: self.was_derived_from,
            was_attributed_to,
            generated_at_time: super::types::current_timestamp_nanos(),
            metadata: self.metadata,
            ecop: self.ecop,
            signature: BraidSignature::unsigned(),
            loam_anchor: None,
        })
    }
}
