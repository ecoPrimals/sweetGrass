// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Router configuration.

use axum::{
    routing::{delete, get, post},
    Router,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::handlers::{attribution, braids, compression, health, jsonrpc, provenance};
use crate::state::AppState;

/// Create the main router with all routes.
pub fn create_router(state: AppState) -> Router {
    // API v1 routes
    let api_v1 = Router::new()
        // Braid endpoints
        .route("/braids", get(braids::list_braids))
        .route("/braids", post(braids::create_provenance_braid))
        .route("/braids/{id}", get(braids::get_braid))
        .route("/braids/{id}", delete(braids::delete_braid))
        .route("/braids/hash/{hash}", get(braids::get_braid_by_hash))
        // Provenance endpoints
        .route("/provenance/{hash}", get(provenance::get_provenance))
        .route("/provenance/{hash}/prov-o", get(provenance::export_prov_o))
        // Attribution endpoints
        .route("/attribution/{hash}", get(attribution::get_attribution))
        .route(
            "/attribution/{hash}/rewards",
            post(attribution::calculate_rewards),
        )
        // Compression endpoints
        .route("/compress", post(compression::compress_session));

    // JSON-RPC 2.0 endpoint (wateringHole required protocol)
    let jsonrpc_route = Router::new().route("/jsonrpc", post(jsonrpc::handle_jsonrpc));

    // Health endpoints
    let health_routes = Router::new()
        .route("/health", get(health::health))
        .route("/health/detailed", get(health::health_detailed))
        .route("/live", get(health::liveness))
        .route("/ready", get(health::readiness));

    // Combine all routes
    Router::new()
        .nest("/api/v1", api_v1)
        .merge(jsonrpc_route)
        .merge(health_routes)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// Create a router for testing (without tracing).
#[cfg(test)]
pub fn create_test_router(state: AppState) -> Router {
    let api_v1 = Router::new()
        .route("/braids", get(braids::list_braids))
        .route("/braids", post(braids::create_braid))
        .route("/braids/{id}", get(braids::get_braid))
        .route("/braids/{id}", delete(braids::delete_braid))
        .route("/braids/hash/{hash}", get(braids::get_braid_by_hash))
        .route("/provenance/{hash}", get(provenance::get_provenance))
        .route("/provenance/{hash}/prov-o", get(provenance::export_prov_o))
        .route("/attribution/{hash}", get(attribution::get_attribution))
        .route(
            "/attribution/{hash}/rewards",
            post(attribution::calculate_rewards),
        )
        .route("/compress", post(compression::compress_session));

    let jsonrpc_route = Router::new().route("/jsonrpc", post(jsonrpc::handle_jsonrpc));

    let health_routes = Router::new()
        .route("/health", get(health::health))
        .route("/health/detailed", get(health::health_detailed))
        .route("/live", get(health::liveness))
        .route("/ready", get(health::readiness));

    Router::new()
        .nest("/api/v1", api_v1)
        .merge(jsonrpc_route)
        .merge(health_routes)
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sweet_grass_core::agent::Did;

    #[test]
    fn test_router_creation() {
        let state = AppState::new_memory(Did::new("did:key:z6MkTest"));
        let _router = create_test_router(state);
        // If we get here, router was created successfully
    }

    #[test]
    fn test_production_router_creation() {
        let state = AppState::new_memory(Did::new("did:key:z6MkTest"));
        let _router = create_router(state);
        // Production router includes tracing and CORS layers
    }
}
