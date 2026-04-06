// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Tests for attribution chain calculation.

use super::*;
use std::collections::HashMap;
use std::sync::Arc;
use sweet_grass_core::Braid;
use sweet_grass_core::activity::ActivityType;
use sweet_grass_core::entity::EntityReference;

fn make_test_braid(hash: &str, agent: &str) -> Braid {
    let did = Did::new(agent);
    Braid::builder()
        .data_hash(hash)
        .mime_type("application/json")
        .size(1024)
        .attributed_to(did)
        .build()
        .expect("should build")
}

#[test]
fn test_single_attribution() {
    let braid = make_test_braid("sha256:test1", "did:key:z6MkTest");
    let calculator = AttributionCalculator::new();

    let chain = calculator.calculate_single(&braid);

    assert!(chain.is_valid());
    assert_eq!(chain.contributors.len(), 1);
    assert_eq!(chain.contributors[0].agent.as_str(), "did:key:z6MkTest");
    assert!((chain.contributors[0].share - 1.0).abs() < 0.001);
}

#[test]
fn test_attribution_with_activity() {
    use sweet_grass_core::agent::AgentAssociation;

    let did1 = Did::new("did:key:z6MkCreator");
    let did1_attr = did1.clone();
    let did2 = Did::new("did:key:z6MkContributor");

    let activity = sweet_grass_core::Activity::builder(ActivityType::Creation)
        .associated_with(AgentAssociation::new(did1, AgentRole::Creator))
        .associated_with(AgentAssociation::new(did2, AgentRole::Contributor))
        .compute_units(2.5)
        .build();

    let braid = Braid::builder()
        .data_hash("sha256:collab")
        .mime_type("application/json")
        .size(1024)
        .attributed_to(did1_attr)
        .generated_by(activity)
        .build()
        .expect("should build");

    let calculator = AttributionCalculator::new();
    let chain = calculator.calculate_single(&braid);

    assert!(chain.is_valid());
    assert_eq!(chain.contributors.len(), 2);
    assert!(chain.total_compute > 0.0);
}

#[test]
fn test_derivation_chain() {
    let parent = make_test_braid("sha256:parent", "did:key:z6MkParent");
    let child = {
        let did = Did::new("did:key:z6MkChild");
        let mut braid = Braid::builder()
            .data_hash("sha256:child")
            .mime_type("application/json")
            .size(512)
            .attributed_to(did)
            .build()
            .expect("should build");
        braid.was_derived_from = vec![EntityReference::by_hash("sha256:parent")];
        braid
    };

    let calculator = AttributionCalculator::new();

    let resolve = |hash: &ContentHash| {
        if hash == "sha256:parent" {
            Some(parent.clone())
        } else {
            None
        }
    };

    let chain = calculator.calculate_with_derivations(&child, resolve);

    assert!(chain.is_valid());
    assert_eq!(chain.max_depth, 1);

    let agents: Vec<_> = chain
        .contributors
        .iter()
        .map(|c| c.agent.as_str())
        .collect();
    assert!(agents.contains(&"did:key:z6MkChild"));
    assert!(agents.contains(&"did:key:z6MkParent"));
}

#[test]
fn test_reward_calculation() {
    let braid = make_test_braid("sha256:test", "did:key:z6MkTest");
    let calculator = AttributionCalculator::new();
    let chain = calculator.calculate_single(&braid);

    let rewards = calculate_rewards(&chain, 100.0).expect("should calculate");

    assert_eq!(rewards.len(), 1);
    assert!((rewards[0].1 - 100.0).abs() < 0.01);
}

#[test]
fn test_aggregate_by_agent() {
    let mut chain = AttributionChain::new(EntityReference::by_hash("sha256:test"));
    let agent = Did::new("did:key:z6MkAgent");

    chain.add_contributor(ContributorShare::new(
        agent.clone(),
        0.3,
        AgentRole::Creator,
        true,
    ));
    chain.add_contributor(ContributorShare::new(agent, 0.2, AgentRole::Curator, true));

    let aggregated = chain.aggregate_by_agent();

    assert_eq!(aggregated.len(), 1);
    assert!((aggregated.get("did:key:z6MkAgent").unwrap() - 0.5).abs() < 0.001);
}

