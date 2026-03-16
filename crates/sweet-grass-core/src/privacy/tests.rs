// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

#![expect(
    clippy::expect_used,
    reason = "test module: expect is standard in tests"
)]

use super::*;

#[test]
fn test_privacy_level_default() {
    let level = PrivacyLevel::default();
    assert_eq!(level, PrivacyLevel::Public);
}

#[test]
fn test_privacy_metadata_builder() {
    let subject = Did::new("did:key:z6MkTest");

    let privacy = PrivacyMetadata::builder()
        .visibility(PrivacyLevel::Private)
        .retention(RetentionPolicy::Duration(DurationSecs(86400)))
        .consent_obtained(true)
        .grant_access(subject.clone())
        .restrict_processing(ProcessingType::Analytics)
        .allow_derivation(false)
        .build();

    assert_eq!(privacy.visibility, PrivacyLevel::Private);
    assert!(privacy.consent_obtained);
    assert!(!privacy.derivation_allowed);
    assert!(privacy.granted_access.contains(&subject));
    assert!(privacy.is_processing_restricted(&ProcessingType::Analytics));
}

#[test]
fn test_access_control() {
    let owner = Did::new("did:key:z6MkOwner");
    let granted = Did::new("did:key:z6MkGranted");
    let other = Did::new("did:key:z6MkOther");

    let privacy = PrivacyMetadata::builder()
        .visibility(PrivacyLevel::Private)
        .grant_access(granted.clone())
        .build();

    assert!(privacy.has_access(&owner, &owner));
    assert!(privacy.has_access(&granted, &owner));
    assert!(!privacy.has_access(&other, &owner));
}

#[test]
fn test_public_access() {
    let anyone = Did::new("did:key:z6MkAnyone");
    let owner = Did::new("did:key:z6MkOwner");

    let privacy = PrivacyMetadata::builder()
        .visibility(PrivacyLevel::Public)
        .build();

    assert!(privacy.has_access(&anyone, &owner));
}

#[test]
fn test_processing_restrictions() {
    let privacy = PrivacyMetadata::builder()
        .restrict_processing(ProcessingType::ThirdPartySharing)
        .build();

    assert!(privacy.is_processing_restricted(&ProcessingType::ThirdPartySharing));
    assert!(!privacy.is_processing_restricted(&ProcessingType::Attribution));
}

#[test]
fn test_restrict_all_processing() {
    let privacy = PrivacyMetadata::builder()
        .restrict_processing(ProcessingType::All)
        .build();

    assert!(privacy.is_processing_restricted(&ProcessingType::Attribution));
    assert!(privacy.is_processing_restricted(&ProcessingType::Analytics));
}

#[test]
fn test_data_subject_request_serialization() {
    let request = DataSubjectRequest::Access {
        subject: Did::new("did:key:z6MkTest"),
    };

    let json = serde_json::to_string(&request).expect("serialize");
    assert!(json.contains("Access"));

    let parsed: DataSubjectRequest = serde_json::from_str(&json).expect("deserialize");
    assert!(matches!(parsed, DataSubjectRequest::Access { .. }));
}

#[test]
fn test_erasure_request() {
    let request = DataSubjectRequest::Erasure {
        subject: Did::new("did:key:z6MkTest"),
        braid_ids: vec!["braid-1".to_string()],
        reason: ErasureReason::ConsentWithdrawn,
    };

    let json = serde_json::to_string(&request).expect("serialize");
    assert!(json.contains("ConsentWithdrawn"));
}

#[test]
fn test_duration_secs_conversion() {
    let duration = Duration::from_secs(3600);
    let secs: DurationSecs = duration.into();
    assert_eq!(secs.0, 3600);

    let back: Duration = secs.into();
    assert_eq!(back, duration);
}

#[test]
fn test_system_time_secs_from() {
    let now = SystemTime::now();
    let secs: SystemTimeSecs = now.into();
    assert!(secs.0 > 0);
}

#[test]
fn test_retention_policy_variants() {
    let until = RetentionPolicy::Until(SystemTimeSecs(1_700_000_000));
    let json = serde_json::to_string(&until).expect("serialize");
    assert!(json.contains("Until"));

    let orphaned = RetentionPolicy::UntilOrphaned;
    let json = serde_json::to_string(&orphaned).expect("serialize");
    assert!(json.contains("UntilOrphaned"));

    let hold = RetentionPolicy::LegalHold {
        reason: "ongoing case".to_string(),
        placed_at: SystemTimeSecs(1_700_000_000),
    };
    let json = serde_json::to_string(&hold).expect("serialize");
    let parsed: RetentionPolicy = serde_json::from_str(&json).expect("deserialize");
    assert!(matches!(parsed, RetentionPolicy::LegalHold { .. }));
}

