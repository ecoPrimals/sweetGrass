// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "test module: expect/unwrap are standard in tests"
)]

use super::*;
use crate::session::{SessionOutcome, SessionVertex};
use sweet_grass_core::agent::Did;

fn make_factory() -> Arc<BraidFactory> {
    Arc::new(BraidFactory::new(Did::new("did:key:z6MkTestFactory")))
}

fn make_vertex(id: &str, hash: &str) -> SessionVertex {
    SessionVertex::new(id, hash, "application/json", Did::new("did:key:z6MkTest")).with_size(1024)
}

#[test]
fn test_empty_session_discarded() {
    let engine = CompressionEngine::new(make_factory());
    let session = Session::new("empty");

    let result = engine.compress(&session).expect("should compress");

    assert!(!result.has_braids());
    assert_eq!(result.count(), 0);
    assert!(matches!(
        result.discard_reason(),
        Some(DiscardReason::EmptySession)
    ));
}

#[test]
fn test_rollback_discarded() {
    let engine = CompressionEngine::new(make_factory());
    let mut session = Session::new("rollback");
    session.add_vertex(make_vertex("v1", "sha256:a").committed());
    session.finalize(SessionOutcome::Rollback);

    let result = engine.compress(&session).expect("should compress");

    assert!(!result.has_braids());
    assert!(matches!(
        result.discard_reason(),
        Some(DiscardReason::Rollback)
    ));
}

#[test]
fn test_exploratory_discarded() {
    let engine = CompressionEngine::new(make_factory());
    let mut session = Session::new("exploratory");
    session.add_vertex(make_vertex("v1", "sha256:a"));

    let result = engine.compress(&session).expect("should compress");

    assert!(!result.has_braids());
    assert!(matches!(
        result.discard_reason(),
        Some(DiscardReason::ExploratoryOnly)
    ));
}

#[test]
fn test_single_braid() {
    let engine = CompressionEngine::new(make_factory());
    let mut session = Session::new("single");
    session.add_vertex(make_vertex("v1", "sha256:a").committed());
    session.add_vertex(make_vertex("v2", "sha256:b").with_parent("v1").committed());
    session.finalize(SessionOutcome::Committed);

    let result = engine.compress(&session).expect("should compress");

    assert!(result.has_braids());
    assert_eq!(result.count(), 1);

    if let CompressionResult::Single(braid) = result {
        assert_eq!(braid.data_hash.as_str(), "sha256:b");
        assert!(braid.was_generated_by.is_some());
        assert!(!braid.was_derived_from.is_empty());
        assert!(braid.ecop.compression.is_some());
    } else {
        panic!("Expected Single result");
    }
}

#[test]
fn test_compression_metadata() {
    let engine = CompressionEngine::new(make_factory());
    let mut session = Session::new("metadata-test");
    session.compute_units = 2.5;
    session.add_vertex(make_vertex("v1", "sha256:root").committed());
    session.add_vertex(
        make_vertex("v2", "sha256:derived")
            .with_parent("v1")
            .committed(),
    );
    session.finalize(SessionOutcome::Committed);

    let result = engine.compress(&session).expect("should compress");

    if let CompressionResult::Single(braid) = result {
        let compression = braid.ecop.compression.unwrap();
        assert_eq!(compression.vertex_count, 2);
        assert_eq!(compression.branch_count, 0);
        assert!(compression.ratio > 0.0 && compression.ratio < 1.0);

        let activity = braid.was_generated_by.unwrap();
        assert_eq!(activity.ecop.compute_units, Some(2.5));
        assert_eq!(activity.ecop.session_ref, Some("metadata-test".to_string()));
    } else {
        panic!("Expected Single result");
    }
}

#[test]
fn test_result_braids_accessor() {
    let engine = CompressionEngine::new(make_factory());
    let mut session = Session::new("accessor");
    session.add_vertex(make_vertex("v1", "sha256:a").committed());
    session.finalize(SessionOutcome::Committed);

    let result = engine.compress(&session).expect("should compress");
    let braids = result.braids();

    assert_eq!(braids.len(), 1);
}

#[test]
fn test_with_config() {
    let config = CompressionConfig {
        split_threshold: 5,
        hierarchical_threshold: 10,
        generate_summaries: true,
        ..Default::default()
    };
    let engine = CompressionEngine::new(make_factory()).with_config(config);

    assert_eq!(engine.config().split_threshold, 5);
    assert!(engine.config().generate_summaries);
}

#[test]
fn test_multiple_result_braids() {
    let braid1 = Braid::builder()
        .data_hash("sha256:multi1")
        .mime_type("application/json")
        .size(100)
        .attributed_to(Did::new("did:key:z6MkTest"))
        .build()
        .expect("should build");

    let braid2 = Braid::builder()
        .data_hash("sha256:multi2")
        .mime_type("application/json")
        .size(200)
        .attributed_to(Did::new("did:key:z6MkTest"))
        .build()
        .expect("should build");

    let summary = Braid::builder()
        .data_hash("sha256:summary")
        .mime_type("application/json")
        .size(50)
        .attributed_to(Did::new("did:key:z6MkTest"))
        .build()
        .expect("should build");

    let result = CompressionResult::Multiple {
        braids: vec![braid1, braid2],
        summary: Some(summary),
    };

    assert!(result.has_braids());
    assert_eq!(result.count(), 3);
    assert_eq!(result.braids().len(), 3);
    assert!(result.discard_reason().is_none());
}