#[test]
fn test_attribution_config() {
    let config = AttributionConfig::default();

    assert!(
        config.weight_for_role(&AgentRole::Creator) > config.weight_for_role(&AgentRole::Publisher)
    );
    assert!(config.decay_factor > 0.0 && config.decay_factor <= 1.0);
}

#[test]
fn test_direct_vs_inherited() {
    let mut chain = AttributionChain::new(EntityReference::by_hash("sha256:test"));
    let agent1 = Did::new("did:key:z6MkDirect");
    let agent2 = Did::new("did:key:z6MkInherited");

    chain.add_contributor(ContributorShare::new(agent1, 0.5, AgentRole::Creator, true));
    chain.add_contributor(ContributorShare::new(
        agent2,
        0.5,
        AgentRole::Creator,
        false,
    ));

    assert_eq!(chain.direct_contributors().len(), 1);
    assert_eq!(chain.inherited_contributors().len(), 1);
}

#[test]
fn test_with_config() {
    let mut weights = HashMap::new();
    weights.insert(AgentRole::Creator, 0.80);
    let config = AttributionConfig {
        role_weights: weights,
        decay_factor: 0.3,
        max_depth: 5,
        min_share: 0.01,
    };
    let calc = AttributionCalculator::with_config(config);
    let braid = make_test_braid("sha256:custom-cfg", "did:key:z6MkCfg");
    let chain = calc.calculate_single(&braid);
    assert!(chain.is_valid());
    assert!((chain.contributors[0].share - 1.0).abs() < 0.001);
}

#[test]
fn test_weight_for_unknown_role() {
    let config = AttributionConfig::default();
    let weight = config.weight_for_role(&AgentRole::Custom("unknown".to_string()));
    assert!((weight - 0.1).abs() < f64::EPSILON);
}

#[test]
fn test_calculate_rewards_invalid_chain() {
    let mut chain = AttributionChain::new(EntityReference::by_hash("sha256:invalid"));
    chain.add_contributor(ContributorShare::new(
        Did::new("did:key:z6MkA"),
        0.3,
        AgentRole::Creator,
        true,
    ));
    chain.add_contributor(ContributorShare::new(
        Did::new("did:key:z6MkB"),
        0.3,
        AgentRole::Creator,
        true,
    ));

    let result = calculate_rewards(&chain, 100.0);
    assert!(result.is_err());
}

fn no_resolve(_: &ContentHash) -> Option<Braid> {
    None
}

#[tokio::test]
async fn test_calculate_batch() {
    let calc = Arc::new(AttributionCalculator::new());
    let braids = vec![
        make_test_braid("sha256:batch1", "did:key:z6MkBatch1"),
        make_test_braid("sha256:batch2", "did:key:z6MkBatch2"),
        make_test_braid("sha256:batch3", "did:key:z6MkBatch3"),
    ];

    let resolve = Arc::new(no_resolve);

    let results = calc.calculate_batch(braids, resolve).await;
    assert_eq!(results.len(), 3);
    for chain in &results {
        assert!(chain.is_valid());
    }
}

#[test]
fn test_infer_role_from_derived_braid() {
    let did = Did::new("did:key:z6MkTransformer");
    let mut braid = Braid::builder()
        .data_hash("sha256:derived")
        .mime_type("application/json")
        .size(256)
        .attributed_to(did)
        .build()
        .expect("build");
    braid.was_derived_from = vec![EntityReference::by_hash("sha256:source")];

    let calc = AttributionCalculator::new();
    let chain = calc.calculate_single(&braid);
    assert!(chain.is_valid());
    assert_eq!(chain.contributors[0].role, AgentRole::Transformer);
}

#[test]
fn test_derivation_cycle_protection() {
    let braid_a = {
        let did = Did::new("did:key:z6MkA");
        let mut b = Braid::builder()
            .data_hash("sha256:cycleA")
            .mime_type("text/plain")
            .size(10)
            .attributed_to(did)
            .build()
            .expect("build");
        b.was_derived_from = vec![EntityReference::by_hash("sha256:cycleB")];
        b
    };
    let braid_b = {
        let did = Did::new("did:key:z6MkB");
        let mut b = Braid::builder()
            .data_hash("sha256:cycleB")
            .mime_type("text/plain")
            .size(10)
            .attributed_to(did)
            .build()
            .expect("build");
        b.was_derived_from = vec![EntityReference::by_hash("sha256:cycleA")];
        b
    };

    let calc = AttributionCalculator::new();
    let resolve = {
        let a = braid_a.clone();
        let b = braid_b;
        move |hash: &ContentHash| {
            if hash == "sha256:cycleA" {
                Some(a.clone())
            } else if hash == "sha256:cycleB" {
                Some(b.clone())
            } else {
                None
            }
        }
    };

    let chain = calc.calculate_with_derivations(&braid_a, resolve);
    assert!(chain.is_valid());
}

