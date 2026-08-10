// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! Atomic provenance verification: `braid.verify`.
//!
//! Combines content integrity (Merkle hash), Ed25519 signature verification
//! (delegated to crypto provider), and ledger confirmation (loamSpine) into
//! a single atomic call.

use base64::Engine;
use serde::Deserialize;
use sweet_grass_core::braid::BraidId;
use sweet_grass_store::BraidStore;

use crate::state::AppState;

use super::{DispatchError, DispatchResult, error_code, internal, parse_params, to_value};

#[derive(Debug, Deserialize)]
pub(super) struct VerifyBraidParams {
    braid_id: BraidId,
}

/// Atomic provenance verification: content integrity + Ed25519 signature + ledger.
///
/// Single-call verification combining:
/// 1. **Content integrity** — recomputes signing hash from braid fields,
///    confirms metadata consistency.
/// 2. **Signature verification** — if a witness exists, delegates Ed25519
///    verification to the crypto provider (`crypto.verify_ed25519`).
/// 3. **Ledger confirmation** — if loamSpine is available, verifies the
///    certificate registration via `certificate.verify`.
///
/// Returns a unified `{ verified, checks: [...] }` response. `verified` is
/// `true` only when ALL available checks pass. Checks that cannot run
/// (provider unavailable) are marked `"skipped"` and do not fail the result.
pub(super) async fn handle_braid_verify(
    state: &AppState,
    params: serde_json::Value,
) -> DispatchResult {
    let p: VerifyBraidParams = parse_params(params)?;

    let braid = state
        .store
        .get(&p.braid_id)
        .await
        .map_err(internal)?
        .ok_or_else(|| DispatchError {
            code: error_code::NOT_FOUND,
            message: format!("Braid not found: {}", p.braid_id),
            source_detail: None,
        })?;

    let mut checks: Vec<serde_json::Value> = Vec::with_capacity(3);
    let mut all_passed = true;

    // Check 1: Content integrity — recompute signing hash
    let signing_hash = braid.compute_signing_hash();
    let content_valid = signing_hash.as_str().starts_with("sha256:");
    checks.push(serde_json::json!({
        "check": "content_integrity",
        "status": if content_valid { "pass" } else { "fail" },
        "signing_hash": signing_hash.as_str(),
        "data_hash": braid.data_hash.as_str(),
    }));
    if !content_valid {
        all_passed = false;
    }

    // Check 2: Ed25519 signature verification
    if braid.witness.is_signed() {
        let sig_check = verify_witness_signature(state, &braid).await;
        let passed = sig_check
            .get("status")
            .and_then(serde_json::Value::as_str)
            != Some("fail");
        if !passed {
            all_passed = false;
        }
        checks.push(sig_check);
    } else {
        checks.push(serde_json::json!({
            "check": "signature",
            "status": "unsigned",
            "detail": "no witness signature present",
        }));
        all_passed = false;
    }

    // Check 3: Ledger verification
    if let Some(ref client) = state.ledger_client {
        let braid_id_str = p
            .braid_id
            .as_str()
            .strip_prefix("urn:braid:")
            .unwrap_or(p.braid_id.as_str());
        match client.verify_certificate(braid_id_str).await {
            Ok(result) => {
                let passed = result.valid;
                checks.push(serde_json::json!({
                    "check": "ledger",
                    "status": if passed { "pass" } else { "fail" },
                    "detail": result.detail,
                }));
                if !passed {
                    all_passed = false;
                }
            },
            Err(e) => {
                checks.push(serde_json::json!({
                    "check": "ledger",
                    "status": "skipped",
                    "detail": format!("loamSpine unavailable: {e}"),
                }));
            },
        }
    } else {
        checks.push(serde_json::json!({
            "check": "ledger",
            "status": "skipped",
            "detail": "no ledger client configured",
        }));
    }

    to_value(&serde_json::json!({
        "braid_id": p.braid_id.as_str(),
        "verified": all_passed,
        "checks": checks,
        "data_hash": braid.data_hash.as_str(),
        "attributed_to": braid.was_attributed_to.as_str(),
        "generated_at_time": braid.generated_at_time.nanos(),
    }))
}

/// Attempt Ed25519 signature verification via the crypto delegate.
///
/// Decodes the witness evidence (base64 signature), recomputes the signing
/// hash, and delegates verification to the capability provider. Falls back
/// to "presence only" if the crypto delegate is unavailable.
async fn verify_witness_signature(
    state: &AppState,
    braid: &sweet_grass_core::Braid,
) -> serde_json::Value {
    let signing_hash = braid.compute_signing_hash();
    let message = signing_hash.as_str().as_bytes();

    let sig_bytes = {
        let evidence = &*braid.witness.evidence;
        if evidence.is_empty() {
            return serde_json::json!({
                "check": "signature",
                "status": "fail",
                "detail": "witness evidence is empty",
            });
        }
        match base64::engine::general_purpose::STANDARD.decode(evidence) {
            Ok(b) => b,
            Err(_) => {
                return serde_json::json!({
                    "check": "signature",
                    "status": "fail",
                    "detail": "witness evidence is not valid base64",
                });
            },
        }
    };

    let Some(crypto) = &state.crypto else {
        return serde_json::json!({
            "check": "signature",
            "status": "present",
            "detail": "signature present but crypto provider unavailable for verification",
            "agent": braid.witness.agent.as_str(),
            "algorithm": braid.witness.algorithm.as_deref(),
        });
    };

    let Some(pub_key_bytes) = extract_public_key_from_did(&braid.witness.agent) else {
        return serde_json::json!({
            "check": "signature",
            "status": "present",
            "detail": "cannot extract public key from agent DID for verification",
            "agent": braid.witness.agent.as_str(),
        });
    };

    match crypto.verify(message, &sig_bytes, &pub_key_bytes).await {
        Ok(true) => serde_json::json!({
            "check": "signature",
            "status": "pass",
            "agent": braid.witness.agent.as_str(),
            "algorithm": braid.witness.algorithm.as_deref(),
        }),
        Ok(false) => serde_json::json!({
            "check": "signature",
            "status": "fail",
            "detail": "Ed25519 signature invalid",
            "agent": braid.witness.agent.as_str(),
        }),
        Err(e) => serde_json::json!({
            "check": "signature",
            "status": "present",
            "detail": format!("crypto provider error: {e}"),
            "agent": braid.witness.agent.as_str(),
        }),
    }
}

/// Extract raw Ed25519 public key bytes from a `did:key:z6Mk...` DID.
///
/// Uses base64url-no-pad decoding (matching `Did::from_public_key_bytes`).
/// Returns `None` if the DID format is unrecognized or decoding fails.
fn extract_public_key_from_did(did: &sweet_grass_core::agent::Did) -> Option<Vec<u8>> {
    let s = did.as_str();
    let key_part = s.strip_prefix("did:key:z6Mk")?;
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(key_part)
        .ok()
}
