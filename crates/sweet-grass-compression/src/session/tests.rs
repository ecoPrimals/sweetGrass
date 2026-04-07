// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

#![cfg(test)]
#![expect(
    clippy::expect_used,
    reason = "test module: expect is standard in tests"
)]

use super::*;

fn make_vertex(id: &str, hash: &str) -> SessionVertex {
    SessionVertex::new(id, hash, "application/json", Did::new("did:key:z6MkTest"))
}

#[test]
fn test_empty_session() {
    let session = Session::new("test-session");
    assert_eq!(session.vertex_count(), 0);
    assert_eq!(session.branch_count(), 0);
    assert_eq!(session.max_depth(), 0);
}

#[test]
fn test_linear_session() {
    let mut session = Session::new("linear");
    session.add_vertex(make_vertex("v1", "sha256:a").committed());
    session.add_vertex(make_vertex("v2", "sha256:b").with_parent("v1").committed());
    session.add_vertex(make_vertex("v3", "sha256:c").with_parent("v2").committed());

    assert_eq!(session.vertex_count(), 3);
    assert_eq!(session.branch_count(), 0);
    assert_eq!(session.max_depth(), 2);
    assert_eq!(session.roots().len(), 1);
    assert_eq!(session.tips().len(), 1);
}

#[test]
fn test_branching_session() {
    let mut session = Session::new("branching");
    session.add_vertex(make_vertex("root", "sha256:root").committed());
    session.add_vertex(
        make_vertex("branch1", "sha256:b1")
            .with_parent("root")
            .committed(),
    );
    session.add_vertex(
        make_vertex("branch2", "sha256:b2")
            .with_parent("root")
            .committed(),
    );

    assert_eq!(session.vertex_count(), 3);
    assert_eq!(session.branch_count(), 1);
    assert_eq!(session.tips().len(), 2);
    assert_eq!(session.unique_outputs().len(), 2);
}

#[test]
fn test_contributors() {
    let mut session = Session::new("collab");
    let agent1 = Did::new("did:key:z6MkAgent1");
    let agent2 = Did::new("did:key:z6MkAgent2");

    session
        .add_vertex(SessionVertex::new("v1", "sha256:a", "text/plain", agent1.clone()).committed());
    session.add_vertex(
        SessionVertex::new("v2", "sha256:b", "text/plain", agent2.clone())
            .with_parent("v1")
            .committed(),
    );

    let contributors = session.contributors();
    assert_eq!(contributors.len(), 2);
    assert!(contributors.contains(&agent1));
    assert!(contributors.contains(&agent2));
}

#[test]
fn test_committed_filter() {
    let mut session = Session::new("mixed");
    session.add_vertex(make_vertex("v1", "sha256:a").committed());
    session.add_vertex(make_vertex("v2", "sha256:b")); // Not committed
    session.add_vertex(make_vertex("v3", "sha256:c").committed());

    assert_eq!(session.vertex_count(), 3);
    assert_eq!(session.committed_vertices().len(), 2);
}

#[test]
fn test_session_finalize() {
    let mut session = Session::new("finalize");
    assert_eq!(session.outcome, SessionOutcome::InProgress);
    assert!(session.ended_at.is_none());

    session.finalize(SessionOutcome::Committed);

    assert_eq!(session.outcome, SessionOutcome::Committed);
    assert!(session.ended_at.is_some());
}

#[test]
fn test_vertex_with_size() {
    let v = make_vertex("v1", "sha256:a").with_size(4096);
    assert_eq!(v.size, 4096);
}

#[test]
fn test_vertex_with_parent() {
    let v = make_vertex("v1", "sha256:a").with_parent("parent1");
    assert_eq!(v.parents, vec!["parent1"]);

    let v = v.with_parent("parent2");
    assert_eq!(v.parents, vec!["parent1", "parent2"]);
}

#[test]
fn test_vertex_with_activity_type() {
    let v = make_vertex("v1", "sha256:a").with_activity_type(ActivityType::Derivation);
    assert_eq!(v.activity_type, ActivityType::Derivation);
}

#[test]
fn test_vertex_committed() {
    let v = make_vertex("v1", "sha256:a").committed();
    assert!(v.committed);
}

#[test]
fn test_vertex_is_root() {
    let root = make_vertex("root", "sha256:r");
    assert!(root.is_root());

    let child = make_vertex("child", "sha256:c").with_parent("root");
    assert!(!child.is_root());
}

#[test]
fn test_unique_outputs_single_tip() {
    let mut session = Session::new("single-tip");
    session.add_vertex(make_vertex("v1", "sha256:a").committed());
    session.add_vertex(make_vertex("v2", "sha256:b").with_parent("v1").committed());

    let outputs = session.unique_outputs();
    assert_eq!(outputs.len(), 1);
    assert_eq!(outputs[0].as_str(), "sha256:b");
}

#[test]
fn test_unique_outputs_excludes_uncommitted() {
    let mut session = Session::new("uncommitted-tip");
    session.add_vertex(make_vertex("v1", "sha256:a").committed());
    session.add_vertex(make_vertex("v2", "sha256:b").with_parent("v1")); // not committed

    let outputs = session.unique_outputs();
    assert_eq!(outputs.len(), 0);
}

