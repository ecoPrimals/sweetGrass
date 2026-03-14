// SPDX-License-Identifier: AGPL-3.0-only
//! # `SweetGrass` Integration
//!
//! Integration with other ecoPrimals via pure Rust tarpc (no gRPC).
//!
//! ## Architecture
//!
//! `SweetGrass` uses **capability-based discovery** to find and integrate
//! with other primals at runtime:
//!
//! - **No hardcoded primal names** - discover by capability, not name
//! - **tarpc for RPC** - pure Rust, no gRPC/protobuf
//! - **Runtime discovery** - via registry service or local discovery
//! - **Test isolation** - mocks only in testing modules
//!
//! ## Capabilities
//!
//! | Capability | Purpose | Example Primal |
//! |------------|---------|----------------|
//! | `Signing` | Braid signatures, DID resolution | Identity primal |
//! | `SessionEvents` | Session activity tracking | Activity primal |
//! | `Anchoring` | Permanent storage anchoring | Persistence primal |
//! | `Compute` | Task execution events | Compute primal |
//!
//! ## Integration Patterns
//!
//! ### 1. Capability-Based Discovery
//!
//! ```no_run
//! use sweet_grass_integration::*;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Discover signing capability (could be BearDog, or any primal offering signing)
//!     let discovery = create_discovery().await;
//!     let primal = discovery.find_one(&Capability::Signing).await.unwrap();
//!     let signing_client = create_signing_client_async(&primal).await.unwrap();
//!     // Use signing_client...
//! }
//! ```
//!
//! ### 2. Anchoring
//!
//! ```no_run
//! use sweet_grass_integration::*;
//!
//! #[tokio::main]
//! async fn main() {
//!     let discovery = create_discovery().await;
//!     let primal = discovery.find_one(&Capability::Anchoring).await.unwrap();
//!     let anchor_client = create_anchoring_client_async(&primal).await.unwrap();
//!     // Use anchor_client...
//! }
//! ```
//!
//! ## Test Support
//!
//! All mocks are isolated to `#[cfg(test)]` or `testing` modules:
//!
//! ```rust
//! #[cfg(test)]
//! use sweet_grass_integration::MockSigningClient;
//!
//! #[tokio::test]
//! async fn test_with_mock() {
//!     let mock = MockSigningClient::new();
//!     // ... test code
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod anchor;
mod discovery;
mod error;
mod listener;
pub mod signer;

#[cfg(any(test, feature = "test-support"))]
pub mod testing;

// Re-exports
pub use anchor::{
    create_anchoring_client_async, AnchorInfo, AnchorManager, AnchorReceipt, AnchoringClient,
    TarpcAnchoringClient,
};
pub use discovery::{
    CachedDiscovery, DiscoveredPrimal, LocalDiscovery, PrimalDiscovery, RegistryDiscovery,
};
pub use error::IntegrationError;
pub use listener::{
    create_session_events_client_async, tarpc_client::TarpcSessionEventsClient, EventHandler,
    SessionEventStream, SessionEventsClient,
};
pub use signer::{create_signing_client_async, SignatureInfo, SigningClient};
pub use sweet_grass_core::config::Capability;

pub use discovery::create_discovery;

// Test support (mocks only)
#[cfg(test)]
pub use anchor::MockAnchoringClient;
#[cfg(test)]
pub use listener::MockSessionEventsClient;
#[cfg(test)]
pub use signer::testing::MockSigningClient;

// Type aliases for documentation (not deprecated)
/// Result type for integration operations
pub type Result<T> = std::result::Result<T, IntegrationError>;

/// Modern capability-based integration - discover by capability, not name
///
/// # Example
///
/// ```no_run
/// use sweet_grass_integration::*;
///
/// #[tokio::main]
/// async fn main() {
///     let discovery = create_discovery().await;
///     let primal = discovery.find_one(&Capability::Signing).await.unwrap();
///     let client = create_signing_client_async(&primal).await.unwrap();
///     // Use client...
/// }
/// ```
///
/// Capability-based integration patterns (modern approach)
///
/// Use `Capability::Signing` instead of specific primal names
/// Use `Capability::SessionEvents` instead of specific primal names
/// Use `Capability::Anchoring` instead of specific primal names
pub mod capability_based {

    pub use super::{
        create_discovery, AnchoringClient, Capability, DiscoveredPrimal, LocalDiscovery,
        PrimalDiscovery, SessionEventsClient, SigningClient,
    };
}
