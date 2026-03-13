// SPDX-License-Identifier: AGPL-3.0-only
//! `SweetGrass` Demo - Full Attribution Pipeline
//!
//! This example demonstrates the complete `SweetGrass` workflow:
//! 1. Creating Braids from data
//! 2. Building derivation chains
//! 3. Calculating attribution
//! 4. Compressing sessions
//! 5. Querying provenance
//! 6. Exporting to PROV-O

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::too_many_lines)]

use std::sync::Arc;

use sweet_grass_compression::{CompressionEngine, Session, SessionOutcome, SessionVertex};
use sweet_grass_core::{
    activity::ActivityType, agent::Did, braid::BraidMetadata, entity::EntityReference,
};
use sweet_grass_factory::BraidFactory;
use sweet_grass_query::QueryEngine;
use sweet_grass_store::{BraidStore, MemoryStore};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🌾 SweetGrass Demo - Attribution Pipeline");
    println!("==========================================\n");

    // Create shared components
    let store: Arc<dyn BraidStore> = Arc::new(MemoryStore::new());
    let alice = Did::new("did:key:z6MkAlice");
    let bob = Did::new("did:key:z6MkBob");
    let charlie = Did::new("did:key:z6MkCharlie");

    let factory = Arc::new(BraidFactory::new(alice.clone()));
    let query_engine = Arc::new(QueryEngine::new(Arc::clone(&store)));
    let compression_engine = Arc::new(CompressionEngine::new(Arc::clone(&factory)));

    // =========================================
    // Part 1: Create Source Data Braids
    // =========================================
    println!("📝 Part 1: Creating Source Data Braids");
    println!("---------------------------------------");

    // Alice creates the original dataset
    let dataset_metadata = BraidMetadata {
        title: Some("Climate Research Dataset".to_string()),
        description: Some("Temperature measurements from sensor network".to_string()),
        tags: vec![
            "climate".to_string(),
            "sensors".to_string(),
            "research".to_string(),
        ],
        ..Default::default()
    };

    let dataset_braid = factory.from_data(
        b"temperature,humidity,timestamp\n22.5,65,2025-01-01T00:00:00Z\n23.1,62,2025-01-01T01:00:00Z",
        "text/csv",
        Some(dataset_metadata),
    )?;

    store.put(&dataset_braid).await?;
    println!("✅ Created dataset Braid: {}", dataset_braid.id);
    println!("   Hash: {}", dataset_braid.data_hash);
    println!("   Attributed to: {}\n", dataset_braid.was_attributed_to);

    // Bob creates analysis code
    let bob_factory = BraidFactory::new(bob.clone());
    let code_metadata = BraidMetadata {
        title: Some("Analysis Script".to_string()),
        description: Some("Python script for climate data analysis".to_string()),
        tags: vec![
            "code".to_string(),
            "python".to_string(),
            "analysis".to_string(),
        ],
        ..Default::default()
    };

    let code_braid = bob_factory.from_data(
        b"import pandas as pd\ndf = pd.read_csv('data.csv')\nresult = df.groupby('date').mean()",
        "text/x-python",
        Some(code_metadata),
    )?;

    store.put(&code_braid).await?;
    println!("✅ Created code Braid: {}", code_braid.id);
    println!("   Attributed to: {}\n", code_braid.was_attributed_to);

    // =========================================
    // Part 2: Create Derived Data Braid
    // =========================================
    println!("🔄 Part 2: Creating Derived Data Braid");
    println!("--------------------------------------");

    // Charlie runs analysis, creating derived output
    let charlie_factory = BraidFactory::new(charlie.clone());
    let sources = vec![
        EntityReference::by_hash(&dataset_braid.data_hash),
        EntityReference::by_hash(&code_braid.data_hash),
    ];

    let result_metadata = BraidMetadata {
        title: Some("Climate Analysis Results".to_string()),
        description: Some("Aggregated temperature trends by month".to_string()),
        tags: vec![
            "results".to_string(),
            "climate".to_string(),
            "derived".to_string(),
        ],
        ..Default::default()
    };

    let result_braid = charlie_factory.derived_from(
        b"month,avg_temp,trend\nJan,21.5,+0.3\nFeb,22.1,+0.5",
        "text/csv",
        sources,
        ActivityType::Computation,
        Some(result_metadata),
    )?;

    store.put(&result_braid).await?;
    println!("✅ Created result Braid: {}", result_braid.id);
    println!(
        "   Derived from {} sources",
        result_braid.was_derived_from.len()
    );
    println!("   Attributed to: {}\n", result_braid.was_attributed_to);

    // =========================================
    // Part 3: Query Provenance Graph
    // =========================================
    println!("🔍 Part 3: Querying Provenance Graph");
    println!("------------------------------------");

    let graph = query_engine
        .provenance_graph(EntityReference::by_hash(&result_braid.data_hash), Some(5))
        .await?;

    println!("Provenance graph for result:");
    println!("  Root: {}", result_braid.data_hash);
    println!("  Entities: {}", graph.entities.len());
    println!("  Depth: {}\n", graph.depth);

    // =========================================
    // Part 4: Calculate Attribution Chain
    // =========================================
    println!("📊 Part 4: Calculating Attribution Chain");
    println!("----------------------------------------");

    // Use full_attribution_chain to traverse derivation graph
    // This gives credit to Alice (dataset) and Bob (code), not just Charlie
    let attribution = query_engine
        .full_attribution_chain(&result_braid.data_hash, Some(5))
        .await?;

    println!("Attribution for result Braid:");
    for contributor in &attribution.contributors {
        println!(
            "  {} - {:.1}% ({:?})",
            contributor.agent,
            contributor.share * 100.0,
            contributor.role
        );
    }
    println!();

    // Calculate rewards
    let total_value = 1000.0; // $1000
    println!("Reward distribution ($1000 total):");
    for contributor in &attribution.contributors {
        let reward = contributor.share * total_value;
        println!("  {} receives ${:.2}", contributor.agent, reward);
    }
    println!();

    // =========================================
    // Part 5: Session Compression (0/1/Many)
    // =========================================
    println!("🗜️  Part 5: Session Compression");
    println!("-------------------------------");

    // Create a session with multiple vertices
    let mut session = Session::new("analysis-session-001");
    session.compute_units = 2.5;

    // Add vertices representing session activity
    session.add_vertex(
        SessionVertex::new("v1", "sha256:input-data", "text/csv", alice.clone())
            .with_size(1024)
            .with_activity_type(ActivityType::Import)
            .committed(),
    );

    session.add_vertex(
        SessionVertex::new("v2", "sha256:processed-data", "text/csv", bob.clone())
            .with_size(2048)
            .with_parent("v1")
            .with_activity_type(ActivityType::Transformation)
            .committed(),
    );

    session.add_vertex(
        SessionVertex::new(
            "v3",
            "sha256:final-output",
            "application/json",
            charlie.clone(),
        )
        .with_size(512)
        .with_parent("v2")
        .with_activity_type(ActivityType::Computation)
        .committed(),
    );

    session.finalize(SessionOutcome::Committed);

    // Compress the session
    let compression_result = compression_engine.compress(&session)?;

    println!("Session compression result:");
    println!("  Vertices: {}", session.vertex_count());
    println!("  Branches: {}", session.branch_count());
    println!("  Braids created: {}", compression_result.count());

    for braid in compression_result.braids() {
        store.put(braid).await?;
        println!("  Stored: {}", braid.id);
    }
    println!();

    // =========================================
    // Part 6: Export to PROV-O
    // =========================================
    println!("📤 Part 6: PROV-O Export");
    println!("------------------------");

    let provo_export = query_engine
        .export_braid_provo(&result_braid.data_hash)
        .await?;

    let json_str = serde_json::to_string_pretty(&provo_export)?;
    println!("PROV-O JSON-LD (truncated):");
    for (i, line) in json_str.lines().take(15).enumerate() {
        println!("  {line}");
        if i == 14 {
            println!("  ...");
        }
    }
    println!();

    // =========================================
    // Final Summary
    // =========================================
    println!("✨ Demo Complete!");
    println!("================");
    println!("Created:");
    println!("  - 1 source dataset Braid (Alice)");
    println!("  - 1 analysis code Braid (Bob)");
    println!("  - 1 derived result Braid (Charlie)");
    println!("  - 1 compressed session Braid");
    println!("\nSweetGrass successfully tracked:");
    println!("  - Data provenance (who created what)");
    println!("  - Derivation chains (what came from what)");
    println!("  - Attribution shares (who gets credit)");
    println!("  - PROV-O interoperability (W3C standard)");

    Ok(())
}
