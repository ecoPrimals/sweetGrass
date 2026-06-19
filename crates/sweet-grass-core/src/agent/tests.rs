// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use super::*;

fn assert_agent_type_bincode_roundtrip(original: &AgentType) {
    let bytes = bincode::serialize(original).unwrap();
    let decoded: AgentType = bincode::deserialize(&bytes).unwrap();
    assert_eq!(&decoded, original);
}

#[test]
fn agent_type_bincode_roundtrip_person() {
    assert_agent_type_bincode_roundtrip(&AgentType::Person { name: None });
    assert_agent_type_bincode_roundtrip(&AgentType::Person {
        name: Some("Bincode Person".to_string()),
    });
}

#[test]
fn agent_type_bincode_roundtrip_software_agent() {
    assert_agent_type_bincode_roundtrip(&AgentType::SoftwareAgent {
        software_name: "SweetGrass".to_string(),
        version: "0.7.27".to_string(),
    });
}

#[test]
fn agent_type_bincode_roundtrip_organization() {
    assert_agent_type_bincode_roundtrip(&AgentType::Organization {
        name: "ecoPrimals".to_string(),
        org_type: None,
    });
    assert_agent_type_bincode_roundtrip(&AgentType::Organization {
        name: "Acme".to_string(),
        org_type: Some("Corporation".to_string()),
    });
}

#[test]
fn agent_type_bincode_roundtrip_device() {
    assert_agent_type_bincode_roundtrip(&AgentType::Device {
        device_type: "sensor".to_string(),
        device_id: None,
    });
    assert_agent_type_bincode_roundtrip(&AgentType::Device {
        device_type: "edge".to_string(),
        device_id: Some("device-99".to_string()),
    });
}

#[test]
fn test_did_creation() {
    let did = Did::new("did:key:z6MkTest123");
    assert!(did.is_valid());
    assert_eq!(did.method(), Some("key"));
}

#[test]
fn test_did_invalid() {
    let did = Did::new("not-a-did");
    assert!(!did.is_valid());
    assert_eq!(did.method(), None);
}

#[test]
fn test_did_from_string() {
    let did: Did = "did:web:example.com".into();
    assert!(did.is_valid());
    assert_eq!(did.method(), Some("web"));
}

#[test]
fn test_agent_role_weights() {
    let epsilon = f64::EPSILON;
    assert!((AgentRole::Creator.default_weight() - 1.0).abs() < epsilon);
    assert!((AgentRole::Contributor.default_weight() - 0.5).abs() < epsilon);
    assert!((AgentRole::ComputeProvider.default_weight() - 0.3).abs() < epsilon);
}

#[test]
fn test_agent_association() {
    let did = Did::new("did:key:z6MkTest");
    let principal = Did::new("did:key:z6MkPrincipal");
    let principal_check = principal.clone();

    let assoc = AgentAssociation::new(did, AgentRole::Creator).on_behalf_of(principal);

    assert!(assoc.is_delegated());
    assert_eq!(assoc.on_behalf_of, Some(principal_check));
}

#[test]
fn test_agent_person() {
    let did = Did::new("did:key:z6MkTest");
    let agent = Agent::person(did.clone(), Some("Alice".to_string()));

    assert_eq!(agent.id, did);
    assert_eq!(agent.name, Some("Alice".to_string()));
    assert!(matches!(agent.agent_type, AgentType::Person { .. }));
}

#[test]
fn test_agent_software() {
    let did = Did::new("did:key:z6MkBot");
    let agent = Agent::software(did, "SweetGrass", "0.1.0");

    assert_eq!(agent.name, Some("SweetGrass".to_string()));
    assert!(matches!(
        agent.agent_type,
        AgentType::SoftwareAgent { software_name, version }
        if software_name == "SweetGrass" && version == "0.1.0"
    ));
}

#[test]
fn test_agent_serialization() {
    let did = Did::new("did:key:z6MkTest");
    let agent = Agent::person(did, Some("Bob".to_string()));

    let json = serde_json::to_string(&agent).expect("should serialize");
    assert!(json.contains("@id"));
    assert!(json.contains("Person"));

    let parsed: Agent = serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(parsed.name, Some("Bob".to_string()));
}

#[test]
fn test_did_as_str() {
    let did = Did::new("did:key:z6MkHello");
    assert_eq!(did.as_str(), "did:key:z6MkHello");
}

#[test]
fn test_did_display() {
    let did = Did::new("did:key:z6MkDisplay");
    assert_eq!(format!("{did}"), "did:key:z6MkDisplay");
}