#[test]
fn test_max_depth_respected() {
    let config = AttributionConfig {
        max_depth: 1,
        ..AttributionConfig::default()
    };
    let calc = AttributionCalculator::with_config(config);

    let grandparent = make_test_braid("sha256:grandparent", "did:key:z6MkGrandparent");
    let parent = {
        let did = Did::new("did:key:z6MkParent");
        let mut b = Braid::builder()
            .data_hash("sha256:parent_depth")
            .mime_type("text/plain")
            .size(10)
            .attributed_to(did)
            .build()
            .expect("build");
        b.was_derived_from = vec![EntityReference::by_hash("sha256:grandparent")];
        b
    };
    let child = {
        let did = Did::new("did:key:z6MkChild");
        let mut b = Braid::builder()
            .data_hash("sha256:child_depth")
            .mime_type("text/plain")
            .size(10)
            .attributed_to(did)
            .build()
            .expect("build");
        b.was_derived_from = vec![EntityReference::by_hash("sha256:parent_depth")];
        b
    };

    let resolve = {
        let p = parent;
        let gp = grandparent;
        move |hash: &ContentHash| {
            if hash == "sha256:parent_depth" {
                Some(p.clone())
            } else if hash == "sha256:grandparent" {
                Some(gp.clone())
            } else {
                None
            }
        }
    };

    let chain = calc.calculate_with_derivations(&child, resolve);
    assert!(chain.is_valid());
    assert!(
        chain.max_depth <= 1,
        "max_depth should be capped at 1, got {}",
        chain.max_depth
    );
}

#[test]
fn test_default_calculator() {
    let calc = AttributionCalculator::default();
    let braid = make_test_braid("sha256:default-calc", "did:key:z6MkDefault");
    let chain = calc.calculate_single(&braid);
    assert!(chain.is_valid());
}

/// Chaos and fault injection tests for attribution edge cases.
///
/// Follows groundSpring pattern: test with NaN, Inf, extreme values,
/// empty inputs, and boundary conditions.
mod chaos {
    use super::*;

    #[test]
    fn chaos_zero_weight_config() {
        let config = AttributionConfig {
            role_weights: HashMap::new(),
            decay_factor: 0.5,
            max_depth: 10,
            min_share: 0.001,
        };
        let calc = AttributionCalculator::with_config(config);
        let braid = make_test_braid("sha256:zero-weight", "did:key:z6MkZero");
        let chain = calc.calculate_single(&braid);
        assert!(chain.is_valid());
        for c in &chain.contributors {
            assert!(c.share.is_finite());
        }
    }

    #[test]
    fn chaos_extreme_decay_factor_zero() {
        let config = AttributionConfig {
            decay_factor: 0.0,
            ..AttributionConfig::default()
        };
        let calc = AttributionCalculator::with_config(config);
        let parent = make_test_braid("sha256:parent-decay0", "did:key:z6MkP");
        let child = {
            let did = Did::new("did:key:z6MkC");
            let mut b = Braid::builder()
                .data_hash("sha256:child-decay0")
                .mime_type("text/plain")
                .size(10)
                .attributed_to(did)
                .build()
                .expect("build");
            b.was_derived_from = vec![EntityReference::by_hash("sha256:parent-decay0")];
            b
        };
        let p = parent;
        let chain = calc.calculate_with_derivations(&child, |h: &ContentHash| {
            if h == "sha256:parent-decay0" {
                Some(p.clone())
            } else {
                None
            }
        });
        assert!(chain.is_valid());
        for c in &chain.contributors {
            assert!(c.share.is_finite());
        }
    }

    #[test]
    fn chaos_extreme_decay_factor_one() {
        let config = AttributionConfig {
            decay_factor: 1.0,
            ..AttributionConfig::default()
        };
        let calc = AttributionCalculator::with_config(config);
        let braid = make_test_braid("sha256:decay1", "did:key:z6MkD1");
        let chain = calc.calculate_single(&braid);
        assert!(chain.is_valid());
    }

