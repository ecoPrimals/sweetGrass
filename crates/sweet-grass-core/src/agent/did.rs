// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Decentralized Identifier (DID) — the cryptographic identity primitive.

use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Decentralized Identifier (DID).
///
/// Uses `Arc<str>` internally so `.clone()` is O(1) (atomic refcount increment).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct Did(Arc<str>);

impl Did {
    /// Create a new DID from a string.
    #[must_use]
    pub fn new(did: impl AsRef<str>) -> Self {
        Self(Arc::from(did.as_ref()))
    }

    /// Get the inner string representation.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Check if this is a valid DID format.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.0.starts_with("did:")
    }

    /// Create a `did:key:` DID from raw Ed25519 public key bytes.
    ///
    /// Encodes the public key as base64 and constructs a `did:key:z6Mk...`
    /// identifier suitable for witness attribution.
    #[must_use]
    pub fn from_public_key_bytes(public_key: &[u8]) -> Self {
        use base64::Engine;
        let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(public_key);
        Self::new(format!("did:key:z6Mk{encoded}"))
    }

    /// Get the DID method (e.g., "key" from "did:key:...").
    #[must_use]
    pub fn method(&self) -> Option<&str> {
        if !self.is_valid() {
            return None;
        }
        self.0.strip_prefix("did:")?.split(':').next()
    }
}

impl<'de> Deserialize<'de> for Did {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self(Arc::from(s)))
    }
}

impl std::fmt::Display for Did {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<&str> for Did {
    fn from(s: &str) -> Self {
        Self(Arc::from(s))
    }
}

impl From<String> for Did {
    fn from(s: String) -> Self {
        Self(Arc::from(s.into_boxed_str()))
    }
}
