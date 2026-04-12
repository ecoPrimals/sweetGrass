// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Row-to-domain conversion for `PostgreSQL`.
//!
//! Pure functions that convert `PgRow` values into domain types (`Braid`,
//! `Activity`) and handle the `u64 ↔ i64` impedance mismatch between Rust
//! domain types and `PostgreSQL` storage.

use sqlx::Row;

use sweet_grass_core::{
    Braid, BraidId,
    activity::{
        Activity, ActivityEcoPrimals, ActivityId, ActivityMetadata, ActivityType, UsedEntity,
    },
    agent::{AgentAssociation, Did},
};
use sweet_grass_store::StoreError;

// ============================================================================
// Safe integer conversion helpers for PostgreSQL storage
// ============================================================================
// PostgreSQL doesn't have unsigned 64-bit integers, so we store u64 as i64.

/// Convert `u64` to `i64` for `PostgreSQL` storage.
/// Returns an error if the value would overflow.
pub fn u64_to_i64(value: u64) -> std::result::Result<i64, StoreError> {
    i64::try_from(value)
        .map_err(|_| StoreError::Internal(format!("Value {value} exceeds maximum storable size")))
}

/// Convert `i64` from `PostgreSQL` to `u64`.
/// Negative values are clamped to 0 (shouldn't happen with valid data).
#[expect(
    clippy::cast_sign_loss,
    reason = "Negative clamped to 0; cast is lossless for non-negative i64"
)]
#[must_use]
pub const fn i64_to_u64(value: i64) -> u64 {
    if value < 0 { 0 } else { value as u64 }
}

/// Convert `i64` from `PostgreSQL` to `usize` for counts/offsets.
/// Truncation on 32-bit targets is acceptable; PG row counts fit in usize.
#[expect(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    reason = "Clamp to 0 then cast; truncation on 32-bit is acceptable for PG counts/offsets"
)]
#[must_use]
pub const fn i64_to_usize(value: i64) -> usize {
    if value < 0 { 0 } else { value as usize }
}

// ============================================================================
// Row-to-domain conversion
// ============================================================================

/// Convert a database row to a `Braid`.
pub fn row_to_braid(row: &sqlx::postgres::PgRow) -> sweet_grass_store::Result<Braid> {
    let braid_id: String = row
        .try_get("braid_id")
        .map_err(|e| StoreError::Internal(e.to_string()))?;
    let data_hash: String = row
        .try_get("data_hash")
        .map_err(|e| StoreError::Internal(e.to_string()))?;
    let mime_type: String = row
        .try_get("mime_type")
        .map_err(|e| StoreError::Internal(e.to_string()))?;
    let size: i64 = row
        .try_get("size")
        .map_err(|e| StoreError::Internal(e.to_string()))?;
    let attributed_to: String = row
        .try_get("attributed_to")
        .map_err(|e| StoreError::Internal(e.to_string()))?;
    let generated_at_time: i64 = row
        .try_get("generated_at_time")
        .map_err(|e| StoreError::Internal(e.to_string()))?;
    let metadata: serde_json::Value = row
        .try_get("metadata")
        .map_err(|e| StoreError::Internal(e.to_string()))?;
    let ecop: serde_json::Value = row
        .try_get("ecop")
        .map_err(|e| StoreError::Internal(e.to_string()))?;
    let was_derived_from: serde_json::Value = row
        .try_get("was_derived_from")
        .map_err(|e| StoreError::Internal(e.to_string()))?;
    let was_generated_by: Option<serde_json::Value> = row
        .try_get("was_generated_by")
        .map_err(|e| StoreError::Internal(e.to_string()))?;
    let signature: serde_json::Value = row
        .try_get("signature")
        .map_err(|e| StoreError::Internal(e.to_string()))?;

    let mut builder = Braid::builder()
        .data_hash(data_hash.as_str())
        .mime_type(&mime_type)
        .size(i64_to_u64(size))
        .attributed_to(Did::new(attributed_to));

    if let Ok(meta) = serde_json::from_value(metadata) {
        builder = builder.metadata(meta);
    }

    let mut braid = builder
        .build()
        .map_err(|e| StoreError::Internal(e.to_string()))?;

    braid.id = BraidId::from_string(braid_id);
    braid.generated_at_time = i64_to_u64(generated_at_time);

    if let Ok(derived) = serde_json::from_value(was_derived_from) {
        braid.was_derived_from = derived;
    }

    if let Some(gen_by) = was_generated_by
        && let Ok(activity) = serde_json::from_value(gen_by)
    {
        braid.was_generated_by = Some(activity);
    }

    if let Ok(witness) =
        serde_json::from_value::<sweet_grass_core::dehydration::Witness>(signature.clone())
    {
        braid.witness = witness;
    } else {
        braid.witness = legacy_signature_to_witness(&signature);
    }

    if let Ok(ecop_parsed) = serde_json::from_value(ecop) {
        braid.ecop = ecop_parsed;
    }

    Ok(braid)
}

