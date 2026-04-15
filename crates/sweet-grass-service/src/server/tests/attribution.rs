// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use super::*;

#[tokio::test]
async fn test_attribution_chain() {
    let server = make_server();
    let braid = create_test_braid(&server).await;

    let chain = server
        .attribution_chain(
            context::current(),
            braid.data_hash.clone(),
            AttributionConfig::default(),
        )
        .await
        .unwrap();

    assert!(!chain.contributors.is_empty());
}

#[tokio::test]
async fn test_attribution_chain_not_found() {
    let server = make_server();

    let result = server
        .attribution_chain(
            context::current(),
            "sha256:nonexistent".to_string().into(),
            AttributionConfig::default(),
        )
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_calculate_rewards() {
    let server = make_server();
    let braid = create_test_braid(&server).await;

    let rewards = server
        .calculate_rewards(context::current(), braid.data_hash.clone(), 100.0)
        .await
        .unwrap();

    assert!(!rewards.is_empty());
    let total: f64 = rewards.iter().map(|r| r.amount).sum();
    assert!((total - 100.0).abs() < 0.01);
}

#[tokio::test]
async fn test_calculate_rewards_not_found() {
    let server = make_server();

    let result = server
        .calculate_rewards(
            context::current(),
            "sha256:nonexistent".to_string().into(),
            100.0,
        )
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_top_contributors() {
    let server = make_server();
    let braid = create_test_braid(&server).await;

    let contributors = server
        .clone()
        .top_contributors(context::current(), braid.data_hash.clone(), 5)
        .await
        .unwrap();

    assert!(!contributors.is_empty());
    for w in contributors.windows(2) {
        assert!(w[0].share >= w[1].share);
    }
}

#[tokio::test]
async fn test_top_contributors_not_found() {
    let server = make_server();
    let result = server
        .top_contributors(
            context::current(),
            "sha256:nonexistent".to_string().into(),
            10,
        )
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_top_contributors_limit() {
    let server = make_server();
    let braid = create_test_braid(&server).await;

    let contributors = server
        .top_contributors(context::current(), braid.data_hash.clone(), 1)
        .await
        .unwrap();

    assert!(contributors.len() <= 1);
}

#[tokio::test]
async fn test_anchor_braid() {
    let server = make_server();
    let hex = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    let request = CreateBraidRequest {
        data_hash: format!("sha256:{hex}").into(),
        mime_type: "application/octet-stream".to_string(),
        size: 0,
        attributed_to: Did::new("did:key:z6MkTest"),
        activity: None,
        derived_from: vec![],
        metadata: None,
    };
    let braid = server
        .clone()
        .create_braid(context::current(), request)
        .await
        .unwrap();

    let result = server
        .anchor_braid(context::current(), braid.id.clone(), "main".to_string())
        .await
        .unwrap();

    assert_eq!(result["spine_id"], "main");
    assert_eq!(result["anchored"], false);
    assert_eq!(result["status"], "prepared");
    assert!(result["content_hash"].is_string());
}

#[tokio::test]
async fn test_anchor_braid_not_found() {
    let server = make_server();
    let result = server
        .anchor_braid(context::current(), BraidId::new(), "main".to_string())
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_anchor_braid_non_sha256_hash() {
    let server = make_server();
    let braid = create_test_braid(&server).await;

    let result = server
        .anchor_braid(context::current(), braid.id.clone(), "main".to_string())
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_verify_anchor_exists() {
    let server = make_server();
    let braid = create_test_braid(&server).await;

    let result = server
        .verify_anchor(context::current(), braid.id.clone())
        .await
        .unwrap();

    assert_eq!(result["anchored"], false);
    assert_eq!(result["verification_status"], "pending_integration");
}

#[tokio::test]
async fn test_verify_anchor_not_found() {
    let server = make_server();
    let result = server
        .verify_anchor(context::current(), BraidId::new())
        .await;
    assert!(result.is_err());
}
