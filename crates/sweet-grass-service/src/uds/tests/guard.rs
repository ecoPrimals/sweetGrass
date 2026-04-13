// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project
//! BTSP insecure guard tests.

use super::super::*;

#[test]
fn guard_passes_no_family_no_insecure() {
    assert!(validate_insecure_guard_with(None, false).is_ok());
}

#[test]
fn guard_passes_family_set_insecure_off() {
    assert!(validate_insecure_guard_with(Some("alpha"), false).is_ok());
}

#[test]
fn guard_passes_insecure_on_no_family() {
    assert!(validate_insecure_guard_with(None, true).is_ok());
}

#[test]
fn guard_fails_family_and_insecure() {
    let err = validate_insecure_guard_with(Some("alpha"), true).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("alpha"), "error should mention family: {msg}");
    assert!(msg.contains("BTSP"), "error should reference BTSP: {msg}");
    assert!(
        msg.contains("BIOMEOS_INSECURE"),
        "error should mention BIOMEOS_INSECURE: {msg}"
    );
}

#[test]
fn guard_error_display_is_descriptive() {
    let err = BtspGuardViolation {
        family_id: "myFamily42".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("myFamily42"));
    assert!(msg.contains("mutually exclusive"));
}