#[test]
fn test_unique_outputs_excludes_intermediate() {
    let mut session = Session::new("intermediate");
    session.add_vertex(make_vertex("v1", "sha256:a").committed());
    session.add_vertex(make_vertex("v2", "sha256:b").with_parent("v1").committed());
    session.add_vertex(make_vertex("v3", "sha256:c").with_parent("v2").committed());

    let outputs = session.unique_outputs();
    assert_eq!(outputs.len(), 1);
    assert_eq!(outputs[0].as_str(), "sha256:c");
}

#[test]
fn test_unique_outputs_empty_session() {
    let session = Session::new("empty");
    assert!(session.unique_outputs().is_empty());
}

#[test]
fn test_temporal_span_empty() {
    let session = Session::new("empty");
    assert_eq!(session.temporal_span(), 0);
}

#[test]
fn test_temporal_span_multiple_vertices() {
    let mut session = Session::new("temporal");
    let mut v1 = make_vertex("v1", "sha256:a");
    v1.timestamp = 1000;
    session.add_vertex(v1);

    let mut v2 = make_vertex("v2", "sha256:b");
    v2.timestamp = 5000;
    session.add_vertex(v2.with_parent("v1"));

    let mut v3 = make_vertex("v3", "sha256:c");
    v3.timestamp = 3000;
    session.add_vertex(v3.with_parent("v1"));

    assert_eq!(session.temporal_span(), 4000); // 5000 - 1000
}

#[test]
fn test_is_atomic_hint() {
    let mut session = Session::new("atomic-hint");
    session.compression_hint = CompressionHint::Atomic;
    session.add_vertex(make_vertex("v1", "sha256:a"));
    session.add_vertex(make_vertex("v2", "sha256:b").with_parent("v1"));

    assert!(session.is_atomic());
}

#[test]
fn test_is_atomic_single_vertex() {
    let mut session = Session::new("single");
    session.add_vertex(make_vertex("v1", "sha256:a"));

    assert!(session.is_atomic());
}

#[test]
fn test_is_atomic_false() {
    let mut session = Session::new("multi");
    session.add_vertex(make_vertex("v1", "sha256:a"));
    session.add_vertex(make_vertex("v2", "sha256:b").with_parent("v1"));

    assert!(!session.is_atomic());
}

#[test]
fn test_has_single_outcome_one() {
    let mut session = Session::new("one");
    session.add_vertex(make_vertex("v1", "sha256:a").committed());
    assert!(session.has_single_outcome());
}

#[test]
fn test_has_single_outcome_none() {
    let session = Session::new("none");
    assert!(session.has_single_outcome());
}

#[test]
fn test_has_single_outcome_multiple() {
    let mut session = Session::new("multi");
    session.add_vertex(make_vertex("root", "sha256:root").committed());
    session.add_vertex(
        make_vertex("b1", "sha256:b1")
            .with_parent("root")
            .committed(),
    );
    session.add_vertex(
        make_vertex("b2", "sha256:b2")
            .with_parent("root")
            .committed(),
    );
    assert!(!session.has_single_outcome());
}

#[test]
fn test_session_outcome_committed() {
    assert_eq!(SessionOutcome::Committed, SessionOutcome::Committed);
}

#[test]
fn test_session_outcome_rollback() {
    assert_eq!(SessionOutcome::Rollback, SessionOutcome::Rollback);
}

#[test]
fn test_session_outcome_in_progress() {
    assert_eq!(SessionOutcome::InProgress, SessionOutcome::InProgress);
}

#[test]
fn test_session_outcome_noop() {
    assert_eq!(SessionOutcome::NoOp, SessionOutcome::NoOp);
}

#[test]
fn test_session_outcome_default() {
    assert_eq!(SessionOutcome::default(), SessionOutcome::InProgress);
}

#[test]
fn test_compression_hint_single() {
    assert_eq!(CompressionHint::Single, CompressionHint::Single);
}

#[test]
fn test_compression_hint_auto() {
    assert_eq!(CompressionHint::Auto, CompressionHint::Auto);
}

#[test]
fn test_compression_hint_atomic() {
    assert_eq!(CompressionHint::Atomic, CompressionHint::Atomic);
}

#[test]
fn test_compression_hint_ephemeral() {
    assert_eq!(CompressionHint::Ephemeral, CompressionHint::Ephemeral);
}

#[test]
fn test_compression_hint_important() {
    assert_eq!(CompressionHint::Important, CompressionHint::Important);
}

#[test]
fn test_compression_hint_default() {
    assert_eq!(CompressionHint::default(), CompressionHint::Auto);
}

#[test]
fn test_session_serialization_roundtrip() {
    let mut session = Session::new("roundtrip");
    session.add_vertex(
        make_vertex("v1", "sha256:a")
            .with_size(100)
            .with_parent("nonexistent")
            .with_activity_type(ActivityType::Creation)
            .committed(),
    );
    session.outcome = SessionOutcome::Committed;
    session.compression_hint = CompressionHint::Atomic;

    let json = serde_json::to_string(&session).expect("serialize");
    let restored: Session = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(restored.id, session.id);
    assert_eq!(restored.vertex_count(), session.vertex_count());
    assert_eq!(restored.outcome, session.outcome);
    assert_eq!(restored.compression_hint, session.compression_hint);
    assert_eq!(restored.vertices[0].size, 100);
    assert!(restored.vertices[0].committed);
}

