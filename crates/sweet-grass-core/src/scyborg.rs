// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

//! scyBorg provenance types — triple-copyleft enforcement.
//!
//! Defines content categories and license expressions for the scyBorg
//! framework, which combines AGPL-3.0 (code), ORC (game mechanics),
//! and CC-BY-SA-4.0 (creative content).
//!
//! These types are stored in sweetGrass braids, referenced by rhizoCrypt
//! vertex metadata, and enforced by LoamSpine certificates.

use serde::{Deserialize, Serialize};

/// Content category for scyBorg triple-copyleft.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContentCategory {
    /// Source code, scripts, build systems.
    Code,
    /// Game mechanics, rules, systems, balance data.
    Mechanic,
    /// Creative content: art, music, narrative, world-building.
    Creative,
    /// Data sets, measurements, experimental results.
    Data,
    /// Trained models, weights, inference artifacts.
    Model,
    /// Documentation, specifications, papers.
    Documentation,
    /// Mixed content requiring per-element categorization.
    Mixed,
}

impl ContentCategory {
    /// The default license for this content category under scyBorg.
    #[must_use]
    pub const fn default_license(&self) -> LicenseId {
        match self {
            Self::Code | Self::Documentation | Self::Model | Self::Mixed => LicenseId::Agpl3Only,
            Self::Mechanic => LicenseId::Orc,
            Self::Creative | Self::Data => LicenseId::CcBySa4,
        }
    }

    /// Human-readable display name for this category.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Code => "code",
            Self::Mechanic => "game mechanic",
            Self::Creative => "creative content",
            Self::Data => "data",
            Self::Model => "model",
            Self::Documentation => "documentation",
            Self::Mixed => "mixed content",
        }
    }
}

/// Well-known license identifiers for scyBorg.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LicenseId {
    /// AGPL-3.0-only — code and models.
    Agpl3Only,
    /// ORC — Open RPG Creative (game mechanics).
    Orc,
    /// CC-BY-SA-4.0 — creative content.
    CcBySa4,
    /// Custom license (referenced by SPDX expression).
    Custom,
}

impl LicenseId {
    /// SPDX identifier string for this license.
    #[must_use]
    pub const fn spdx(&self) -> &'static str {
        match self {
            Self::Agpl3Only => "AGPL-3.0-only",
            Self::Orc => "ORC-1.0",
            Self::CcBySa4 => "CC-BY-SA-4.0",
            Self::Custom => "LicenseRef-custom",
        }
    }
}

/// A license expression attached to a braid or certificate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LicenseExpression {
    /// Primary license.
    pub license: LicenseId,
    /// SPDX expression string (for complex/compound licenses).
    pub spdx_expression: String,
    /// Content category this license applies to.
    pub category: ContentCategory,
}

/// An attribution notice generated from provenance data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributionNotice {
    /// The content being attributed.
    pub content_ref: String,
    /// Category of the content.
    pub category: ContentCategory,
    /// License that applies.
    pub license: LicenseExpression,
    /// Chain of contributors (DIDs or agent names).
    pub contributors: Vec<String>,
    /// Derivation chain (braid references showing how content evolved).
    pub derivation_chain: Vec<String>,
    /// Human-readable attribution text.
    pub notice_text: String,
}