/// Convert a database row to an `Activity`.
pub fn row_to_activity(row: &sqlx::postgres::PgRow) -> sweet_grass_store::Result<Activity> {
    let activity_id: String = row
        .try_get("activity_id")
        .map_err(|e| StoreError::Internal(e.to_string()))?;
    let activity_type_str: String = row
        .try_get("activity_type")
        .map_err(|e| StoreError::Internal(e.to_string()))?;
    let started_at_time: i64 = row
        .try_get("started_at_time")
        .map_err(|e| StoreError::Internal(e.to_string()))?;
    let ended_at_time: Option<i64> = row
        .try_get("ended_at_time")
        .map_err(|e| StoreError::Internal(e.to_string()))?;
    let used_entities: serde_json::Value = row
        .try_get("used_entities")
        .map_err(|e| StoreError::Internal(e.to_string()))?;
    let was_associated_with: serde_json::Value = row
        .try_get("was_associated_with")
        .map_err(|e| StoreError::Internal(e.to_string()))?;
    let metadata: serde_json::Value = row
        .try_get("metadata")
        .map_err(|e| StoreError::Internal(e.to_string()))?;
    let ecop: serde_json::Value = row
        .try_get("ecop")
        .map_err(|e| StoreError::Internal(e.to_string()))?;

    let activity_type = parse_activity_type(&activity_type_str);

    let used: Vec<UsedEntity> = serde_json::from_value(used_entities)
        .map_err(|e| StoreError::Internal(format!("activity used_entities: {e}")))?;
    let associations: Vec<AgentAssociation> = serde_json::from_value(was_associated_with)
        .map_err(|e| StoreError::Internal(format!("activity was_associated_with: {e}")))?;
    let meta: ActivityMetadata = serde_json::from_value(metadata)
        .map_err(|e| StoreError::Internal(format!("activity metadata: {e}")))?;
    let ecop_parsed: ActivityEcoPrimals = serde_json::from_value(ecop)
        .map_err(|e| StoreError::Internal(format!("activity ecop: {e}")))?;

    Ok(Activity {
        id: ActivityId::from_string(activity_id),
        activity_type,
        used,
        was_associated_with: associations,
        started_at_time: i64_to_u64(started_at_time),
        ended_at_time: ended_at_time.map(i64_to_u64),
        metadata: meta,
        ecop: ecop_parsed,
    })
}

/// Convert a pre-`WireWitnessRef` (LD-Proof) JSONB value into a [`Witness`].
pub fn legacy_signature_to_witness(
    v: &serde_json::Value,
) -> sweet_grass_core::dehydration::Witness {
    use sweet_grass_core::dehydration::{
        WITNESS_ALGORITHM_ED25519, WITNESS_ENCODING_BASE64, WITNESS_ENCODING_NONE,
        WITNESS_KIND_MARKER, WITNESS_KIND_SIGNATURE, WITNESS_TIER_LOCAL, WITNESS_TIER_OPEN,
    };

    let sig_type = v.get("type").and_then(|v| v.as_str()).unwrap_or("Unsigned");
    let proof_value = v.get("proof_value").and_then(|v| v.as_str()).unwrap_or("");
    let verification_method = v
        .get("verification_method")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let created = v
        .get("created")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);

    let is_signed = sig_type != "Unsigned" && !proof_value.is_empty();
    sweet_grass_core::dehydration::Witness {
        agent: Did::new(verification_method.split('#').next().unwrap_or("")),
        kind: if is_signed {
            WITNESS_KIND_SIGNATURE
        } else {
            WITNESS_KIND_MARKER
        }
        .to_owned(),
        evidence: proof_value.to_owned(),
        witnessed_at: created,
        encoding: if is_signed {
            WITNESS_ENCODING_BASE64
        } else {
            WITNESS_ENCODING_NONE
        }
        .to_owned(),
        algorithm: if sig_type.contains("Ed25519") {
            Some(WITNESS_ALGORITHM_ED25519.to_owned())
        } else {
            None
        },
        tier: Some(
            if is_signed {
                WITNESS_TIER_LOCAL
            } else {
                WITNESS_TIER_OPEN
            }
            .to_owned(),
        ),
        context: None,
    }
}

