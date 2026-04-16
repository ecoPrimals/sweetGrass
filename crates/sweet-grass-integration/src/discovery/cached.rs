// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! TTL-based caching decorator for [`PrimalDiscovery`] implementations.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use sweet_grass_core::config::Capability;
use tokio::sync::RwLock;

use super::{DiscoveredPrimal, DiscoveryBackend, DiscoveryError, PrimalDiscovery};

/// Walk nested backends until a concrete `Local` or `Registry` implementation is reached,
/// avoiding trait dispatch through [`DiscoveryBackend`] that would recurse through
/// [`CachedDiscovery`] and produce an infinitely-sized async state machine.
async fn find_by_capability_leaf(
    mut current: &DiscoveryBackend,
    capability: &Capability,
) -> Result<Vec<DiscoveredPrimal>, DiscoveryError> {
    loop {
        match current {
            DiscoveryBackend::Local(d) => return d.find_by_capability(capability).await,
            DiscoveryBackend::Registry(d) => return d.find_by_capability(capability).await,
            DiscoveryBackend::Cached(c) => current = c.inner.as_ref(),
        }
    }
}

async fn announce_leaf(
    mut current: &DiscoveryBackend,
    primal: &DiscoveredPrimal,
) -> Result<(), DiscoveryError> {
    loop {
        match current {
            DiscoveryBackend::Local(d) => return d.announce(primal).await,
            DiscoveryBackend::Registry(d) => return d.announce(primal).await,
            DiscoveryBackend::Cached(c) => current = c.inner.as_ref(),
        }
    }
}

async fn health_leaf(current: &DiscoveryBackend) -> bool {
    let mut current = current;
    loop {
        match current {
            DiscoveryBackend::Local(d) => return d.health().await,
            DiscoveryBackend::Registry(d) => return d.health().await,
            DiscoveryBackend::Cached(c) => current = c.inner.as_ref(),
        }
    }
}

/// Discovery client that caches results and handles failover.
///
/// Wraps any [`PrimalDiscovery`] implementation with a TTL-based cache
/// to reduce repeated network queries for the same capability.
#[derive(Clone)]
pub struct CachedDiscovery {
    inner: Arc<DiscoveryBackend>,
    cache: Arc<RwLock<HashMap<Capability, Vec<DiscoveredPrimal>>>>,
    cache_ttl: Duration,
}

impl CachedDiscovery {
    /// Create a new cached discovery client.
    #[must_use]
    pub fn new(inner: Arc<DiscoveryBackend>, cache_ttl: Duration) -> Self {
        Self {
            inner,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl,
        }
    }

    /// Invalidate cache for a capability.
    pub async fn invalidate(&self, capability: &Capability) {
        let mut cache = self.cache.write().await;
        cache.remove(capability);
    }

    /// Invalidate all cache entries.
    pub async fn invalidate_all(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }
}

impl PrimalDiscovery for CachedDiscovery {
    async fn find_by_capability(
        &self,
        capability: &Capability,
    ) -> Result<Vec<DiscoveredPrimal>, DiscoveryError> {
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(capability) {
                let valid: Vec<_> = cached
                    .iter()
                    .filter(|p| {
                        p.last_seen
                            .elapsed()
                            .map(|e| e < self.cache_ttl)
                            .unwrap_or(false)
                    })
                    .cloned()
                    .collect();

                if !valid.is_empty() {
                    return Ok(valid);
                }
            }
        }

        let primals = find_by_capability_leaf(self.inner.as_ref(), capability).await?;

        {
            let mut cache = self.cache.write().await;
            cache.insert(capability.clone(), primals.clone());
        }

        Ok(primals)
    }

    async fn announce(&self, primal: &DiscoveredPrimal) -> Result<(), DiscoveryError> {
        announce_leaf(self.inner.as_ref(), primal).await
    }

    async fn health(&self) -> bool {
        health_leaf(self.inner.as_ref()).await
    }

    async fn find_one(&self, capability: &Capability) -> Result<DiscoveredPrimal, DiscoveryError> {
        let mut primals = self.find_by_capability(capability).await?;
        primals.sort_by(|a, b| b.last_seen.cmp(&a.last_seen));
        primals
            .into_iter()
            .find(|p| p.healthy)
            .ok_or_else(|| DiscoveryError::CapabilityNotFound(capability.clone()))
    }
}
