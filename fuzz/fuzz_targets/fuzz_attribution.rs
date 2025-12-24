//! Fuzz test for attribution calculations.
//!
//! Tests that arbitrary attribution inputs don't cause panics and produce valid results.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use sweet_grass_core::agent::{AgentRole, Did};
use sweet_grass_core::entity::EntityReference;
use sweet_grass_factory::{AttributionCalculator, AttributionChain, ContributorShare};

/// Arbitrary contributor input for fuzzing.
#[derive(Debug, Arbitrary)]
struct FuzzContributor {
    did_suffix: String,
    share: u8, // Will be converted to 0.0-1.0
    role: u8,  // Will be mapped to AgentRole
    direct: bool,
}

/// Arbitrary attribution input for fuzzing.
#[derive(Debug, Arbitrary)]
struct FuzzAttribution {
    hash: String,
    contributors: Vec<FuzzContributor>,
    total_value: u32, // For reward calculation
}

fn u8_to_role(r: u8) -> AgentRole {
    match r % 6 {
        0 => AgentRole::Creator,
        1 => AgentRole::Contributor,
        2 => AgentRole::DataProvider,
        3 => AgentRole::ComputeProvider,
        4 => AgentRole::Curator,
        _ => AgentRole::Publisher,
    }
}

fuzz_target!(|input: FuzzAttribution| {
    // Create a chain from arbitrary input
    let entity = EntityReference::by_hash(format!("sha256:{}", input.hash));
    let mut chain = AttributionChain::new(entity);

    // Add contributors
    for c in input.contributors.iter().take(100) {
        // Limit to prevent OOM
        let did = Did::new(format!("did:key:z6Mk{}", c.did_suffix));
        let share = f64::from(c.share) / 255.0; // Normalize to 0.0-1.0
        let role = u8_to_role(c.role);

        chain.add_contributor(ContributorShare::new(did, share, role, c.direct));
    }

    // Normalize should not panic
    chain.normalize();

    // Check validity
    let _ = chain.is_valid();

    // Aggregate should not panic
    let _ = chain.aggregate_by_agent();

    // Get contributors
    let _ = chain.direct_contributors();
    let _ = chain.inherited_contributors();

    // Calculate rewards if we have contributors
    if !chain.contributors.is_empty() && chain.is_valid() {
        let total_value = f64::from(input.total_value);
        let _ = sweet_grass_factory::calculate_rewards(&chain, total_value);
    }

    // Create a calculator and calculate
    let calculator = AttributionCalculator::default();

    // Create a test braid if we have contributors
    if !input.contributors.is_empty() {
        let first = &input.contributors[0];
        let did = Did::new(format!("did:key:z6Mk{}", first.did_suffix));

        let braid = sweet_grass_core::Braid::builder()
            .data_hash(&format!("sha256:{}", input.hash))
            .mime_type("text/plain")
            .size(100)
            .attributed_to(did)
            .build();

        if let Ok(braid) = braid {
            let _ = calculator.calculate_single(&braid);
        }
    }
});

