// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! `BearDog` Secure Tunnel Protocol (BTSP) — Phase 2 server handshake.
//!
//! Implements the server side of the BTSP handshake per
//! `BTSP_PROTOCOL_STANDARD.md` v1.0.  All cryptographic operations are
//! delegated to a primal offering `Capability::Signing` (`BearDog`) via
//! JSON-RPC over UDS — sweetGrass never touches key material directly.
//!
//! ## Activation
//!
//! When [`is_btsp_required`] returns `true` (i.e. `FAMILY_ID` is set and
//! `BIOMEOS_INSECURE` is not `"1"`), incoming connections are auto-detected
//! via first-line inspection:
//!
//! - First byte **not** `{` → length-prefixed BTSP handshake (canonical wire format)
//! - First line is `{"protocol":"btsp",...}` → JSON-line BTSP handshake
//!   (primalSpring-compatible, same 4-step handshake over newline-delimited JSON)
//! - First line is `{"jsonrpc":"2.0",...}` → raw JSON-RPC (health probes,
//!   biomeOS, springs)
//!
//! In development mode (no family) all connections use raw newline-delimited
//! JSON-RPC. This first-line auto-detect aligns with Phase 45b wire-format
//! guidance and matches `BearDog` (PG-35) / `Squirrel` (PG-30) patterns.

pub mod protocol;
pub mod server;

pub use protocol::{
    BtspError, ChallengeResponse, ClientHello, HandshakeComplete, HandshakeError, ServerHello,
    read_frame, read_jsonline, write_frame, write_jsonline,
};
pub use server::{
    perform_server_handshake, perform_server_handshake_jsonline,
    perform_server_handshake_jsonline_with, perform_server_handshake_with,
};

use sweet_grass_core::primal_names::env_vars;

/// Returns `true` when BTSP handshake is required on incoming connections.
///
/// Per `BTSP_PROTOCOL_STANDARD` §Security Model:
/// - `FAMILY_ID` set (non-empty, not `"default"`) AND `BIOMEOS_INSECURE` not `"1"` → required
/// - No `FAMILY_ID` → development mode, raw JSON-RPC
#[must_use]
pub fn is_btsp_required() -> bool {
    let family_id = crate::uds::resolve_family_id_from_env();
    is_btsp_required_with(
        family_id.as_deref(),
        std::env::var(env_vars::BIOMEOS_INSECURE)
            .map(|v| v == "1")
            .unwrap_or(false),
    )
}

/// DI-friendly variant for tests.
#[must_use]
pub const fn is_btsp_required_with(family_id: Option<&str>, biomeos_insecure: bool) -> bool {
    family_id.is_some() && !biomeos_insecure
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn btsp_required_when_family_set() {
        assert!(is_btsp_required_with(Some("alpha"), false));
    }

    #[test]
    fn btsp_not_required_no_family() {
        assert!(!is_btsp_required_with(None, false));
    }

    #[test]
    fn btsp_not_required_insecure_no_family() {
        assert!(!is_btsp_required_with(None, true));
    }

    #[test]
    fn btsp_not_required_insecure_with_family() {
        assert!(!is_btsp_required_with(Some("alpha"), true));
    }
}