#[test]
fn test_did_from_owned_string() {
    let did = Did::from("did:web:example.com".to_string());
    assert!(did.is_valid());
    assert_eq!(did.as_str(), "did:web:example.com");
}

#[test]
fn test_did_roundtrip_json() {
    let did = Did::new("did:key:z6MkRoundtrip");
    let json = serde_json::to_string(&did).expect("serialize");
    let parsed: Did = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed, did);
}

#[test]
fn test_agent_type_default() {
    let default = AgentType::default();
    assert!(matches!(default, AgentType::Person { name: None }));
}

#[test]
fn test_agent_role_display_custom() {
    let custom = AgentRole::Custom("MyRole".to_string());
    assert_eq!(format!("{custom}"), "MyRole");
}

#[test]
fn test_agent_role_display_standard() {
    assert_eq!(format!("{}", AgentRole::Creator), "Creator");
    assert_eq!(format!("{}", AgentRole::Contributor), "Contributor");
    assert_eq!(format!("{}", AgentRole::Publisher), "Publisher");
    assert_eq!(format!("{}", AgentRole::Validator), "Validator");
    assert_eq!(format!("{}", AgentRole::DataProvider), "DataProvider");
    assert_eq!(format!("{}", AgentRole::ComputeProvider), "ComputeProvider");
    assert_eq!(format!("{}", AgentRole::StorageProvider), "StorageProvider");
    assert_eq!(format!("{}", AgentRole::Orchestrator), "Orchestrator");
    assert_eq!(format!("{}", AgentRole::Curator), "Curator");
    assert_eq!(format!("{}", AgentRole::Transformer), "Transformer");
    assert_eq!(format!("{}", AgentRole::Owner), "Owner");
}

#[test]
fn test_agent_role_all_weights() {
    let epsilon = f64::EPSILON;
    assert!((AgentRole::Publisher.default_weight() - 0.1).abs() < epsilon);
    assert!((AgentRole::Validator.default_weight() - 0.1).abs() < epsilon);
    assert!((AgentRole::DataProvider.default_weight() - 0.4).abs() < epsilon);
    assert!((AgentRole::Transformer.default_weight() - 0.3).abs() < epsilon);
    assert!((AgentRole::StorageProvider.default_weight() - 0.2).abs() < epsilon);
    assert!((AgentRole::Curator.default_weight() - 0.2).abs() < epsilon);
    assert!((AgentRole::Orchestrator.default_weight() - 0.15).abs() < epsilon);
    assert!((AgentRole::Owner.default_weight() - 0.8).abs() < epsilon);
    assert!((AgentRole::Custom("x".to_string()).default_weight() - 0.2).abs() < epsilon);
}

#[test]
fn test_agent_association_with_plan() {
    let did = Did::new("did:key:z6MkPlanner");
    let assoc = AgentAssociation::new(did, AgentRole::Orchestrator).with_plan("protocol-v2");
    assert_eq!(assoc.had_plan, Some("protocol-v2".to_string()));
    assert!(!assoc.is_delegated());
}

#[test]
fn test_agent_organization() {
    let did = Did::new("did:web:orgexample.com");
    let agent = Agent::organization(did.clone(), "Test Org");
    assert_eq!(agent.id, did);
    assert_eq!(agent.name, Some("Test Org".to_string()));
    assert!(matches!(
        agent.agent_type,
        AgentType::Organization { name, org_type: None } if name == "Test Org"
    ));
}

#[test]
fn test_agent_type_device() {
    let agent_type = AgentType::Device {
        device_type: "sensor".to_string(),
        device_id: Some("sensor-42".to_string()),
    };
    let json = serde_json::to_string(&agent_type).expect("serialize");
    assert!(json.contains("Device"));
    let parsed: AgentType = serde_json::from_str(&json).expect("deserialize");
    assert!(matches!(parsed, AgentType::Device { .. }));
}

#[test]
fn test_agent_type_software_agent_json_roundtrip() {
    let agent_type = AgentType::SoftwareAgent {
        software_name: "SweetGrass".to_string(),
        version: "0.7.27".to_string(),
    };
    let json = serde_json::to_string(&agent_type).expect("serialize");
    let parsed: AgentType = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed, agent_type);
}

#[test]
fn test_agent_type_organization_json_roundtrip() {
    let agent_type = AgentType::Organization {
        name: "ecoPrimals".to_string(),
        org_type: Some("Foundation".to_string()),
    };
    let json = serde_json::to_string(&agent_type).expect("serialize");
    let parsed: AgentType = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed, agent_type);
}