    #[test]
    fn chaos_zero_min_share() {
        let config = AttributionConfig {
            min_share: 0.0,
            ..AttributionConfig::default()
        };
        let calc = AttributionCalculator::with_config(config);
        let braid = make_test_braid("sha256:min0", "did:key:z6MkMin0");
        let chain = calc.calculate_single(&braid);
        assert!(chain.is_valid());
    }

    #[test]
    fn chaos_max_depth_zero() {
        let config = AttributionConfig {
            max_depth: 0,
            ..AttributionConfig::default()
        };
        let calc = AttributionCalculator::with_config(config);
        let parent = make_test_braid("sha256:p-depth0", "did:key:z6MkPD0");
        let child = {
            let did = Did::new("did:key:z6MkCD0");
            let mut b = Braid::builder()
                .data_hash("sha256:c-depth0")
                .mime_type("text/plain")
                .size(10)
                .attributed_to(did)
                .build()
                .expect("build");
            b.was_derived_from = vec![EntityReference::by_hash("sha256:p-depth0")];
            b
        };
        let p = parent;
        let chain = calc.calculate_with_derivations(&child, |h: &ContentHash| {
            if h == "sha256:p-depth0" {
                Some(p.clone())
            } else {
                None
            }
        });
        assert!(chain.is_valid());
        assert_eq!(chain.max_depth, 0, "Should not recurse at all");
    }

    #[test]
    fn chaos_empty_contributors_normalize() {
        let mut chain = AttributionChain::new(EntityReference::by_hash("sha256:empty"));
        chain.normalize();
        assert!(chain.contributors.is_empty());
    }

    #[test]
    fn chaos_single_zero_share_normalize() {
        let mut chain = AttributionChain::new(EntityReference::by_hash("sha256:zsn"));
        chain.add_contributor(ContributorShare::new(
            Did::new("did:key:z6MkZ"),
            0.0,
            AgentRole::Creator,
            true,
        ));
        chain.normalize();
        for c in &chain.contributors {
            assert!(c.share.is_finite());
        }
    }

