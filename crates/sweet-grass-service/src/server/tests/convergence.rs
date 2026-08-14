// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use super::*;
use tarpc::context;

#[tokio::test]
async fn test_convergence_check_missing_braid() {
    let server = make_server();
    let result = server
        .convergence_check(
            context::current(),
            "sha256:missing-convergence".to_string().into(),
        )
        .await
        .unwrap();

    assert_eq!(result["data_hash"], "sha256:missing-convergence");
    assert_eq!(result["converged"], false);
    assert_eq!(result["depth"], 0);
}

#[tokio::test]
async fn test_convergence_check_with_braid() {
    let server = make_server();
    let braid = create_test_braid(&server).await;

    let result = server
        .convergence_check(context::current(), braid.data_hash.clone())
        .await
        .unwrap();

    assert_eq!(result["data_hash"], braid.data_hash.as_str());
    assert_eq!(result["converged"], false);
    assert!(result["depth"].as_u64().is_some_and(|d| d > 0));
}

#[tokio::test]
async fn test_convergence_pressure_empty_store() {
    let server = make_server();
    let result = server
        .convergence_pressure(context::current(), 10)
        .await
        .unwrap();

    assert_eq!(result["total_scanned"], 0);
    assert_eq!(result["converged"], 0);
    assert_eq!(result["pressure"], 0.0);
    assert_eq!(result["throttle"], false);
}

#[tokio::test]
async fn test_convergence_pressure_unconverged_braids() {
    let server = make_server();
    create_test_braid(&server).await;
    create_test_braid(&server).await;

    let result = server
        .convergence_pressure(context::current(), 10)
        .await
        .unwrap();

    assert_eq!(result["total_scanned"], 2);
    assert_eq!(result["converged"], 0);
    assert_eq!(result["pressure"], 1.0);
    assert_eq!(result["throttle"], true);
}
