// SPDX-License-Identifier: AGPL-3.0-only

#![allow(
    clippy::float_cmp,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::clone_on_ref_ptr
)]

use super::*;
use sweet_grass_compression::{SessionOutcome, SessionVertex};
use sweet_grass_core::agent::Did;
use tarpc::context;

fn make_server() -> SweetGrassServer {
    let store = Arc::new(MemoryStore::new());
    let did = Did::new("did:key:z6MkTest");
    let factory = Arc::new(BraidFactory::new(did));
    let query = Arc::new(QueryEngine::new(store.clone()));
    let compression = Arc::new(CompressionEngine::new(factory.clone()));
    let attribution = Arc::new(AttributionCalculator::new());

    SweetGrassServer::new(store, factory, query, compression, attribution)
}

use std::sync::atomic::{AtomicU64, Ordering};
static COUNTER: AtomicU64 = AtomicU64::new(0);

async fn create_test_braid(server: &SweetGrassServer) -> Braid {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let request = CreateBraidRequest {
        data_hash: format!("sha256:test{id}").into(),
        mime_type: "text/plain".to_string(),
        size: 1024,
        attributed_to: Did::new("did:key:z6MkTest"),
        activity: None,
        derived_from: vec![],
        metadata: None,
    };
    server
        .clone()
        .create_braid(context::current(), request)
        .await
        .unwrap()
}

#[tokio::test]
async fn test_health_check() {
    let server = make_server();
    let status = server.health_check(context::current()).await.unwrap();
    assert_eq!(status.status, "UP");
    assert_eq!(status.braid_count, 0);
}

#[tokio::test]
async fn test_status() {
    let server = make_server();
    let status = server.status(context::current()).await.unwrap();
    assert!(status.healthy);
    assert_eq!(status.store_type, "memory");
    assert_eq!(status.braid_count, 0);
}

#[tokio::test]
async fn test_create_and_get_braid() {
    let server = make_server();

    let request = CreateBraidRequest {
        data_hash: "sha256:abc123".to_string().into(),
        mime_type: "text/plain".to_string(),
        size: 1024,
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

    assert_eq!(braid.data_hash.as_str(), "sha256:abc123");

    let retrieved = server
        .get_braid(context::current(), braid.id.clone())
        .await
        .unwrap();

    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().data_hash.as_str(), "sha256:abc123");
}

#[tokio::test]
async fn test_get_braid_not_found() {
    let server = make_server();
    let result = server
        .get_braid(context::current(), BraidId::new())
        .await
        .unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_get_braid_by_hash() {
    let server = make_server();
    let braid = create_test_braid(&server).await;

    let retrieved = server
        .clone()
        .get_braid_by_hash(context::current(), braid.data_hash.clone())
        .await
        .unwrap();

    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, braid.id);
}