    #[test]
    fn chaos_rewards_zero_value() {
        let braid = make_test_braid("sha256:r0", "did:key:z6MkR0");
        let calc = AttributionCalculator::new();
        let chain = calc.calculate_single(&braid);
        let rewards = calculate_rewards(&chain, 0.0);
        assert!(rewards.is_ok());
        let r = rewards.unwrap();
        assert_eq!(r.len(), 1);
        assert!((r[0].1 - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn chaos_rewards_large_value() {
        let braid = make_test_braid("sha256:big", "did:key:z6MkBig");
        let calc = AttributionCalculator::new();
        let chain = calc.calculate_single(&braid);
        let rewards = calculate_rewards(&chain, f64::MAX / 2.0);
        assert!(rewards.is_ok());
        for (_, v) in rewards.unwrap() {
            assert!(v.is_finite());
        }
    }

    #[test]
    fn chaos_many_contributors() {
        let mut chain = AttributionChain::new(EntityReference::by_hash("sha256:many"));
        for i in 0..100 {
            chain.add_contributor(ContributorShare::new(
                Did::new(format!("did:key:z6Mk{i}")),
                0.01,
                AgentRole::Contributor,
                true,
            ));
        }
        chain.normalize();
        let total: f64 = chain.contributors.iter().map(|c| c.share).sum();
        assert!((total - 1.0).abs() < 0.001);
    }

    #[test]
    fn chaos_deep_derivation_chain() {
        let braids: Vec<Braid> = (0..20)
            .map(|i| {
                let did = Did::new(format!("did:key:z6MkDeep{i}"));
                let mut b = Braid::builder()
                    .data_hash(format!("sha256:deep{i}"))
                    .mime_type("text/plain")
                    .size(10)
                    .attributed_to(did)
                    .build()
                    .expect("build");
                if i > 0 {
                    b.was_derived_from =
                        vec![EntityReference::by_hash(format!("sha256:deep{}", i - 1))];
                }
                b
            })
            .collect();

        let calc = AttributionCalculator::new();
        let bs = braids.clone();
        let chain = calc.calculate_with_derivations(braids.last().unwrap(), |h: &ContentHash| {
            bs.iter().find(|b| &b.data_hash == h).cloned()
        });
        assert!(chain.is_valid());
        assert!(chain.max_depth <= DEFAULT_ATTRIBUTION_MAX_DEPTH);
    }
}

/// Property-based tests for attribution calculations.
mod proptests {
    use super::*;
    use proptest::prelude::*;

    fn arb_did() -> impl Strategy<Value = Did> {
        "[a-zA-Z0-9]{8,20}".prop_map(|s| Did::new(format!("did:key:z6Mk{s}")))
    }

    fn arb_share() -> impl Strategy<Value = f64> {
        (0.0..=1.0_f64).prop_map(|f| (f * 1000.0).round() / 1000.0)
    }

    fn arb_role() -> impl Strategy<Value = AgentRole> {
        prop_oneof![
            Just(AgentRole::Creator),
            Just(AgentRole::Contributor),
            Just(AgentRole::DataProvider),
            Just(AgentRole::ComputeProvider),
            Just(AgentRole::Curator),
            Just(AgentRole::Publisher),
        ]
    }

    proptest! {
        #[test]
        fn prop_normalized_chain_sums_to_one(
            shares in proptest::collection::vec((arb_did(), arb_share(), arb_role()), 1..10)
        ) {
            let mut chain = AttributionChain::new(EntityReference::by_hash("sha256:test"));

            for (did, share, role) in shares {
                if share > 0.0 {
                    chain.add_contributor(ContributorShare::new(did, share, role, true));
                }
            }

            if !chain.contributors.is_empty() {
                chain.normalize();
                let total: f64 = chain.contributors.iter().map(|c| c.share).sum();
                prop_assert!((total - 1.0).abs() < 0.001, "Sum should be ~1.0, got {}", total);
            }
        }

        #[test]
        fn prop_all_shares_non_negative(
            shares in proptest::collection::vec((arb_did(), arb_share()), 1..5)
        ) {
            let mut chain = AttributionChain::new(EntityReference::by_hash("sha256:test"));

            for (did, share) in shares {
                chain.add_contributor(ContributorShare::new(did, share, AgentRole::Creator, true));
            }

            chain.normalize();

            for c in &chain.contributors {
                prop_assert!(c.share >= 0.0, "Share should be non-negative: {}", c.share);
            }
        }

        #[test]
        fn prop_aggregation_preserves_total(
            shares in proptest::collection::vec((arb_did(), arb_share()), 1..10)
        ) {
            let mut chain = AttributionChain::new(EntityReference::by_hash("sha256:test"));

            for (did, share) in shares {
                chain.add_contributor(ContributorShare::new(did, share, AgentRole::Creator, true));
            }

            let original_total: f64 = chain.contributors.iter().map(|c| c.share).sum();
            let aggregated = chain.aggregate_by_agent();
            let aggregated_total: f64 = aggregated.values().sum();

            prop_assert!(
                (original_total - aggregated_total).abs() < 0.001,
                "Aggregation should preserve total: {} vs {}",
                original_total,
                aggregated_total
            );
        }

        #[test]
        fn prop_rewards_distribute_full_value(
            total_value in 1.0..10000.0_f64,
            shares in proptest::collection::vec(arb_share(), 1..5)
        ) {
            let mut chain = AttributionChain::new(EntityReference::by_hash("sha256:test"));

            for (i, share) in shares.iter().enumerate() {
                if *share > 0.0 {
                    let did = Did::new(format!("did:key:z6MkAgent{i}"));
                    chain.add_contributor(ContributorShare::new(did, *share, AgentRole::Creator, true));
                }
            }

            if chain.contributors.is_empty() {
                return Ok(());
            }

            chain.normalize();

            let rewards = calculate_rewards(&chain, total_value);
            prop_assert!(rewards.is_ok());

            let reward_sum: f64 = rewards.unwrap().iter().map(|(_, v)| v).sum();
            prop_assert!(
                (reward_sum - total_value).abs() < 0.01,
                "Rewards should sum to total value: {} vs {}",
                reward_sum,
                total_value
            );
        }

        #[test]
        fn prop_role_weights_in_range(role in arb_role()) {
            let config = AttributionConfig::default();
            let weight = config.weight_for_role(&role);
            prop_assert!(weight >= 0.0 && weight <= 1.0, "Weight out of range: {}", weight);
        }
    }
}
