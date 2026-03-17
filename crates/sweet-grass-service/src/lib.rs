// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! `SweetGrass` Service Layer.
//!
//! Pure Rust service providing tarpc (primary) and REST (fallback) APIs.
//!
//! ## Protocol Stack
//!
//! | Protocol | Env Var | Latency | Use Case |
//! |----------|---------|---------|----------|
//! | tarpc | `SWEETGRASS_TARPC_ADDRESS` | ~50μs | Primal-to-primal |
//! | REST | `SWEETGRASS_REST_ADDRESS` | ~10ms | Debug, admin |
//!
//! Addresses are discovered at runtime via discovery service or environment variables.
//!
//! ## tarpc API (Primary)
//!
//! High-performance binary RPC using `#[tarpc::service]` macros.
//! No gRPC, no protobuf, no vendor lock-in.
//!
//! ```rust,ignore
//! use sweet_grass_service::rpc::SweetGrassRpcClient;
//! use tarpc::{client, context};
//!
//! // Address discovered via registry service or SWEETGRASS_TARPC_ADDRESS env var
//! let client = connect_tarpc(&discovered_address).await?;
//! let braid = client.get_braid(context::current(), braid_id).await??;
//! ```
//!
//! ## REST API (Fallback)
//!
//! ### Braids
//! - `GET /api/v1/braids/:id` — Get a Braid by ID
//! - `GET /api/v1/braids/hash/:hash` — Get a Braid by content hash
//! - `POST /api/v1/braids` — Create a new Braid
//! - `GET /api/v1/braids?agent=...&tag=...` — Query Braids
//!
//! ### Provenance
//! - `GET /api/v1/provenance/:hash` — Get provenance graph
//! - `GET /api/v1/provenance/:hash/prov-o` — Export as PROV-O JSON-LD
//!
//! ### Attribution
//! - `GET /api/v1/attribution/:hash` — Get attribution chain
//!
//! ### Compression
//! - `POST /api/v1/compress` — Compress session to Braids
//!
//! ### Health
//! - `GET /health` — Health check

#![cfg_attr(not(test), forbid(unsafe_code))]
#![cfg_attr(test, deny(unsafe_code))]
#![warn(missing_docs)]

pub mod bootstrap;
pub mod error;
pub mod exit;
pub mod factory;
pub mod handlers;
pub mod router;
pub mod rpc;
pub mod server;
pub mod state;
pub mod streaming;
#[cfg(unix)]
pub mod uds;

pub use bootstrap::{
    BootstrapConfig, BootstrapError, BootstrapResult, create_app_state_from_env, infant_bootstrap,
    infant_bootstrap_with_config,
};
pub use error::ServiceError;
pub use factory::{BraidStoreFactory, StorageConfig};
pub use router::create_router;
pub use rpc::{RpcError, SweetGrassRpc, SweetGrassRpcClient};
pub use server::{SweetGrassServer, start_tarpc_server};
pub use state::AppState;

/// Result type for service operations.
pub type Result<T> = std::result::Result<T, ServiceError>;
