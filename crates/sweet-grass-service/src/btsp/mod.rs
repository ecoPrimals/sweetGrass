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
//! BTSP handshake runs on every incoming UDS/TCP connection when
//! [`is_btsp_required`] returns `true` (i.e. `FAMILY_ID` is set and
//! `BIOMEOS_INSECURE` is not `"1"`).  In development mode (no family)
//! connections fall through to raw newline-delimited JSON-RPC.

pub mod protocol;
pub mod server;

pub use protocol::{
    BtspError, ChallengeResponse, ClientHello, HandshakeComplete, HandshakeError, ServerHello,
    read_frame, write_frame,
};
pub use server::perform_server_handshake;

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