#[test]
fn test_multiple_result_without_summary() {
    let braid = Braid::builder()
        .data_hash("sha256:nosummary")
        .mime_type("application/json")
        .size(100)
        .attributed_to(Did::new("did:key:z6MkTest"))
        .build()
        .expect("should build");

    let result = CompressionResult::Multiple {
        braids: vec![braid],
        summary: None,
    };

    assert!(result.has_braids());
    assert_eq!(result.count(), 1);
    assert_eq!(result.braids().len(), 1);
}

#[test]
fn test_branching_session_produces_multiple() {
    let config = CompressionConfig {
        split_threshold: 3,
        generate_summaries: true,
        ..Default::default()
    };
    let engine = CompressionEngine::new(make_factory()).with_config(config);

    let mut session = Session::new("branching");
    session.add_vertex(make_vertex("root", "sha256:root").committed());
    session.add_vertex(
        make_vertex("b1-1", "sha256:b1-1")
            .with_parent("root")
            .committed(),
    );
    session.add_vertex(
        make_vertex("b1-2", "sha256:b1-2")
            .with_parent("b1-1")
            .committed(),
    );
    session.add_vertex(
        make_vertex("b2-1", "sha256:b2-1")
            .with_parent("root")
            .committed(),
    );
    session.add_vertex(
        make_vertex("b2-2", "sha256:b2-2")
            .with_parent("b2-1")
            .committed(),
    );
    session.finalize(SessionOutcome::Committed);

    let result = engine.compress(&session).expect("should compress");
    assert!(result.has_braids());
}

#[test]
fn test_deep_session() {
    let config = CompressionConfig {
        hierarchical_threshold: 3,
        generate_summaries: true,
        ..Default::default()
    };
    let engine = CompressionEngine::new(make_factory()).with_config(config);

    let mut session = Session::new("deep");
    session.add_vertex(make_vertex("v1", "sha256:l1").committed());
    session.add_vertex(make_vertex("v2", "sha256:l2").with_parent("v1").committed());
    session.add_vertex(make_vertex("v3", "sha256:l3").with_parent("v2").committed());
    session.add_vertex(make_vertex("v4", "sha256:l4").with_parent("v3").committed());
    session.finalize(SessionOutcome::Committed);

    let result = engine.compress(&session).expect("should compress");
    assert!(result.has_braids());
}

#[test]
fn test_session_with_compute_units() {
    let engine = CompressionEngine::new(make_factory());
    let mut session = Session::new("compute");
    let test_compute_units = 3.5;
    session.compute_units = test_compute_units;
    session.add_vertex(make_vertex("v1", "sha256:compute").committed());
    session.finalize(SessionOutcome::Committed);

    let result = engine.compress(&session).expect("should compress");

    if let CompressionResult::Single(braid) = result {
        let activity = braid.was_generated_by.expect("should have activity");
        assert_eq!(activity.ecop.compute_units, Some(test_compute_units));
    } else {
        panic!("Expected Single result");
    }
}

#[test]
fn test_with_source_propagates() {
    let engine = CompressionEngine::new(make_factory()).with_source("discovered-primal");
    let mut session = Session::new("source-test");
    session.add_vertex(make_vertex("v1", "sha256:src").committed());
    session.finalize(SessionOutcome::Committed);

    let result = engine.compress(&session).expect("should compress");
    if let CompressionResult::Single(braid) = result {
        assert_eq!(
            braid.ecop.source_primal.as_deref(),
            Some("discovered-primal")
        );
    } else {
        panic!("Expected Single result");
    }
}

#[test]
fn test_branching_with_summary_disabled() {
    let config = CompressionConfig {
        split_threshold: 3,
        generate_summaries: false,
        ..Default::default()
    };
    let engine = CompressionEngine::new(make_factory()).with_config(config);

    let mut session = Session::new("no-summary");
    session.add_vertex(make_vertex("root", "sha256:root").committed());
    session.add_vertex(
        make_vertex("b1-1", "sha256:b1-1")
            .with_parent("root")
            .committed(),
    );
    session.add_vertex(
        make_vertex("b2-1", "sha256:b2-1")
            .with_parent("root")
            .committed(),
    );
    session.add_vertex(
        make_vertex("b1-2", "sha256:b1-2")
            .with_parent("b1-1")
            .committed(),
    );
    session.add_vertex(
        make_vertex("b2-2", "sha256:b2-2")
            .with_parent("b2-1")
            .committed(),
    );
    session.finalize(SessionOutcome::Committed);

    let result = engine.compress(&session).expect("should compress");
    if let CompressionResult::Multiple { summary, .. } = &result {
        assert!(summary.is_none(), "summaries disabled");
    }
    assert!(result.has_braids());
}

