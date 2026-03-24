// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! Query filtering and sorting for the in-memory store.
//!
//! This module contains the logic for matching Braids against query filters
//! and sorting results according to specified order.

use sweet_grass_core::Braid;

use crate::traits::{QueryFilter, QueryOrder};

/// Check if a Braid matches all criteria in a filter.
pub fn matches(braid: &Braid, filter: &QueryFilter) -> bool {
    if !matches_data_hash(braid, filter) {
        return false;
    }
    if !matches_agent(braid, filter) {
        return false;
    }
    if !matches_braid_type(braid, filter) {
        return false;
    }
    if !matches_time_range(braid, filter) {
        return false;
    }
    if !matches_mime_type(braid, filter) {
        return false;
    }
    if !matches_tag(braid, filter) {
        return false;
    }
    if !matches_ecop_fields(braid, filter) {
        return false;
    }
    true
}

/// Check data hash match.
fn matches_data_hash(braid: &Braid, filter: &QueryFilter) -> bool {
    filter
        .data_hash
        .as_ref()
        .is_none_or(|hash| &braid.data_hash == hash)
}

/// Check agent (attribution) match.
fn matches_agent(braid: &Braid, filter: &QueryFilter) -> bool {
    filter
        .attributed_to
        .as_ref()
        .is_none_or(|agent| &braid.was_attributed_to == agent)
}

/// Check Braid type match (compares discriminants only).
fn matches_braid_type(braid: &Braid, filter: &QueryFilter) -> bool {
    filter.braid_type.as_ref().is_none_or(|expected| {
        std::mem::discriminant(&braid.braid_type) == std::mem::discriminant(expected)
    })
}

/// Check time range match.
const fn matches_time_range(braid: &Braid, filter: &QueryFilter) -> bool {
    if let Some(after) = filter.created_after
        && braid.generated_at_time < after
    {
        return false;
    }
    if let Some(before) = filter.created_before
        && braid.generated_at_time > before
    {
        return false;
    }
    true
}

/// Check MIME type match (prefix match for wildcards like "image/").
fn matches_mime_type(braid: &Braid, filter: &QueryFilter) -> bool {
    filter
        .mime_type
        .as_ref()
        .is_none_or(|mime| braid.mime_type.starts_with(mime))
}

/// Check tag match.
fn matches_tag(braid: &Braid, filter: &QueryFilter) -> bool {
    filter.tag.as_ref().is_none_or(|tag| {
        braid
            .metadata
            .tags
            .iter()
            .any(|t| t.as_ref() == tag.as_str())
    })
}

/// Check ecoPrimals-specific fields match.
fn matches_ecop_fields(braid: &Braid, filter: &QueryFilter) -> bool {
    // Source primal check
    if let Some(ref primal) = filter.source_primal {
        match &braid.ecop.source_primal {
            Some(p) if p.as_ref() == primal.as_str() => {},
            _ => return false,
        }
    }

    // Niche check
    if let Some(ref niche) = filter.niche {
        match &braid.ecop.niche {
            Some(n) if n.as_ref() == niche.as_str() => {},
            _ => return false,
        }
    }

    true
}

/// Sort Braids according to the specified order.
pub fn sort(braids: &mut [Braid], order: &QueryOrder) {
    match order {
        QueryOrder::NewestFirst => {
            braids.sort_by(|a, b| b.generated_at_time.cmp(&a.generated_at_time));
        },
        QueryOrder::OldestFirst => {
            braids.sort_by(|a, b| a.generated_at_time.cmp(&b.generated_at_time));
        },
        QueryOrder::LargestFirst => {
            braids.sort_by(|a, b| b.size.cmp(&a.size));
        },
        QueryOrder::SmallestFirst => {
            braids.sort_by(|a, b| a.size.cmp(&b.size));
        },
    }
}

