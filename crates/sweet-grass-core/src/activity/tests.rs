// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

#![expect(
    clippy::expect_used,
    reason = "test module: expect is standard in tests"
)]

use super::*;
use crate::agent::{AgentRole, Did};

#[test]
fn test_activity_id_generation() {
    let id1 = ActivityId::new();
    let id2 = ActivityId::new();
    assert_ne!(id1, id2);
    assert!(id1.as_str().starts_with("urn:activity:uuid:"));
}

#[test]
fn test_activity_builder() {
    let did = Did::new("did:key:z6MkTest");
    let activity = Activity::builder(ActivityType::Computation)
        .associated_with(AgentAssociation::new(did.clone(), AgentRole::Creator))
        .compute_units(1.5)
        .build();

    assert_eq!(activity.activity_type, ActivityType::Computation);
    assert_eq!(activity.ecop.compute_units, Some(1.5));
    assert_eq!(activity.primary_agent(), Some(&did));
}

#[test]
fn test_activity_duration() {
    let activity = Activity::builder(ActivityType::Analysis)
        .started_at(1000)
        .ended_at(2000)
        .build();

    assert_eq!(activity.duration_ns(), Some(1000));
    assert!(activity.is_complete());
}

#[test]
fn test_activity_incomplete() {
    let activity = Activity::builder(ActivityType::Creation).build();
    assert!(!activity.is_complete());
    assert!(activity.duration_ns().is_none());
}

#[test]
fn test_activity_serialization() {
    let activity = Activity::builder(ActivityType::Transformation)
        .compute_units(2.5)
        .build();

    let json = serde_json::to_string(&activity).expect("should serialize");
    assert!(json.contains("@id"));
    assert!(json.contains("Transformation"));

    let parsed: Activity = serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(parsed.activity_type, ActivityType::Transformation);
}

#[test]
fn test_activity_id_from_task() {
    let id = ActivityId::from_task("task-123");
    assert_eq!(id.as_str(), "urn:activity:task:task-123");
}

#[test]
fn test_activity_id_from_string() {
    let id = ActivityId::from_string("custom-id");
    assert_eq!(id.as_str(), "custom-id");
}

#[test]
fn test_activity_id_display() {
    let id = ActivityId::new();
    let display = format!("{id}");
    assert!(display.starts_with("urn:activity:uuid:"));
}

#[test]
fn test_activity_id_default() {
    let id = ActivityId::default();
    assert!(id.as_str().starts_with("urn:activity:uuid:"));
}

#[test]
fn test_activity_type_display() {
    assert_eq!(format!("{}", ActivityType::Creation), "Creation");
    assert_eq!(format!("{}", ActivityType::Computation), "Computation");
    assert_eq!(format!("{}", ActivityType::Import), "Import");
    assert_eq!(format!("{}", ActivityType::Derivation), "Derivation");
}

#[test]
fn test_used_entity() {
    let used = UsedEntity::new(EntityReference::by_hash("sha256:input"))
        .with_role(EntityRole::Input)
        .with_time(42);
    assert_eq!(used.time, Some(42));
    assert!(matches!(used.role, EntityRole::Input));
}

#[test]
fn test_activity_builder_uses() {
    let used = UsedEntity::new(EntityReference::by_hash("sha256:used"));
    let activity = Activity::builder(ActivityType::Derivation)
        .uses(used)
        .build();
    assert_eq!(activity.used.len(), 1);
}

#[test]
fn test_activity_builder_session_ref() {
    let activity = Activity::builder(ActivityType::Creation)
        .session_ref("session-42")
        .build();
    assert_eq!(activity.ecop.session_ref, Some("session-42".to_string()));
}

#[test]
fn test_activity_builder_compute_task() {
    let activity = Activity::builder(ActivityType::Computation)
        .compute_task("task-99")
        .build();
    assert_eq!(activity.ecop.compute_task, Some("task-99".to_string()));
}

#[test]
fn test_toadstool_task_alias_backward_compat() {
    let json = r#"{"@id":"urn:activity:uuid:test","@type":"Computation","used":[],"was_associated_with":[],"started_at_time":0,"metadata":{"parameters":{}},"ecop":{"toadstool_task":"old-task-1"}}"#;
    let activity: Activity = serde_json::from_str(json).expect("should deserialize with alias");
    assert_eq!(activity.ecop.compute_task, Some("old-task-1".to_string()));
}

#[test]
fn test_activity_id_from_str_ref() {
    let id: ActivityId = "urn:activity:custom:abc".into();
    assert_eq!(id.as_str(), "urn:activity:custom:abc");
}

#[test]
fn test_activity_id_from_owned_string() {
    let id: ActivityId = String::from("urn:activity:custom:owned").into();
    assert_eq!(id.as_str(), "urn:activity:custom:owned");
}

#[test]
fn test_activity_type_custom_display() {
    let custom = ActivityType::Custom {
        type_uri: "https://example.org/custom-activity".to_string(),
    };
    assert_eq!(format!("{custom}"), "https://example.org/custom-activity");
}