/// Parse an activity type from its string representation.
pub fn parse_activity_type(s: &str) -> ActivityType {
    match s {
        "Creation" => ActivityType::Creation,
        "Import" => ActivityType::Import,
        "Extraction" => ActivityType::Extraction,
        "Generation" => ActivityType::Generation,
        "Transformation" => ActivityType::Transformation,
        "Derivation" => ActivityType::Derivation,
        "Aggregation" => ActivityType::Aggregation,
        "Filtering" => ActivityType::Filtering,
        "Merge" => ActivityType::Merge,
        "Split" => ActivityType::Split,
        "Analysis" => ActivityType::Analysis,
        "Computation" => ActivityType::Computation,
        "Simulation" => ActivityType::Simulation,
        "MachineLearning" => ActivityType::MachineLearning,
        "Inference" => ActivityType::Inference,
        "Experiment" => ActivityType::Experiment,
        "Observation" => ActivityType::Observation,
        "Measurement" => ActivityType::Measurement,
        "Validation" => ActivityType::Validation,
        "Editing" => ActivityType::Editing,
        "Review" => ActivityType::Review,
        "Approval" => ActivityType::Approval,
        "Publication" => ActivityType::Publication,
        "SessionStart" => ActivityType::SessionStart,
        "SessionCommit" => ActivityType::SessionCommit,
        "SessionRollback" => ActivityType::SessionRollback,
        "SliceCheckout" => ActivityType::SliceCheckout,
        "SliceReturn" => ActivityType::SliceReturn,
        "CertificateMint" => ActivityType::CertificateMint,
        "CertificateTransfer" => ActivityType::CertificateTransfer,
        "CertificateLoan" => ActivityType::CertificateLoan,
        "CertificateReturn" => ActivityType::CertificateReturn,
        other => ActivityType::Custom {
            type_uri: other.to_string(),
        },
    }
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "test assertions")]
mod tests {
    use proptest::prelude::*;

    use sweet_grass_core::{
        activity::ActivityType,
        dehydration::{
            WITNESS_ALGORITHM_ED25519, WITNESS_ENCODING_BASE64, WITNESS_ENCODING_NONE,
            WITNESS_KIND_MARKER, WITNESS_KIND_SIGNATURE, WITNESS_TIER_LOCAL, WITNESS_TIER_OPEN,
        },
    };
    use sweet_grass_store::StoreError;

    use super::{
        i64_to_u64, i64_to_usize, legacy_signature_to_witness, parse_activity_type, u64_to_i64,
    };

    #[test]
    fn u64_to_i64_zero() {
        assert_eq!(u64_to_i64(0).expect("zero"), 0);
    }

    #[test]
    fn u64_to_i64_max_representable() {
        assert_eq!(u64_to_i64(i64::MAX as u64).expect("fits"), i64::MAX);
    }

    #[test]
    fn u64_to_i64_overflows_past_i64_max() {
        let err = u64_to_i64((i64::MAX as u64) + 1).expect_err("overflow");
        assert!(matches!(err, StoreError::Internal(_)));
    }

    #[test]
    fn u64_to_i64_u64_max_overflows() {
        let err = u64_to_i64(u64::MAX).expect_err("overflow");
        assert!(matches!(err, StoreError::Internal(_)));
    }

    proptest! {
        #[test]
        fn u64_to_i64_roundtrip_via_i64_to_u64_for_nonnegative(n in 0i64..=i64::MAX) {
            let u = i64_to_u64(n);
            let back = u64_to_i64(u).expect("non-negative i64 fits in u64 and i64");
            prop_assert_eq!(back, n);
        }
    }

    #[test]
    fn i64_to_u64_cases() {
        assert_eq!(i64_to_u64(0), 0);
        assert_eq!(i64_to_u64(-1), 0);
        assert_eq!(i64_to_u64(i64::MAX), i64::MAX as u64);
        assert_eq!(i64_to_u64(i64::MIN), 0);
    }