#[test]
fn test_privacy_level_authenticated_access() {
    let privacy = PrivacyMetadata::builder()
        .visibility(PrivacyLevel::Authenticated)
        .build();
    let anyone = Did::new("did:key:z6MkAnyone");
    let owner = Did::new("did:key:z6MkOwner");
    assert!(privacy.has_access(&anyone, &owner));
}

#[test]
fn test_privacy_level_encrypted_access() {
    let granted = Did::new("did:key:z6MkGranted");
    let denied = Did::new("did:key:z6MkDenied");
    let owner = Did::new("did:key:z6MkOwner");

    let privacy = PrivacyMetadata::builder()
        .visibility(PrivacyLevel::Encrypted)
        .grant_access(granted.clone())
        .build();

    assert!(privacy.has_access(&owner, &owner));
    assert!(privacy.has_access(&granted, &owner));
    assert!(!privacy.has_access(&denied, &owner));
}

#[test]
fn test_privacy_level_anonymized_public() {
    let privacy = PrivacyMetadata::builder()
        .visibility(PrivacyLevel::AnonymizedPublic {
            anonymized_fields: vec!["agent_name".to_string()],
        })
        .build();
    let anyone = Did::new("did:key:z6MkAnyone");
    let owner = Did::new("did:key:z6MkOwner");
    assert!(privacy.has_access(&anyone, &owner));
}

#[test]
fn test_data_subject_request_rectification() {
    let request = DataSubjectRequest::Rectification {
        subject: Did::new("did:key:z6MkSubject"),
        braid_id: "braid-1".to_string(),
        corrections: vec![("field".to_string(), "new_value".to_string())],
    };
    let json = serde_json::to_string(&request).expect("serialize");
    assert!(json.contains("Rectification"));
}

#[test]
fn test_data_subject_request_portability() {
    let request = DataSubjectRequest::Portability {
        subject: Did::new("did:key:z6MkSubject"),
        format: ExportFormat::Csv,
    };
    let json = serde_json::to_string(&request).expect("serialize");
    assert!(json.contains("Csv"));
}

#[test]
fn test_data_subject_request_objection() {
    let request = DataSubjectRequest::Objection {
        subject: Did::new("did:key:z6MkSubject"),
        processing_type: ProcessingType::RewardCalculation,
    };
    let json = serde_json::to_string(&request).expect("serialize");
    assert!(json.contains("Objection"));
}

#[test]
fn test_erasure_reason_variants() {
    let reasons = [
        ErasureReason::NoLongerNecessary,
        ErasureReason::UnlawfulProcessing,
        ErasureReason::LegalObligation,
        ErasureReason::Other("custom reason".to_string()),
    ];
    for reason in &reasons {
        let json = serde_json::to_string(reason).expect("serialize");
        let parsed: ErasureReason = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(&parsed, reason);
    }
}

#[test]
fn test_export_format_default() {
    assert_eq!(ExportFormat::default(), ExportFormat::JsonLd);
}

#[test]
fn test_consent_details_serialization() {
    let details = ConsentDetails {
        obtained_at: SystemTimeSecs(1_700_000_000),
        mechanism: ConsentMechanism::ExplicitOptIn,
        policy_version: "1.0".to_string(),
        purposes: vec!["provenance".to_string(), "attribution".to_string()],
    };
    let json = serde_json::to_string(&details).expect("serialize");
    let parsed: ConsentDetails = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed.mechanism, ConsentMechanism::ExplicitOptIn);
    assert_eq!(parsed.purposes.len(), 2);
}

#[test]
fn test_privacy_builder_consent_details() {
    let details = ConsentDetails {
        obtained_at: SystemTimeSecs(1_700_000_000),
        mechanism: ConsentMechanism::ContractNecessity,
        policy_version: "2.0".to_string(),
        purposes: vec!["research".to_string()],
    };
    let privacy = PrivacyMetadata::builder().consent_details(details).build();
    assert!(privacy.consent_obtained);
    assert!(privacy.consent_details.is_some());
}

#[test]
fn test_privacy_metadata_builder_default() {
    let builder = PrivacyMetadataBuilder::default();
    let privacy = builder.build();
    assert_eq!(privacy.visibility, PrivacyLevel::Public);
    assert!(privacy.derivation_allowed);
}
