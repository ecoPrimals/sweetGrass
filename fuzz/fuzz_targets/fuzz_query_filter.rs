// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Fuzz test for QueryFilter.
//!
//! Tests that arbitrary filter configurations don't cause panics.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use sweet_grass_store::QueryFilter;

/// Arbitrary filter input for fuzzing.
#[derive(Debug, Arbitrary)]
struct FuzzFilter {
    data_hash: Option<String>,
    attributed_to: Option<String>,
    mime_type: Option<String>,
    tag: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
}

fuzz_target!(|input: FuzzFilter| {
    // Build a filter from arbitrary input
    let mut filter = QueryFilter::new();

    if let Some(hash) = input.data_hash {
        filter = filter.with_hash(&hash);
    }

    if let Some(did) = input.attributed_to {
        filter = filter.with_agent(&sweet_grass_core::agent::Did::new(did));
    }

    if let Some(mime) = input.mime_type {
        filter = filter.with_mime(&mime);
    }

    if let Some(tag) = input.tag {
        filter = filter.with_tag(&tag);
    }

    if let Some(limit) = input.limit {
        filter = filter.with_limit(limit);
    }

    if let Some(offset) = input.offset {
        filter = filter.with_offset(offset);
    }

    // The filter should be valid regardless of input
    let _ = filter;
});