impl AttributionNotice {
    /// Generate notice text from the structured attribution data.
    #[must_use]
    pub fn generate_notice_text(&self) -> String {
        let contributors = if self.contributors.is_empty() {
            String::from("Unknown contributors")
        } else {
            self.contributors.join(", ")
        };
        format!(
            "{} by {} — licensed under {} ({})",
            self.content_ref,
            contributors,
            self.license.spdx_expression,
            self.category.display_name(),
        )
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_content_category_default_licenses() {
        assert_eq!(
            ContentCategory::Code.default_license(),
            LicenseId::Agpl3Only
        );
        assert_eq!(
            ContentCategory::Documentation.default_license(),
            LicenseId::Agpl3Only
        );
        assert_eq!(ContentCategory::Mechanic.default_license(), LicenseId::Orc);
        assert_eq!(
            ContentCategory::Creative.default_license(),
            LicenseId::CcBySa4
        );
        assert_eq!(ContentCategory::Data.default_license(), LicenseId::CcBySa4);
        assert_eq!(
            ContentCategory::Model.default_license(),
            LicenseId::Agpl3Only
        );
        assert_eq!(
            ContentCategory::Mixed.default_license(),
            LicenseId::Agpl3Only
        );
    }

    #[test]
    fn test_license_spdx_ids() {
        assert_eq!(LicenseId::Agpl3Only.spdx(), "AGPL-3.0-only");
        assert_eq!(LicenseId::Orc.spdx(), "ORC-1.0");
        assert_eq!(LicenseId::CcBySa4.spdx(), "CC-BY-SA-4.0");
        assert_eq!(LicenseId::Custom.spdx(), "LicenseRef-custom");
    }

    #[test]
    fn test_attribution_notice_generation() {
        let notice = AttributionNotice {
            content_ref: "Tileset v1.0".to_string(),
            category: ContentCategory::Creative,
            license: LicenseExpression {
                license: LicenseId::CcBySa4,
                spdx_expression: "CC-BY-SA-4.0".to_string(),
                category: ContentCategory::Creative,
            },
            contributors: vec!["Alice".to_string(), "Bob".to_string()],
            derivation_chain: vec!["braid:abc123".to_string()],
            notice_text: String::new(),
        };
        let generated = notice.generate_notice_text();
        assert!(generated.contains("Tileset v1.0"));
        assert!(generated.contains("Alice, Bob"));
        assert!(generated.contains("CC-BY-SA-4.0"));
        assert!(generated.contains("creative content"));
    }

    #[test]
    fn test_attribution_notice_generation_empty_contributors() {
        let notice = AttributionNotice {
            content_ref: "Orphan work".to_string(),
            category: ContentCategory::Code,
            license: LicenseExpression {
                license: LicenseId::Agpl3Only,
                spdx_expression: "AGPL-3.0-only".to_string(),
                category: ContentCategory::Code,
            },
            contributors: vec![],
            derivation_chain: vec![],
            notice_text: String::new(),
        };
        let generated = notice.generate_notice_text();
        assert!(generated.contains("Unknown contributors"));
        assert!(generated.contains("Orphan work"));
        assert!(generated.contains("AGPL-3.0-only"));
    }

    #[test]
    fn test_serialization_roundtrip() {
        let category = ContentCategory::Mechanic;
        let json = serde_json::to_string(&category).unwrap();
        let restored: ContentCategory = serde_json::from_str(&json).unwrap();
        assert_eq!(category, restored);

        let license_id = LicenseId::Orc;
        let json = serde_json::to_string(&license_id).unwrap();
        let restored: LicenseId = serde_json::from_str(&json).unwrap();
        assert_eq!(license_id, restored);

        let expr = LicenseExpression {
            license: LicenseId::CcBySa4,
            spdx_expression: "CC-BY-SA-4.0".to_string(),
            category: ContentCategory::Creative,
        };
        let json = serde_json::to_string(&expr).unwrap();
        let restored: LicenseExpression = serde_json::from_str(&json).unwrap();
        assert_eq!(expr, restored);

        let notice = AttributionNotice {
            content_ref: "Test content".to_string(),
            category: ContentCategory::Code,
            license: LicenseExpression {
                license: LicenseId::Agpl3Only,
                spdx_expression: "AGPL-3.0-only".to_string(),
                category: ContentCategory::Code,
            },
            contributors: vec!["did:key:z6Mk...".to_string()],
            derivation_chain: vec!["braid:xyz".to_string()],
            notice_text: "Custom notice".to_string(),
        };
        let json = serde_json::to_string(&notice).unwrap();
        let restored: AttributionNotice = serde_json::from_str(&json).unwrap();
        assert_eq!(notice.content_ref, restored.content_ref);
        assert_eq!(notice.category, restored.category);
        assert_eq!(notice.license.license, restored.license.license);
        assert_eq!(notice.contributors, restored.contributors);
        assert_eq!(notice.notice_text, restored.notice_text);
    }
}