    #[test]
    fn i64_to_usize_cases() {
        assert_eq!(i64_to_usize(0), 0);
        assert_eq!(i64_to_usize(-1), 0);
        #[expect(
            clippy::cast_possible_truncation,
            reason = "verifying production behavior"
        )]
        let expected = i64::MAX as usize;
        assert_eq!(i64_to_usize(i64::MAX), expected);
        assert_eq!(i64_to_usize(i64::MIN), 0);
    }

    #[test]
    fn parse_activity_type_known_variants() {
        let cases = [
            ("Creation", ActivityType::Creation),
            ("Import", ActivityType::Import),
            ("Extraction", ActivityType::Extraction),
            ("Generation", ActivityType::Generation),
            ("Transformation", ActivityType::Transformation),
            ("Derivation", ActivityType::Derivation),
            ("Aggregation", ActivityType::Aggregation),
            ("Filtering", ActivityType::Filtering),
            ("Merge", ActivityType::Merge),
            ("Split", ActivityType::Split),
            ("Analysis", ActivityType::Analysis),
            ("Computation", ActivityType::Computation),
            ("Simulation", ActivityType::Simulation),
            ("MachineLearning", ActivityType::MachineLearning),
            ("Inference", ActivityType::Inference),
            ("Experiment", ActivityType::Experiment),
            ("Observation", ActivityType::Observation),
            ("Measurement", ActivityType::Measurement),
            ("Validation", ActivityType::Validation),
            ("Editing", ActivityType::Editing),
            ("Review", ActivityType::Review),
            ("Approval", ActivityType::Approval),
            ("Publication", ActivityType::Publication),
            ("SessionStart", ActivityType::SessionStart),
            ("SessionCommit", ActivityType::SessionCommit),
            ("SessionRollback", ActivityType::SessionRollback),
            ("SliceCheckout", ActivityType::SliceCheckout),
            ("SliceReturn", ActivityType::SliceReturn),
            ("CertificateMint", ActivityType::CertificateMint),
            ("CertificateTransfer", ActivityType::CertificateTransfer),
            ("CertificateLoan", ActivityType::CertificateLoan),
            ("CertificateReturn", ActivityType::CertificateReturn),
        ];
        for (s, expected) in cases {
            assert_eq!(parse_activity_type(s), expected, "mismatch for {s:?}");
        }
    }

    #[test]
    fn parse_activity_type_unknown_is_custom() {
        let s = "https://example.com/types/Foo";
        match parse_activity_type(s) {
            ActivityType::Custom { type_uri } => assert_eq!(type_uri, s),
            other => panic!("expected Custom, got {other:?}"),
        }
    }

    proptest! {
        #[test]
        fn parse_activity_type_never_panics(s in any::<String>()) {
            let _ = parse_activity_type(&s);
        }
    }

    #[test]
    fn legacy_signature_empty_object_is_unsigned_marker() {
        let v = serde_json::json!({});
        let w = legacy_signature_to_witness(&v);
        assert_eq!(w.kind, WITNESS_KIND_MARKER);
        assert_eq!(w.evidence, "");
        assert_eq!(w.witnessed_at, 0);
        assert_eq!(w.encoding, WITNESS_ENCODING_NONE);
        assert_eq!(w.algorithm, None);
        assert_eq!(w.tier.as_deref(), Some(WITNESS_TIER_OPEN));
        assert_eq!(w.agent.as_str(), "");
    }

    #[test]
    fn legacy_signature_null_is_unsigned_marker() {
        let v = serde_json::Value::Null;
        let w = legacy_signature_to_witness(&v);
        assert_eq!(w.kind, WITNESS_KIND_MARKER);
        assert_eq!(w.encoding, WITNESS_ENCODING_NONE);
    }

    #[test]
    fn legacy_signature_signed_ed25519() {
        let v = serde_json::json!({
            "type": "Ed25519Signature2020",
            "proof_value": "c2lnZWQ=",
            "verification_method": "did:example:alice#key-1",
            "created": 1_700_000_000_u64
        });
        let w = legacy_signature_to_witness(&v);
        assert_eq!(w.kind, WITNESS_KIND_SIGNATURE);
        assert_eq!(w.evidence, "c2lnZWQ=");
        assert_eq!(w.witnessed_at, 1_700_000_000);
        assert_eq!(w.encoding, WITNESS_ENCODING_BASE64);
        assert_eq!(w.algorithm.as_deref(), Some(WITNESS_ALGORITHM_ED25519));
        assert_eq!(w.tier.as_deref(), Some(WITNESS_TIER_LOCAL));
        assert_eq!(w.agent.as_str(), "did:example:alice");
    }

    #[test]
    fn legacy_signature_missing_fields_use_defaults() {
        let v = serde_json::json!({"type": "Ed25519Signature2020"});
        let w = legacy_signature_to_witness(&v);
        assert_eq!(w.kind, WITNESS_KIND_MARKER);
        assert_eq!(w.evidence, "");
        assert_eq!(w.witnessed_at, 0);
        assert_eq!(w.encoding, WITNESS_ENCODING_NONE);
        assert_eq!(w.algorithm.as_deref(), Some(WITNESS_ALGORITHM_ED25519));
        assert_eq!(w.tier.as_deref(), Some(WITNESS_TIER_OPEN));
    }
}