#[tokio::test]
async fn test_get_braid_by_hash_not_found() {
    let server = make_server();
    let result = server
        .get_braid_by_hash(context::current(), "sha256:nonexistent".to_string().into())
        .await
        .unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_query_braids() {
    let server = make_server();
    create_test_braid(&server).await;
    create_test_braid(&server).await;

    let result = server
        .query_braids(
            context::current(),
            QueryFilter::new(),
            QueryOrder::NewestFirst,
        )
        .await
        .unwrap();

    assert_eq!(result.total_count, 2);
    assert_eq!(result.braids.len(), 2);
}

#[tokio::test]
async fn test_query_braids_with_filter() {
    let server = make_server();
    let braid = create_test_braid(&server).await;

    let filter = QueryFilter::new().with_hash(braid.data_hash.clone());
    let result = server
        .query_braids(context::current(), filter, QueryOrder::NewestFirst)
        .await
        .unwrap();

    assert_eq!(result.total_count, 1);
}

#[tokio::test]
async fn test_delete_braid() {
    let server = make_server();
    let braid = create_test_braid(&server).await;

    let deleted = server
        .clone()
        .delete_braid(context::current(), braid.id.clone())
        .await
        .unwrap();

    assert!(deleted);

    let retrieved = server
        .get_braid(context::current(), braid.id)
        .await
        .unwrap();
    assert!(retrieved.is_none());
}

#[tokio::test]
async fn test_braids_by_agent() {
    let server = make_server();
    create_test_braid(&server).await;

    let agent = Did::new("did:key:z6MkTest");
    let braids = server
        .braids_by_agent(context::current(), agent)
        .await
        .unwrap();

    assert_eq!(braids.len(), 1);
}

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

    // Chain was created successfully
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

    // Should have at least one contributor
    assert!(!rewards.is_empty());
    // Total should sum close to 100
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
async fn test_agent_contributions() {
    let server = make_server();
    create_test_braid(&server).await;
    create_test_braid(&server).await;

    let agent = Did::new("did:key:z6MkTest");
    let contributions = server
        .agent_contributions(context::current(), agent.clone(), None)
        .await
        .unwrap();

    assert_eq!(contributions.agent, agent);
    assert_eq!(contributions.total_count, 2);
    assert_eq!(contributions.braids.len(), 2);
}

#[tokio::test]
async fn test_compress_session() {
    let server = make_server();

    let mut session = Session::new("test-session");
    session.outcome = SessionOutcome::Committed;
    session.add_vertex(
        SessionVertex::new(
            "v1",
            "sha256:test",
            "text/plain",
            Did::new("did:key:z6MkTest"),
        )
        .with_size(100)
        .committed(),
    );

    let result = server
        .compress_session(context::current(), session)
        .await
        .unwrap();

    // Should produce some result
    assert!(result.has_braids() || result.discard_reason().is_some());
}

#[tokio::test]
async fn test_create_meta_braid() {
    let server = make_server();
    let braid1 = create_test_braid(&server).await;
    let braid2 = create_test_braid(&server).await;

    let meta = server
        .create_meta_braid(
            context::current(),
            vec![braid1.id, braid2.id],
            SummaryType::Session {
                session_id: "test-session".to_string(),
            },
        )
        .await
        .unwrap();

    assert!(matches!(
        meta.braid_type,
        sweet_grass_core::BraidType::Collection { .. }
    ));
}

#[tokio::test]
async fn test_provenance_graph() {
    let server = make_server();
    let braid = create_test_braid(&server).await;

    let entity = EntityReference::by_hash(&braid.data_hash);
    let graph = server
        .provenance_graph(context::current(), entity, 5, true)
        .await
        .unwrap();

    assert!(!graph.entities.is_empty());
}

#[tokio::test]
async fn test_export_provo() {
    let server = make_server();
    let braid = create_test_braid(&server).await;

    let doc = server
        .clone()
        .export_provo(context::current(), braid.data_hash.clone())
        .await
        .unwrap();

    assert!(doc.content.get("@context").is_some());
}

#[tokio::test]
async fn test_export_provo_not_found() {
    let server = make_server();

    let result = server
        .export_provo(context::current(), "sha256:nonexistent".to_string().into())
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
    // Shares should be descending
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
async fn test_export_graph_provo() {
    let server = make_server();
    let braid = create_test_braid(&server).await;

    let entity = EntityReference::by_hash(&braid.data_hash);
    let doc = server
        .export_graph_provo(context::current(), entity, 5)
        .await
        .unwrap();

    assert!(doc.content.get("@context").is_some());
    assert!(doc.content.get("@graph").is_some());
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

#[tokio::test]
async fn test_agent_contributions_with_time_range() {
    let server = make_server();
    create_test_braid(&server).await;

    let agent = Did::new("did:key:z6MkTest");
    let range = TimeRange {
        start: 0,
        end: u64::MAX,
    };
    let contributions = server
        .agent_contributions(context::current(), agent, Some(range))
        .await
        .unwrap();

    assert_eq!(contributions.total_count, 1);
}

#[tokio::test]
async fn test_agent_contributions_empty_time_range() {
    let server = make_server();
    create_test_braid(&server).await;

    let agent = Did::new("did:key:z6MkTest");
    let range = TimeRange { start: 0, end: 0 };
    let contributions = server
        .agent_contributions(context::current(), agent, Some(range))
        .await
        .unwrap();

    assert_eq!(contributions.total_count, 0);
}
