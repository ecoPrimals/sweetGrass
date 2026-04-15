// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

use std::sync::atomic::Ordering;

use sweet_grass_core::agent::Did;
use sweet_grass_store::{QueryFilter, QueryOrder};
use tarpc::context;
use tarpc::serde_transport::tcp;
use tarpc::tokio_serde::formats::Bincode;

use crate::rpc::{CreateBraidRequest, SweetGrassRpcClient};
use crate::server::start_tarpc_server;
use crate::server::tests::{COUNTER, TEST_BIND_ADDR, make_server};

#[tokio::test]
async fn test_start_tarpc_server_binds_and_accepts() {
    let listener = std::net::TcpListener::bind(TEST_BIND_ADDR).expect("bind");
    let addr = listener.local_addr().expect("local_addr");
    drop(listener);

    let server = make_server();
    let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let server_handle =
        tokio::spawn(async move { start_tarpc_server(addr, server, shutdown_rx).await });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let transport = tcp::connect(addr, Bincode::default).await.expect("connect");
    let client = SweetGrassRpcClient::new(tarpc::client::Config::default(), transport).spawn();

    let status = client
        .health_check(context::current())
        .await
        .expect("tarpc transport")
        .expect("rpc call");
    assert_eq!(status.status, "UP");

    server_handle.abort();
}

#[tokio::test]
async fn test_tarpc_roundtrip_braid_crud() {
    let listener = std::net::TcpListener::bind(TEST_BIND_ADDR).expect("bind");
    let addr = listener.local_addr().expect("local_addr");
    drop(listener);

    let server = make_server();
    let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let server_handle =
        tokio::spawn(async move { start_tarpc_server(addr, server, shutdown_rx).await });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let transport = tcp::connect(addr, Bincode::default).await.expect("connect");
    let client = SweetGrassRpcClient::new(tarpc::client::Config::default(), transport).spawn();

    let request = CreateBraidRequest {
        data_hash: format!("sha256:tarpc{}", COUNTER.fetch_add(1, Ordering::SeqCst)).into(),
        mime_type: "text/plain".to_string(),
        size: 1024,
        attributed_to: Did::new("did:key:z6MkTest"),
        activity: None,
        derived_from: vec![],
        metadata: None,
    };

    let braid = client
        .create_braid(context::current(), request)
        .await
        .expect("tarpc transport")
        .expect("rpc call");

    let retrieved = client
        .get_braid(context::current(), braid.id.clone())
        .await
        .expect("tarpc transport")
        .expect("rpc call");
    assert_eq!(retrieved.as_ref().map(|b| &b.id), Some(&braid.id));

    let query_result = client
        .query_braids(
            context::current(),
            QueryFilter::new(),
            QueryOrder::NewestFirst,
        )
        .await
        .expect("tarpc transport")
        .expect("rpc call");
    assert!(query_result.total_count >= 1);

    let deleted = client
        .delete_braid(context::current(), braid.id.clone())
        .await
        .expect("tarpc transport")
        .expect("rpc call");
    assert!(deleted);

    let after = client
        .get_braid(context::current(), braid.id)
        .await
        .expect("tarpc transport")
        .expect("rpc call");
    assert!(after.is_none());

    server_handle.abort();
}

#[tokio::test]
async fn test_tarpc_health_liveness_and_readiness() {
    let listener = std::net::TcpListener::bind(TEST_BIND_ADDR).expect("bind");
    let addr = listener.local_addr().expect("local_addr");
    drop(listener);

    let server = make_server();
    let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let server_handle =
        tokio::spawn(async move { start_tarpc_server(addr, server, shutdown_rx).await });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let transport = tcp::connect(addr, Bincode::default).await.expect("connect");
    let client = SweetGrassRpcClient::new(tarpc::client::Config::default(), transport).spawn();

    let live = client
        .health_liveness(context::current())
        .await
        .expect("tarpc transport");
    assert!(live);

    let ready = client
        .health_readiness(context::current())
        .await
        .expect("tarpc transport");
    assert!(ready);

    server_handle.abort();
}

#[tokio::test]
async fn test_tarpc_status() {
    let listener = std::net::TcpListener::bind(TEST_BIND_ADDR).expect("bind");
    let addr = listener.local_addr().expect("local_addr");
    drop(listener);

    let server = make_server();
    let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let server_handle =
        tokio::spawn(async move { start_tarpc_server(addr, server, shutdown_rx).await });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let transport = tcp::connect(addr, Bincode::default).await.expect("connect");
    let client = SweetGrassRpcClient::new(tarpc::client::Config::default(), transport).spawn();

    let svc = client
        .status(context::current())
        .await
        .expect("tarpc transport")
        .expect("rpc call");

    assert!(svc.healthy);
    assert_eq!(svc.store_type, "memory");
    assert_eq!(svc.braid_count, 0);
    assert_eq!(svc.version, env!("CARGO_PKG_VERSION"));

    server_handle.abort();
}

#[tokio::test]
async fn test_start_tarpc_server_shutdown_exits() {
    let listener = std::net::TcpListener::bind(TEST_BIND_ADDR).expect("bind");
    let addr = listener.local_addr().expect("local_addr");
    drop(listener);

    let server = make_server();
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let server_handle =
        tokio::spawn(async move { start_tarpc_server(addr, server, shutdown_rx).await });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    shutdown_tx.send(true).expect("signal shutdown");

    let join_result = tokio::time::timeout(std::time::Duration::from_secs(5), server_handle)
        .await
        .expect("server should exit within timeout");

    let server_result = join_result.expect("join server task");
    assert!(server_result.is_ok());
}
