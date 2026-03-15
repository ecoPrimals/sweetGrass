// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Primal self-knowledge and identity.
//!
//! A primal knows its own name and capabilities, not others.

use std::time::SystemTime;

use crate::config::Capability;
use crate::identity;

/// Error loading primal self-knowledge from the environment.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum BootstrapEnvError {
    /// An environment variable that should be a port number is not valid.
    #[error("{var_name} must be a valid port number (0-65535), got: {value}")]
    InvalidPort {
        /// Name of the environment variable.
        var_name: String,
        /// The invalid value that was read.
        value: String,
    },
}

/// What a primal knows about itself at startup (Infant Discovery).
///
/// A primal is born knowing only itself - everything else is discovered
/// at runtime through the universal adapter (e.g., service mesh).
///
/// ## Infant Discovery Pattern
///
/// 1. Primal reads environment variables (zero hardcoding)
/// 2. Establishes self-knowledge (name, ID, capabilities)
/// 3. Discovers storage, discovery service, other primals
/// 4. Operates with full network effects
///
/// ## Environment Variables
///
/// - `PRIMAL_NAME`: Human-readable name (default: "sweetgrass")
/// - `PRIMAL_INSTANCE_ID`: Unique ID (default: random UUID)
/// - `PRIMAL_CAPABILITIES`: Comma-separated list of capabilities
/// - `TARPC_PORT`: tarpc endpoint port (0 = auto-allocate)
/// - `REST_PORT`: REST endpoint port (0 = auto-allocate)
///
/// ## Example
///
/// ```rust
/// use sweet_grass_core::primal_info::SelfKnowledge;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Environment:
/// // PRIMAL_NAME=sweetgrass
/// // PRIMAL_INSTANCE_ID=sg-prod-01
/// // PRIMAL_CAPABILITIES=signing,session_events
///
/// let self_knowledge = SelfKnowledge::from_env()?;
/// assert_eq!(self_knowledge.name, "sweetgrass");
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct SelfKnowledge {
    /// Human-readable primal name.
    pub name: String,

    /// Unique instance identifier (persists across restarts).
    pub instance_id: String,

    /// Capabilities this primal offers.
    pub capabilities: Vec<Capability>,

    /// tarpc RPC endpoint port (0 = auto-allocate).
    pub tarpc_port: u16,

    /// REST API endpoint port (0 = auto-allocate).
    pub rest_port: u16,

    /// When this knowledge was established.
    pub established_at: SystemTime,
}

impl SelfKnowledge {
    /// Load self-knowledge from environment variables (Infant Bootstrap).
    ///
    /// # Environment Variables
    ///
    /// - `PRIMAL_NAME`: Name (default: "sweetgrass")
    /// - `PRIMAL_INSTANCE_ID`: Instance ID (default: random UUID)
    /// - `PRIMAL_CAPABILITIES`: Comma-separated capabilities (default: empty)
    /// - `TARPC_PORT`: tarpc port (default: 0 = auto-allocate)
    /// - `REST_PORT`: REST port (default: 0 = auto-allocate)
    ///
    /// # Errors
    ///
    /// Returns error if environment variables are malformed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sweet_grass_core::primal_info::SelfKnowledge;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Default behavior (no env vars set)
    /// let self_knowledge = SelfKnowledge::from_env()?;
    /// assert_eq!(self_knowledge.name, "sweetgrass");
    /// assert_eq!(self_knowledge.tarpc_port, 0); // Auto-allocate
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_env() -> Result<Self, BootstrapEnvError> {
        Ok(Self {
            name: std::env::var("PRIMAL_NAME")
                .unwrap_or_else(|_| identity::PRIMAL_NAME.to_string()),
            instance_id: std::env::var("PRIMAL_INSTANCE_ID")
                .unwrap_or_else(|_| uuid::Uuid::new_v4().to_string()),
            capabilities: Self::parse_capabilities(),
            tarpc_port: Self::parse_port("TARPC_PORT", 0)?,
            rest_port: Self::parse_port("REST_PORT", 0)?,
            established_at: SystemTime::now(),
        })
    }

    /// Parse capabilities from environment.
    fn parse_capabilities() -> Vec<Capability> {
        let caps_str = std::env::var("PRIMAL_CAPABILITIES").unwrap_or_default();

        if caps_str.is_empty() {
            return Vec::new();
        }

        let mut capabilities = Vec::new();
        for cap in caps_str.split(',') {
            let cap = cap.trim();
            // from_string always returns Some, treating unknowns as Custom
            if let Some(capability) = Capability::from_string(cap) {
                capabilities.push(capability);
            }
        }

        capabilities
    }

