//! SweetGrass Service Binary
//!
//! Pure Rust attribution service with:
//! - REST API (Axum)
//! - tarpc RPC (optional)
//! - Multiple storage backends
//! - Environment-driven configuration

#![forbid(unsafe_code)]
#![warn(clippy::unwrap_used, clippy::expect_used)]

use std::net::SocketAddr;

use clap::Parser;
use sweet_grass_core::agent::Did;
use sweet_grass_service::{create_router, AppState};
use tracing::info;

#[derive(Parser, Debug)]
#[command(name = "sweet-grass-service")]
#[command(version, about = "SweetGrass Attribution Service", long_about = None)]
struct Cli {
    /// REST API port
    #[arg(short, long, default_value = "8080")]
    port: u16,

    /// Storage backend: memory, postgres, sled
    #[arg(short, long, default_value = "memory")]
    storage: String,

    /// PostgreSQL connection string (if storage=postgres)
    #[arg(long, env = "DATABASE_URL")]
    database_url: Option<String>,

    /// Sled database path (if storage=sled)
    #[arg(long, default_value = "./data/sweetgrass.db")]
    sled_path: String,

    /// Log level
    #[arg(short, long, default_value = "info")]
    log_level: String,

    /// Default agent DID for braid creation
    #[arg(long, default_value = "did:key:z6MkSweetGrass")]
    default_agent: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse CLI args
    let cli = Cli::parse();

    // Initialize tracing
    let log_level = cli
        .log_level
        .parse::<tracing::Level>()
        .unwrap_or(tracing::Level::INFO);
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .init();

    info!("🌾 Starting SweetGrass Attribution Service");
    info!("Storage: {}", cli.storage);

    // Initialize app state
    let default_agent = Did::new(&cli.default_agent);
    let state = match cli.storage.as_str() {
        "memory" => {
            info!("Using in-memory storage");
            AppState::new_memory(default_agent)
        },
        _ => {
            return Err(format!("Unsupported storage backend: {}", cli.storage).into());
        },
    };

    // Create router with all handlers (includes state)
    let app = create_router(state);

    // Bind and serve
    let addr = SocketAddr::from(([0, 0, 0, 0], cli.port));
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("✅ SweetGrass REST API ready");
    info!("   → http://0.0.0.0:{}", cli.port);
    info!("   → Health: http://0.0.0.0:{}/health", cli.port);

    axum::serve(listener, app).await?;

    Ok(())
}
