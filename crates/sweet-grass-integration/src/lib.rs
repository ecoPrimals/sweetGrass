//! Integration adapters for `SweetGrass`.
//!
//! This crate provides capability-based discovery and integration with other
//! primals. `SweetGrass` discovers primals at runtime based on what capabilities
//! they offer, not hardcoded addresses.
//!
//! ## Key Concepts
//!
//! - **Capability-based discovery**: Find primals by what they can do
//! - **No hardcoded addresses**: Primals are discovered, not configured
//! - **Test isolation**: Mocks are in `testing` modules, not production code
//! - **Zero-knowledge startup**: Service starts with minimal config, discovers rest
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                     SweetGrass Integration                       │
//! │                                                                  │
//! │  ┌─────────────────────────────────────────────────────────────┐│
//! │  │                  Capability Discovery                        ││
//! │  │    find_by_capability(Signing) → DiscoveredPrimal           ││
//! │  └─────────────────────────────────────────────────────────────┘│
//! │                            │                                     │
//! │         ┌──────────────────┼──────────────────┐                 │
//! │         ▼                  ▼                  ▼                 │
//! │  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐         │
//! │  │  Session    │    │  Anchoring  │    │   Signing   │         │
//! │  │   Events    │    │  Service    │    │   Service   │         │
//! │  │ (Activity)  │    │ (Permanent) │    │ (Identity)  │         │
//! │  └─────────────┘    └─────────────┘    └─────────────┘         │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use sweet_grass_integration::{
//!     LocalDiscovery, DiscoverySigner, Capability,
//!     create_signing_client_async,
//! };
//! use std::sync::Arc;
//!
//! // Create discovery service
//! let discovery = Arc::new(LocalDiscovery::new());
//!
//! // Register a primal (or discover from network)
//! discovery.register(/* primal info */).await;
//!
//! // Find signing primal and create client
//! let primal = discovery.find_one(&Capability::Signing).await?;
//! let client = create_signing_client_async(&primal).await?;
//!
//! // Create signer with the client
//! let signer = DiscoverySigner::with_client(client).await?;
//! let signed_braid = signer.sign_braid(&braid).await?;
//! ```

#![forbid(unsafe_code)]
// Note: These pedantic lints are planned for cleanup in v0.3.0
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::doc_markdown)]

pub mod anchor;
pub mod discovery;
pub mod error;
pub mod listener;
pub mod signer;

// Core discovery exports
pub use discovery::{
    create_discovery, CachedDiscovery, DiscoveredPrimal, DiscoveryError, LocalDiscovery,
    PrimalDiscovery, ServiceInfo, SongbirdDiscovery, SongbirdRpc,
};

// Anchor exports (capability-based naming)
#[cfg(any(test, feature = "test-support"))]
pub use anchor::MockAnchoringClient;
pub use anchor::{
    create_anchoring_client_async, AnchorInfo, AnchorManager, AnchorReceipt, AnchoringClient,
    AnchoringRpc, TarpcAnchoringClient,
};

// Deprecated anchor aliases for backward compatibility
#[allow(deprecated)]
pub use anchor::{
    create_loamspine_client_async, LoamSpineClient, LoamSpineRpc, TarpcLoamSpineClient,
};

#[cfg(any(test, feature = "test-support"))]
#[allow(deprecated)]
pub use anchor::MockLoamSpineClient;

// Error exports
pub use error::IntegrationError;

// Listener exports (capability-based naming)
#[cfg(any(test, feature = "test-support"))]
pub use listener::MockSessionEventsClient;
pub use listener::{
    create_session_events_client_async, EventHandler, SessionEvent, SessionEventStream,
    SessionEventType, SessionEventsClient, SessionEventsRpc, TarpcSessionEventsClient,
};

// Deprecated listener aliases for backward compatibility
#[allow(deprecated)]
pub use listener::{
    create_rhizocrypt_client_async, RhizoCryptClient, RhizoCryptRpc, TarpcRhizoCryptClient,
};

#[cfg(any(test, feature = "test-support"))]
#[allow(deprecated)]
pub use listener::MockRhizoCryptClient;

// Signer exports (capability-based naming)
pub use signer::{
    create_signing_client_async, DiscoverySigner, LegacySigner, SignatureInfo, Signer,
    SigningClient, SigningRpc, SigningRpcClient, TarpcSigningClient,
};

// Deprecated aliases for backward compatibility
#[allow(deprecated)]
pub use signer::{create_beardog_client, create_beardog_client_async, TarpcBearDogClient};

// Type alias for backward compatibility
#[deprecated(since = "0.3.0", note = "Use LegacySigner - capability-based naming")]
pub type BearDogSigner = LegacySigner;

// Test support exports (only when feature enabled or in tests)
#[cfg(any(test, feature = "test-support"))]
pub use signer::testing::MockSigningClient;

// Backward compatibility alias
#[cfg(any(test, feature = "test-support"))]
#[deprecated(
    since = "0.3.0",
    note = "Use MockSigningClient - capability-based naming"
)]
#[allow(deprecated)]
pub use signer::testing::MockBearDogClient;

/// Result type for integration operations.
pub type Result<T> = std::result::Result<T, IntegrationError>;
