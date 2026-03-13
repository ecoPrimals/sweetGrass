// SPDX-License-Identifier: AGPL-3.0-only
//! Attribution chain calculation.
//!
//! This module implements the attribution calculation algorithm
//! that determines how credit/rewards should be distributed
//! among contributors to a piece of data.

mod chain;

pub use chain::{AttributionChain, ContributorShare};

use std::collections::HashMap;
use std::sync::Arc;
use sweet_grass_core::{
    agent::{AgentRole, Did},
    entity::EntityReference,
    Braid, ContentHash,
};

use crate::error::FactoryError;
use crate::Result;

/// Default weight for the Curator role in attribution calculation.
pub const DEFAULT_CURATOR_ROLE_WEIGHT: f64 = 0.10;

/// Default maximum derivation depth to consider in attribution chains.
pub const DEFAULT_ATTRIBUTION_MAX_DEPTH: u32 = 10;

/// Configuration for attribution calculation.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AttributionConfig {
    /// Role weights (how much each role contributes).
    pub role_weights: HashMap<AgentRole, f64>,

    /// Decay factor per derivation depth (0.0 to 1.0).
    pub decay_factor: f64,

    /// Maximum derivation depth to consider.
    pub max_depth: u32,

    /// Minimum share to include (smaller shares are dropped).
    pub min_share: f64,
}

impl Default for AttributionConfig {
    fn default() -> Self {
        let mut role_weights = HashMap::new();
        role_weights.insert(AgentRole::Creator, 0.40);
        role_weights.insert(AgentRole::Contributor, 0.25);
        role_weights.insert(AgentRole::Transformer, 0.20);
        role_weights.insert(AgentRole::Curator, DEFAULT_CURATOR_ROLE_WEIGHT);
        role_weights.insert(AgentRole::Publisher, 0.05);

        Self {
            role_weights,
            decay_factor: 0.5,
            max_depth: DEFAULT_ATTRIBUTION_MAX_DEPTH,
            min_share: 0.001,
        }
    }
}

impl AttributionConfig {
    /// Get the weight for a role.
    #[must_use]
    pub fn weight_for_role(&self, role: &AgentRole) -> f64 {
        *self.role_weights.get(role).unwrap_or(&0.1)
    }
}

/// Calculator for attribution chains.
#[derive(Clone)]
pub struct AttributionCalculator {
    config: AttributionConfig,
}

