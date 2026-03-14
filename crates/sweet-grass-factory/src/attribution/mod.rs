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

        #[expect(
            clippy::cast_possible_wrap,
            reason = "depth is recursion depth; powi requires i32; depth is bounded by graph depth"
        )]
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

        #[expect(
            clippy::cast_precision_loss,
            reason = "len() is small; usize->f64 precision loss is acceptable for weight calculation"
        )]
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
#[allow(
    clippy::float_cmp,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::manual_range_contains
)]
mod tests;
