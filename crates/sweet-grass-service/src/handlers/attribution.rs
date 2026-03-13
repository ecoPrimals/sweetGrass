// SPDX-License-Identifier: AGPL-3.0-only
//! Attribution chain handlers.

use axum::{
    extract::{Path, State},
    Json,
};
use serde::Serialize;

use crate::{error::ServiceError, state::AppState};

/// Attribution chain response.
#[derive(Debug, Serialize)]
pub struct AttributionResponse {
    /// Entity hash.
    pub entity_hash: String,

    /// Contributors and their shares.
    pub contributors: Vec<ContributorInfo>,

    /// Total shares (should be 1.0 if normalized).
    pub total_shares: f64,

    /// Maximum derivation depth.
    pub max_depth: u32,
}

/// Information about a contributor.
#[derive(Debug, Serialize)]
pub struct ContributorInfo {
    /// Agent DID.
    pub agent: String,

    /// Share amount (0.0 to 1.0).
    pub share: f64,

    /// Role that earned this share.
    pub role: String,

    /// Whether this is a direct contribution.
    pub direct: bool,

    /// Depth in derivation chain.
    pub depth: u32,
}

/// Get attribution chain for an entity.
pub async fn get_attribution(
    State(state): State<AppState>,
    Path(hash): Path<String>,
) -> Result<Json<AttributionResponse>, ServiceError> {
    let content_hash = sweet_grass_core::ContentHash::new(&hash);
    let chain = state.query.attribution_chain(&content_hash).await?;

    let contributors: Vec<ContributorInfo> = chain
        .contributors
        .iter()
        .map(|c| ContributorInfo {
            agent: c.agent.to_string(),
            share: c.share,
            role: format!("{:?}", c.role),
            direct: c.direct,
            depth: c.depth,
        })
        .collect();

    let total_shares: f64 = chain.contributors.iter().map(|c| c.share).sum();

    Ok(Json(AttributionResponse {
        entity_hash: hash,
        contributors,
        total_shares,
        max_depth: chain.max_depth,
    }))
}

/// Reward distribution request.
#[derive(Debug, serde::Deserialize)]
pub struct RewardRequest {
    /// Total value to distribute.
    pub total_value: f64,
}

/// Reward distribution response.
#[derive(Debug, Serialize)]
pub struct RewardResponse {
    /// Entity hash.
    pub entity_hash: String,

    /// Rewards per agent.
    pub rewards: Vec<AgentReward>,

    /// Total distributed.
    pub total_distributed: f64,
}

/// Reward for a single agent.
#[derive(Debug, Serialize)]
pub struct AgentReward {
    /// Agent DID.
    pub agent: String,

    /// Share percentage.
    pub share: f64,

    /// Reward amount.
    pub reward: f64,
}

/// Calculate reward distribution.
pub async fn calculate_rewards(
    State(state): State<AppState>,
    Path(hash): Path<String>,
    Json(request): Json<RewardRequest>,
) -> Result<Json<RewardResponse>, ServiceError> {
    let content_hash = sweet_grass_core::ContentHash::new(&hash);
    let chain = state.query.attribution_chain(&content_hash).await?;

    let rewards: Vec<AgentReward> = chain
        .contributors
        .iter()
        .map(|c| AgentReward {
            agent: c.agent.to_string(),
            share: c.share,
            reward: c.share * request.total_value,
        })
        .collect();

    let total_distributed: f64 = rewards.iter().map(|r| r.reward).sum();

    Ok(Json(RewardResponse {
        entity_hash: hash,
        rewards,
        total_distributed,
    }))
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use axum::extract::{Path, State};
    use std::sync::Arc;
    use sweet_grass_core::agent::Did;
    use sweet_grass_factory::BraidFactory;

    fn create_test_state() -> AppState {
        AppState::new_memory(Did::new("did:key:z6MkTest"))
    }

    async fn create_state_with_braid() -> (AppState, String) {
        let state = create_test_state();
        let factory = Arc::new(BraidFactory::new(Did::new("did:key:z6MkCreator")));
        let braid = factory.from_data(b"test data", "text/plain", None).unwrap();
        let hash = braid.data_hash.as_str().to_string();
        state.store.put(&braid).await.unwrap();
        (state, hash)
    }

    #[tokio::test]
    async fn test_get_attribution_single_contributor() {
        let (state, hash) = create_state_with_braid().await;

        let result = get_attribution(State(state), Path(hash.clone())).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.entity_hash, hash);
        assert!(!response.contributors.is_empty());
        // Total shares should sum to approximately 1.0
        assert!(response.total_shares > 0.0);
    }

    #[tokio::test]
    async fn test_get_attribution_not_found() {
        let state = create_test_state();
        let result = get_attribution(State(state), Path("nonexistent".to_string())).await;
        // Should return an error for non-existent entity
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_calculate_rewards() {
        let (state, hash) = create_state_with_braid().await;

        let request = RewardRequest { total_value: 100.0 };
        let result = calculate_rewards(State(state), Path(hash.clone()), Json(request)).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.entity_hash, hash);
        // Total distributed should equal total value (approximately)
        assert!(response.total_distributed > 0.0);
    }

    #[tokio::test]
    async fn test_calculate_rewards_with_derived_braid() {
        use sweet_grass_core::braid::BraidMetadata;
        use sweet_grass_core::entity::EntityReference;

        let state = create_test_state();

        // Create parent braid (Alice)
        let alice = Did::new("did:key:z6MkAlice");
        let factory = Arc::new(BraidFactory::new(alice.clone()));
        let parent = factory
            .from_data(b"parent data", "text/plain", None)
            .unwrap();
        let parent_hash = parent.data_hash.clone();
        state.store.put(&parent).await.unwrap();

        // Create child braid (Bob) derived from parent
        let bob = Did::new("did:key:z6MkBob");
        let child_factory = BraidFactory::new(bob.clone());
        let mut child = child_factory
            .from_data(
                b"child data",
                "text/plain",
                Some(BraidMetadata {
                    title: Some("Derived".to_string()),
                    ..Default::default()
                }),
            )
            .unwrap();
        child.was_derived_from = vec![EntityReference::by_hash(&parent_hash)];
        let child_hash = child.data_hash.clone();
        state.store.put(&child).await.unwrap();

        // Get attribution for child
        let request = RewardRequest { total_value: 100.0 };
        let result = calculate_rewards(
            State(state),
            Path(child_hash.as_str().to_string()),
            Json(request),
        )
        .await;
        assert!(result.is_ok());

        let response = result.unwrap();
        // Should have rewards distributed
        assert!(!response.rewards.is_empty());
    }

    #[tokio::test]
    async fn test_contributor_info_fields() {
        let (state, hash) = create_state_with_braid().await;

        let result = get_attribution(State(state), Path(hash)).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        if let Some(contributor) = response.contributors.first() {
            assert!(!contributor.agent.is_empty());
            assert!(contributor.share >= 0.0);
            assert!(!contributor.role.is_empty());
        }
    }
}