#[test]
fn test_max_depth_diamond() {
    let mut session = Session::new("diamond");
    session.add_vertex(make_vertex("root", "sha256:root").committed());
    session.add_vertex(
        make_vertex("left", "sha256:left")
            .with_parent("root")
            .committed(),
    );
    session.add_vertex(
        make_vertex("right", "sha256:right")
            .with_parent("root")
            .committed(),
    );
    session.add_vertex(
        make_vertex("tip", "sha256:tip")
            .with_parent("left")
            .with_parent("right")
            .committed(),
    );

    assert_eq!(session.max_depth(), 2);
}

#[test]
fn test_max_depth_deep_chain() {
    let mut session = Session::new("chain");
    session.add_vertex(make_vertex("v0", "sha256:v0").committed());
    session.add_vertex(make_vertex("v1", "sha256:v1").with_parent("v0").committed());
    session.add_vertex(make_vertex("v2", "sha256:v2").with_parent("v1").committed());
    session.add_vertex(make_vertex("v3", "sha256:v3").with_parent("v2").committed());
    session.add_vertex(make_vertex("v4", "sha256:v4").with_parent("v3").committed());

    assert_eq!(session.max_depth(), 4);
}

#[test]
fn test_roots_empty() {
    let session = Session::new("empty");
    assert!(session.roots().is_empty());
}

#[test]
fn test_tips_empty() {
    let session = Session::new("empty");
    assert!(session.tips().is_empty());
}

#[test]
fn test_roots_multiple() {
    let mut session = Session::new("multi-root");
    session.add_vertex(make_vertex("r1", "sha256:r1").committed());
    session.add_vertex(make_vertex("r2", "sha256:r2").committed());
    session.add_vertex(
        make_vertex("merge", "sha256:m")
            .with_parent("r1")
            .with_parent("r2")
            .committed(),
    );

    let roots = session.roots();
    assert_eq!(roots.len(), 2);
}

#[test]
fn test_tips_single_vertex() {
    let mut session = Session::new("single");
    session.add_vertex(make_vertex("only", "sha256:only").committed());

    let tips = session.tips();
    assert_eq!(tips.len(), 1);
    assert_eq!(tips[0].id, "only");
}

mod proptests {
    use super::*;
    use proptest::prelude::*;

    fn arb_vertex_id() -> impl Strategy<Value = String> {
        "[a-z]{1,8}".prop_map(|s| format!("v-{s}"))
    }

    fn arb_hash() -> impl Strategy<Value = String> {
        "[a-f0-9]{8}".prop_map(|h| format!("sha256:{h}"))
    }

    proptest! {
        #[test]
        fn vertex_count_matches_additions(
            ids in proptest::collection::vec((arb_vertex_id(), arb_hash()), 1..30)
        ) {
            let mut session = Session::new("prop-session");
            let mut unique_ids = std::collections::HashSet::new();

            for (id, hash) in &ids {
                if unique_ids.insert(id.clone()) {
                    session.add_vertex(make_vertex(id, hash).committed());
                }
            }

            prop_assert_eq!(session.vertex_count(), unique_ids.len());
        }

        #[test]
        fn roots_are_parentless(count in 1usize..10) {
            let mut session = Session::new("prop-roots");

            for i in 0..count {
                session.add_vertex(
                    make_vertex(&format!("r{i}"), &format!("sha256:r{i}")).committed(),
                );
            }

            let roots = session.roots();
            prop_assert_eq!(roots.len(), count);
            for root in roots {
                prop_assert!(root.parents.is_empty());
            }
        }

        #[test]
        fn tips_have_no_children(chain_len in 1usize..8) {
            let mut session = Session::new("prop-tips");

            session.add_vertex(make_vertex("v0", "sha256:v0").committed());
            for i in 1..chain_len {
                session.add_vertex(
                    make_vertex(
                        &format!("v{i}"),
                        &format!("sha256:v{i}"),
                    )
                    .with_parent(format!("v{}", i - 1))
                    .committed(),
                );
            }

            let tips = session.tips();
            prop_assert_eq!(tips.len(), 1);
            prop_assert_eq!(&tips[0].id, &format!("v{}", chain_len - 1));
        }

        #[test]
        fn max_depth_of_chain(chain_len in 1usize..15) {
            let mut session = Session::new("prop-depth");

            session.add_vertex(make_vertex("v0", "sha256:v0").committed());
            for i in 1..chain_len {
                session.add_vertex(
                    make_vertex(
                        &format!("v{i}"),
                        &format!("sha256:v{i}"),
                    )
                    .with_parent(format!("v{}", i - 1))
                    .committed(),
                );
            }

            let depth = session.max_depth();
            prop_assert_eq!(depth, chain_len - 1);
        }
    }
}
