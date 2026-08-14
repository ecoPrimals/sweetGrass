// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use super::*;
use crate::rpc::{CreateBraidRequest, RpcError};
use sweet_grass_store::{QueryFilter, QueryOrder};
use tarpc::context;

#[tokio::test]
async fn test_create_braid_propagates_store_put_error() {
    let (server, fault) = make_fault_injection_server();
    fault.set_fail_puts(true);

    let request = CreateBraidRequest {
        data_hash: "sha256:store-error-put".to_string().into(),
        mime_type: "text/plain".to_string(),
        size: 128,
        attributed_to: Did::new("did:key:z6MkTest"),
        activity: None,
        derived_from: vec![],
        metadata: None,
    };

    let result = server.create_braid(context::current(), request).await;
    assert!(matches!(result, Err(RpcError::Store(_))));
}

#[tokio::test]
async fn test_get_braid_propagates_store_get_error() {
    let (server, fault) = make_fault_injection_server();
    fault.set_fail_gets(true);

    let result = server.get_braid(context::current(), BraidId::new()).await;
    assert!(matches!(result, Err(RpcError::Store(_))));
}

#[tokio::test]
async fn test_get_braid_by_hash_propagates_store_error() {
    let (server, fault) = make_fault_injection_server();
    fault.set_fail_gets(true);

    let result = server
        .get_braid_by_hash(context::current(), "sha256:missing".to_string().into())
        .await;
    assert!(matches!(result, Err(RpcError::Store(_))));
}

#[tokio::test]
async fn test_query_braids_propagates_store_error() {
    let (server, fault) = make_fault_injection_server();
    fault.set_fail_queries(true);

    let result = server
        .query_braids(
            context::current(),
            QueryFilter::new(),
            QueryOrder::NewestFirst,
        )
        .await;
    assert!(matches!(result, Err(RpcError::Store(_))));
}

#[tokio::test]
async fn test_braids_by_agent_propagates_store_error() {
    let (server, fault) = make_fault_injection_server();
    fault.set_fail_queries(true);

    let result = server
        .braids_by_agent(context::current(), Did::new("did:key:z6MkTest"))
        .await;
    assert!(matches!(result, Err(RpcError::Store(_))));
}

#[tokio::test]
async fn test_agent_contributions_propagates_store_error() {
    let (server, fault) = make_fault_injection_server();
    fault.set_fail_queries(true);

    let result = server
        .agent_contributions(context::current(), Did::new("did:key:z6MkTest"), None)
        .await;
    assert!(matches!(result, Err(RpcError::Store(_))));
}

#[tokio::test]
async fn test_convergence_check_propagates_store_error() {
    let (server, fault) = make_fault_injection_server();
    fault.set_fail_queries(true);

    let result = server
        .convergence_check(context::current(), "sha256:check".to_string().into())
        .await;
    assert!(matches!(result, Err(RpcError::Store(_))));
}

#[tokio::test]
async fn test_convergence_pressure_propagates_store_error() {
    let (server, fault) = make_fault_injection_server();
    fault.set_fail_queries(true);

    let result = server.convergence_pressure(context::current(), 5).await;
    assert!(matches!(result, Err(RpcError::Store(_))));
}