    /// Parse port from environment.
    fn parse_port(var_name: &str, default: u16) -> Result<u16, BootstrapEnvError> {
        std::env::var(var_name).map_or(Ok(default), |val| {
            val.parse().map_err(|_| BootstrapEnvError::InvalidPort {
                var_name: var_name.to_string(),
                value: val,
            })
        })
    }

    /// Get uptime since establishment.
    #[must_use]
    pub fn uptime(&self) -> std::time::Duration {
        self.established_at
            .elapsed()
            .unwrap_or(std::time::Duration::from_secs(0))
    }

    /// Check if this primal offers a capability.
    #[must_use]
    pub fn offers(&self, capability: &Capability) -> bool {
        self.capabilities.contains(capability)
    }
}

impl Default for SelfKnowledge {
    fn default() -> Self {
        Self {
            name: identity::PRIMAL_NAME.to_string(),
            instance_id: uuid::Uuid::new_v4().to_string(),
            capabilities: Vec::new(),
            tarpc_port: 0, // Dynamic allocation
            rest_port: 0,  // Dynamic allocation
            established_at: SystemTime::now(),
        }
    }
}

#[cfg(test)]
#[allow(unsafe_code)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test module: expect/unwrap are standard in tests"
)]
mod tests {
    use serial_test::serial;

    use super::*;

    fn clear_env() {
        unsafe {
            std::env::remove_var("PRIMAL_NAME");
        }
        unsafe {
            std::env::remove_var("PRIMAL_INSTANCE_ID");
        }
        unsafe {
            std::env::remove_var("PRIMAL_CAPABILITIES");
        }
        unsafe {
            std::env::remove_var("TARPC_PORT");
        }
        unsafe {
            std::env::remove_var("REST_PORT");
        }
    }

    #[test]
    #[serial]
    fn test_self_knowledge_from_env_defaults() {
        clear_env();
        let sk = SelfKnowledge::from_env().expect("should parse defaults");
        assert_eq!(sk.name, "sweetgrass");
        assert!(!sk.instance_id.is_empty());
        assert_eq!(sk.capabilities.len(), 0);
        assert_eq!(sk.tarpc_port, 0);
        assert_eq!(sk.rest_port, 0);
    }

    #[test]
    #[serial]
    fn test_self_knowledge_from_env_custom() {
        clear_env();
        unsafe {
            std::env::set_var("PRIMAL_NAME", "sweetgrass-test");
        }
        unsafe {
            std::env::set_var("PRIMAL_INSTANCE_ID", "test-123");
        }
        unsafe {
            std::env::set_var("PRIMAL_CAPABILITIES", "signing,anchoring");
        }
        unsafe {
            std::env::set_var("TARPC_PORT", "9091");
        }
        unsafe {
            std::env::set_var("REST_PORT", "9080");
        }

        let sk = SelfKnowledge::from_env().expect("should parse custom");
        assert_eq!(sk.name, "sweetgrass-test");
        assert_eq!(sk.instance_id, "test-123");
        assert_eq!(sk.capabilities.len(), 2);
        assert!(sk.offers(&Capability::Signing));
        assert!(sk.offers(&Capability::Anchoring));
        assert_eq!(sk.tarpc_port, 9091);
        assert_eq!(sk.rest_port, 9080);
    }

    #[test]
    #[serial]
    fn test_self_knowledge_custom_capability() {
        clear_env();
        unsafe {
            std::env::set_var("PRIMAL_CAPABILITIES", "signing,custom_feature");
        }

        let sk = SelfKnowledge::from_env().expect("should parse custom capability");
        assert_eq!(sk.capabilities.len(), 2);
        assert!(sk.offers(&Capability::Signing));
        assert!(matches!(sk.capabilities[1], Capability::Custom(_)));
    }

    #[test]
    #[serial]
    fn test_self_knowledge_invalid_port() {
        clear_env();
        unsafe {
            std::env::set_var("TARPC_PORT", "not_a_number");
        }

        let result = SelfKnowledge::from_env();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, BootstrapEnvError::InvalidPort { .. }));
        assert!(err.to_string().contains("TARPC_PORT"));
    }

    #[test]
    fn test_self_knowledge_uptime() {
        let sk = SelfKnowledge::default();
        // Uptime should be measurable immediately (even if < 1ms)
        let uptime = sk.uptime();
        assert!(uptime.as_nanos() > 0, "Uptime should be positive");
    }

    #[test]
    fn test_self_knowledge_offers() {
        let mut sk = SelfKnowledge::default();
        sk.capabilities.push(Capability::Signing);
        assert!(sk.offers(&Capability::Signing));
        assert!(!sk.offers(&Capability::Anchoring));
    }
}