#[test]
fn test_hierarchical_with_summary_disabled() {
    let config = CompressionConfig {
        hierarchical_threshold: 3,
        generate_summaries: false,
        ..Default::default()
    };
    let engine = CompressionEngine::new(make_factory()).with_config(config);

    let mut session = Session::new("hier-no-summary");
    session.add_vertex(make_vertex("v1", "sha256:h1").committed());
    session.add_vertex(make_vertex("v2", "sha256:h2").with_parent("v1").committed());
    session.add_vertex(make_vertex("v3", "sha256:h3").with_parent("v2").committed());
    session.add_vertex(make_vertex("v4", "sha256:h4").with_parent("v3").committed());
    session.finalize(SessionOutcome::Committed);

    let result = engine.compress(&session).expect("should compress");
    if let CompressionResult::Multiple { summary, .. } = &result {
        assert!(summary.is_none(), "summaries disabled");
    }
    assert!(result.has_braids());
}

#[test]
fn test_session_with_ended_at() {
    let engine = CompressionEngine::new(make_factory());
    let mut session = Session::new("ended");
    session.started_at = 1000;
    session.ended_at = Some(2000);
    session.add_vertex(make_vertex("v1", "sha256:ended").committed());
    session.finalize(SessionOutcome::Committed);

    let result = engine.compress(&session).expect("should compress");
    if let CompressionResult::Single(braid) = result {
        let activity = braid.was_generated_by.expect("should have activity");
        assert!(
            activity.ended_at_time.is_some(),
            "ended_at should propagate"
        );
    } else {
        panic!("Expected Single result");
    }
}

#[test]
fn test_compression_result_none_accessors() {
    let result = CompressionResult::None {
        reason: DiscardReason::EmptySession,
    };
    assert!(!result.has_braids());
    assert_eq!(result.count(), 0);
    assert!(result.braids().is_empty());
    assert!(matches!(
        result.discard_reason(),
        Some(DiscardReason::EmptySession)
    ));
}

#[test]
fn test_hierarchical_triggers_multiple_with_summary() {
    let config = CompressionConfig {
        split_threshold: 2,
        hierarchical_threshold: 3,
        generate_summaries: true,
        ..Default::default()
    };
    let engine = CompressionEngine::new(make_factory()).with_config(config);

    let mut session = Session::new("hier-with-summary");
    session.add_vertex(make_vertex("v1", "sha256:h1").committed());
    session.add_vertex(make_vertex("v2", "sha256:h2").with_parent("v1").committed());
    session.add_vertex(make_vertex("v3", "sha256:h3").with_parent("v2").committed());
    session.add_vertex(make_vertex("v4", "sha256:h4").with_parent("v3").committed());
    session.finalize(SessionOutcome::Committed);

    let result = engine.compress(&session).expect("should compress");
    assert!(result.has_braids());
    if let CompressionResult::Multiple { braids, summary } = &result {
        assert!(!braids.is_empty());
        assert!(summary.is_some(), "summaries enabled → should have summary");
    } else {
        panic!("expected Multiple result from hierarchical strategy");
    }
}

#[test]
fn test_split_triggers_multiple_with_summary() {
    let config = CompressionConfig {
        split_threshold: 2,
        generate_summaries: true,
        ..Default::default()
    };
    let engine = CompressionEngine::new(make_factory()).with_config(config);

    // Diamond DAG: v1→(v2,v3), v2+v3→v4, v4→(v5,v6), v5+v6→v7, v7→(v8,v9), v8+v9→v10
    // branch_count = 3 (v1,v4,v7 each have >1 child), tips = 1 (v10)
    // convergence = 1/3 ≈ 0.33 < 0.5 → Split
    let mut session = Session::new("split-diamond");
    session.add_vertex(make_vertex("v1", "sha256:d1").committed());
    session.add_vertex(make_vertex("v2", "sha256:d2").with_parent("v1").committed());
    session.add_vertex(make_vertex("v3", "sha256:d3").with_parent("v1").committed());
    session.add_vertex(
        make_vertex("v4", "sha256:d4")
            .with_parent("v2")
            .with_parent("v3")
            .committed(),
    );
    session.add_vertex(make_vertex("v5", "sha256:d5").with_parent("v4").committed());
    session.add_vertex(make_vertex("v6", "sha256:d6").with_parent("v4").committed());
    session.add_vertex(
        make_vertex("v7", "sha256:d7")
            .with_parent("v5")
            .with_parent("v6")
            .committed(),
    );
    session.add_vertex(make_vertex("v8", "sha256:d8").with_parent("v7").committed());
    session.add_vertex(make_vertex("v9", "sha256:d9").with_parent("v7").committed());
    session.add_vertex(
        make_vertex("v10", "sha256:d10")
            .with_parent("v8")
            .with_parent("v9")
            .committed(),
    );
    session.finalize(SessionOutcome::Committed);

    let result = engine.compress(&session).expect("should compress");
    assert!(result.has_braids());
    if let CompressionResult::Multiple { braids, summary } = &result {
        assert!(!braids.is_empty());
        assert!(summary.is_some(), "summaries enabled → should have summary");
    } else {
        panic!("expected Multiple result from split strategy");
    }
}