/// Apply pagination to a result set.
///
/// Returns (paginated results, `has_more`).
pub fn paginate(braids: Vec<Braid>, filter: &QueryFilter) -> (Vec<Braid>, bool) {
    let offset = filter.offset.unwrap_or(0);
    let limit = filter.limit.unwrap_or(usize::MAX);
    let total = braids.len();

    let has_more = offset + limit < total;
    let result: Vec<Braid> = braids.into_iter().skip(offset).take(limit).collect();

    (result, has_more)
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "test module: expect is standard in tests"
)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use sweet_grass_core::agent::Did;

    fn make_braid(hash: &str, agent: &str, size: u64) -> Braid {
        Braid::builder()
            .data_hash(hash)
            .mime_type("application/json")
            .size(size)
            .attributed_to(Did::new(agent))
            .build()
            .expect("should build")
    }

    #[test]
    fn test_matches_empty_filter() {
        let braid = make_braid("sha256:test", "did:key:z6MkTest", 1024);
        let filter = QueryFilter::new();

        assert!(matches(&braid, &filter));
    }

    #[test]
    fn test_matches_agent_filter() {
        let braid = make_braid("sha256:test", "did:key:z6MkTest", 1024);

        let matching = QueryFilter::new().with_agent(Did::new("did:key:z6MkTest"));
        assert!(matches(&braid, &matching));

        let not_matching = QueryFilter::new().with_agent(Did::new("did:key:z6MkOther"));
        assert!(!matches(&braid, &not_matching));
    }

    #[test]
    fn test_matches_hash_filter() {
        let braid = make_braid("sha256:test", "did:key:z6MkTest", 1024);

        let matching = QueryFilter::new().with_hash("sha256:test".to_string());
        assert!(matches(&braid, &matching));

        let not_matching = QueryFilter::new().with_hash("sha256:other".to_string());
        assert!(!matches(&braid, &not_matching));
    }

    #[test]
    fn test_matches_mime_prefix() {
        let braid = make_braid("sha256:test", "did:key:z6MkTest", 1024);

        // "application/json" starts with "application/"
        let matching = QueryFilter::new().with_mime_type("application/".to_string());
        assert!(matches(&braid, &matching));

        let not_matching = QueryFilter::new().with_mime_type("image/".to_string());
        assert!(!matches(&braid, &not_matching));
    }

    #[test]
    fn test_sort_newest_first() {
        let mut braids = vec![
            make_braid("sha256:a", "did:key:z6Mk", 100),
            make_braid("sha256:b", "did:key:z6Mk", 100),
            make_braid("sha256:c", "did:key:z6Mk", 100),
        ];

        // Manually set timestamps for predictable sorting
        braids[0].generated_at_time = 100;
        braids[1].generated_at_time = 300;
        braids[2].generated_at_time = 200;

        sort(&mut braids, &QueryOrder::NewestFirst);

        assert_eq!(braids[0].generated_at_time, 300);
        assert_eq!(braids[1].generated_at_time, 200);
        assert_eq!(braids[2].generated_at_time, 100);
    }

    #[test]
    fn test_sort_by_size() {
        let mut braids = vec![
            make_braid("sha256:a", "did:key:z6Mk", 500),
            make_braid("sha256:b", "did:key:z6Mk", 100),
            make_braid("sha256:c", "did:key:z6Mk", 300),
        ];

        sort(&mut braids, &QueryOrder::LargestFirst);

        assert_eq!(braids[0].size, 500);
        assert_eq!(braids[1].size, 300);
        assert_eq!(braids[2].size, 100);
    }

    #[test]
    fn test_paginate() {
        let braids: Vec<Braid> = (0..10)
            .map(|i| make_braid(&format!("sha256:{i}"), "did:key:z6Mk", 100))
            .collect();

        let filter = QueryFilter::new().with_offset(2).with_limit(3);
        let (result, has_more) = paginate(braids, &filter);

        assert_eq!(result.len(), 3);
        assert!(has_more);
    }

    #[test]
    fn test_paginate_no_more() {
        let braids: Vec<Braid> = (0..5)
            .map(|i| make_braid(&format!("sha256:{i}"), "did:key:z6Mk", 100))
            .collect();

        let filter = QueryFilter::new().with_offset(2).with_limit(10);
        let (result, has_more) = paginate(braids, &filter);

        assert_eq!(result.len(), 3);
        assert!(!has_more);
    }

    #[test]
    fn test_matches_time_range() {
        let mut braid = make_braid("sha256:time", "did:key:z6Mk", 100);
        braid.generated_at_time = 500;

        let in_range = QueryFilter::new().with_time_range(100, 900);
        assert!(matches(&braid, &in_range));

        let too_early = QueryFilter {
            created_after: Some(600),
            ..QueryFilter::new()
        };
        assert!(!matches(&braid, &too_early));

        let too_late = QueryFilter {
            created_before: Some(400),
            ..QueryFilter::new()
        };
        assert!(!matches(&braid, &too_late));
    }

    #[test]
    fn test_matches_braid_type() {
        let braid = make_braid("sha256:type", "did:key:z6Mk", 100);
        let matching = QueryFilter::new().with_type(sweet_grass_core::braid::BraidType::default());
        assert!(matches(&braid, &matching));
    }

    #[test]
    fn test_matches_tag() {
        let mut braid = make_braid("sha256:tag", "did:key:z6Mk", 100);
        braid.metadata.tags.push("important".into());

        let matching = QueryFilter::new().with_tag("important");
        assert!(matches(&braid, &matching));

        let not_matching = QueryFilter::new().with_tag("unrelated");
        assert!(!matches(&braid, &not_matching));
    }

    #[test]
    fn test_matches_ecop_source_primal() {
        let mut braid = make_braid("sha256:ecop", "did:key:z6Mk", 100);
        braid.ecop.source_primal = Some(Arc::from("sweetGrass"));

        let matching = QueryFilter {
            source_primal: Some("sweetGrass".to_string()),
            ..QueryFilter::new()
        };
        assert!(matches(&braid, &matching));

        let not_matching = QueryFilter {
            source_primal: Some("other".to_string()),
            ..QueryFilter::new()
        };
        assert!(!matches(&braid, &not_matching));
    }

    #[test]
    fn test_matches_ecop_niche() {
        let mut braid = make_braid("sha256:niche", "did:key:z6Mk", 100);
        braid.ecop.niche = Some(Arc::from("chemistry"));

        let matching = QueryFilter {
            niche: Some("chemistry".to_string()),
            ..QueryFilter::new()
        };
        assert!(matches(&braid, &matching));

        let not_matching = QueryFilter {
            niche: Some("biology".to_string()),
            ..QueryFilter::new()
        };
        assert!(!matches(&braid, &not_matching));
    }

    #[test]
    fn test_sort_oldest_first() {
        let mut braids = vec![
            make_braid("sha256:a", "did:key:z6Mk", 100),
            make_braid("sha256:b", "did:key:z6Mk", 100),
        ];
        braids[0].generated_at_time = 300;
        braids[1].generated_at_time = 100;

        sort(&mut braids, &QueryOrder::OldestFirst);
        assert_eq!(braids[0].generated_at_time, 100);
        assert_eq!(braids[1].generated_at_time, 300);
    }

    #[test]
    fn test_sort_smallest_first() {
        let mut braids = vec![
            make_braid("sha256:a", "did:key:z6Mk", 500),
            make_braid("sha256:b", "did:key:z6Mk", 100),
            make_braid("sha256:c", "did:key:z6Mk", 300),
        ];

        sort(&mut braids, &QueryOrder::SmallestFirst);
        assert_eq!(braids[0].size, 100);
        assert_eq!(braids[1].size, 300);
        assert_eq!(braids[2].size, 500);
    }

    #[test]
    fn test_matches_ecop_source_primal_none() {
        let braid = make_braid("sha256:no-ecop", "did:key:z6Mk", 100);
        let filter = QueryFilter {
            source_primal: Some("sweetGrass".to_string()),
            ..QueryFilter::new()
        };
        assert!(!matches(&braid, &filter));
    }

    #[test]
    fn test_matches_ecop_niche_none() {
        let braid = make_braid("sha256:no-niche", "did:key:z6Mk", 100);
        let filter = QueryFilter {
            niche: Some("chemistry".to_string()),
            ..QueryFilter::new()
        };
        assert!(!matches(&braid, &filter));
    }
}