impl AttributionCalculator {
    /// Create a new calculator with default config.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: AttributionConfig::default(),
        }
    }

    /// Create a new calculator with custom config.
    #[must_use]
    pub const fn with_config(config: AttributionConfig) -> Self {
        Self { config }
    }

    /// Calculate attribution for a single Braid.
    #[must_use]
    pub fn calculate_single(&self, braid: &Braid) -> AttributionChain {
        let entity = EntityReference::by_hash(braid.data_hash.clone());
        let mut chain = AttributionChain::new(entity);

        let role = Self::infer_role_from_braid(braid);
        let weight = self.config.weight_for_role(&role);

        chain.add_contributor(ContributorShare::new(
            braid.was_attributed_to.clone(),
            weight,
            role,
            true,
        ));

        if let Some(activity) = &braid.was_generated_by {
            for assoc in &activity.was_associated_with {
                if assoc.agent != braid.was_attributed_to {
                    let weight = self.config.weight_for_role(&assoc.role);
                    chain.add_contributor(ContributorShare::new(
                        assoc.agent.clone(),
                        weight,
                        assoc.role.clone(),
                        true,
                    ));
                }
            }

            if let Some(compute) = activity.ecop.compute_units {
                chain.total_compute = compute;
            }
        }

        chain.normalize();
        chain
    }

    /// Calculate attribution including derivation chain.
    ///
    /// This takes a resolver function that can look up Braids by hash.
    pub fn calculate_with_derivations<F>(&self, braid: &Braid, resolve: F) -> AttributionChain
    where
        F: Fn(&ContentHash) -> Option<Braid>,
    {
        let entity = EntityReference::by_hash(braid.data_hash.clone());
        let mut chain = AttributionChain::new(entity);
        let mut visited = std::collections::HashSet::<ContentHash>::new();

        self.calculate_recursive(braid, &mut chain, &resolve, 0, &mut visited, 1.0);

        chain
            .contributors
            .retain(|c| c.share >= self.config.min_share);
        chain.normalize();
        chain
    }

    /// Calculate attribution for multiple Braids in parallel.
    ///
    /// Returns attribution chains for each Braid, calculated concurrently.
    /// This method spawns parallel tasks for each Braid to maximize throughput.
    pub async fn calculate_batch<F>(
        self: Arc<Self>,
        braids: Vec<Braid>,
        resolve: Arc<F>,
    ) -> Vec<AttributionChain>
    where
        F: Fn(&ContentHash) -> Option<Braid> + Send + Sync + 'static,
    {
        use futures::stream::{FuturesUnordered, StreamExt};

        let mut futures = FuturesUnordered::new();

        for braid in braids {
            let resolve = Arc::clone(&resolve);
            let calculator = Arc::clone(&self);

            futures.push(tokio::spawn(async move {
                calculator.calculate_with_derivations(&braid, |hash| resolve(hash))
            }));
        }

        let mut results = Vec::new();
        while let Some(result) = futures.next().await {
            if let Ok(chain) = result {
                results.push(chain);
            }
        }

        results
    }

    fn calculate_recursive<F>(
        &self,
        braid: &Braid,
        chain: &mut AttributionChain,
        resolve: &F,
        depth: u32,
        visited: &mut std::collections::HashSet<ContentHash>,
        weight_multiplier: f64,
    ) where
        F: Fn(&ContentHash) -> Option<Braid>,
    {
        if depth > self.config.max_depth {
            return;
        }

        let hash = braid.data_hash.clone();
        if visited.contains(&hash) {
            return;
        }
        visited.insert(hash);

        #[allow(clippy::cast_possible_wrap)]
        let decay = self.config.decay_factor.powi(depth as i32);
        let effective_weight = weight_multiplier * decay;

        let role = Self::infer_role_from_braid(braid);
        let base_weight = self.config.weight_for_role(&role);

        chain.add_contributor(
            ContributorShare::new(
                braid.was_attributed_to.clone(),
                base_weight * effective_weight,
                role,
                depth == 0,
            )
            .with_depth(depth),
        );

        if let Some(activity) = &braid.was_generated_by {
            for assoc in &activity.was_associated_with {
                if assoc.agent != braid.was_attributed_to {
                    let assoc_weight = self.config.weight_for_role(&assoc.role);
                    chain.add_contributor(
                        ContributorShare::new(
                            assoc.agent.clone(),
                            assoc_weight * effective_weight,
                            assoc.role.clone(),
                            depth == 0,
                        )
                        .with_depth(depth),
                    );
                }
            }

            if let Some(compute) = activity.ecop.compute_units {
                chain.total_compute += compute * effective_weight;
            }
        }

        #[allow(clippy::cast_precision_loss)]
        let derivation_weight = effective_weight / braid.was_derived_from.len().max(1) as f64;
        for derived in &braid.was_derived_from {
            if let Some(hash) = derived.content_hash() {
                if let Some(parent) = resolve(hash) {
                    self.calculate_recursive(
                        &parent,
                        chain,
                        resolve,
                        depth + 1,
                        visited,
                        derivation_weight,
                    );
                }
            }
        }
    }

    fn infer_role_from_braid(braid: &Braid) -> AgentRole {
        if let Some(activity) = &braid.was_generated_by {
            if let Some(assoc) = activity.was_associated_with.first() {
                return assoc.role.clone();
            }
        }

        if braid.was_derived_from.is_empty() {
            AgentRole::Creator
        } else {
            AgentRole::Transformer
        }
    }
}

impl Default for AttributionCalculator {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate reward distribution for a given total value.
///
/// # Errors
///
/// Returns an error if the attribution chain is not normalized.
pub fn calculate_rewards(chain: &AttributionChain, total_value: f64) -> Result<Vec<(Did, f64)>> {
    if !chain.is_valid() {
        return Err(FactoryError::Attribution(
            "Attribution chain is not normalized".to_string(),
        ));
    }

    let rewards: Vec<(Did, f64)> = chain
        .contributors
        .iter()
        .map(|c| (c.agent.clone(), c.share * total_value))
        .collect();

    Ok(rewards)
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use sweet_grass_core::activity::ActivityType;
    use sweet_grass_core::Braid;

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
            config.weight_for_role(&AgentRole::Creator)
                > config.weight_for_role(&AgentRole::Publisher)
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
}

/// Property-based tests for attribution calculations.
#[cfg(test)]
#[allow(clippy::float_cmp, clippy::unwrap_used, clippy::manual_range_contains)]
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
