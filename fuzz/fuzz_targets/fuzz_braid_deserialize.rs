// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Fuzz test for Braid deserialization.
//!
//! Tests that arbitrary JSON input doesn't cause panics in Braid deserialization.

#![no_main]
#![forbid(unsafe_code)]

use libfuzzer_sys::fuzz_target;
use sweet_grass_core::Braid;

fuzz_target!(|data: &[u8]| {
    // Try to deserialize as JSON
    let _ = serde_json::from_slice::<Braid>(data);

    // Try to interpret as UTF-8 and parse
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = serde_json::from_str::<Braid>(s);
    }
});

