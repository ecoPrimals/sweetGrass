// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project
//! `SweetGrass` `UniBin` — single binary, multiple modes.
//!
//! Follows the wateringHole `UNIBIN_ARCHITECTURE_STANDARD`:
//! - `sweetgrass server` — start REST + tarpc + JSON-RPC
//! - `sweetgrass status` — check running instance
//! - `sweetgrass --version` / `sweetgrass --help`
//!
//! Exit codes per standard:
//! - 0: success
//! - 1: general error
//! - 2: configuration error
//! - 3: network error

#![forbid(unsafe_code)]
#![warn(clippy::unwrap_used, clippy::expect_used)]

use std::net::SocketAddr;

use clap::{Parser, Subcommand};
use sweet_grass_service::{
    create_router, infant_bootstrap_with_config, start_tarpc_server, BootstrapConfig,
    StorageConfig, SweetGrassServer,
};
use tracing::info;

mod exit_code {
    pub const SUCCESS: i32 = 0;
    pub const GENERAL_ERROR: i32 = 1;
    pub const CONFIG_ERROR: i32 = 2;
    pub const NETWORK_ERROR: i32 = 3;
}

#[derive(Parser, Debug)]
#[command(name = "sweetgrass")]
#[command(
    version,
    about = "SweetGrass — semantic provenance and attribution layer"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start the `SweetGrass` service (REST + JSON-RPC + tarpc).
    Server {
        /// REST/JSON-RPC bind address.
        #[arg(long, env = "SWEETGRASS_HTTP_ADDRESS", default_value = "0.0.0.0:0")]
        http_address: String,

        /// tarpc bind address.
        #[arg(long, env = "SWEETGRASS_TARPC_ADDRESS", default_value = "0.0.0.0:0")]
        tarpc_address: String,

        /// Storage backend: memory, postgres, redb, sled (requires --features sled).
        #[arg(short, long, env = "STORAGE_BACKEND", default_value = "memory")]
        storage: String,

        /// `PostgreSQL` connection string (if storage=postgres).
        #[arg(long, env = "DATABASE_URL")]
        database_url: Option<String>,

        /// Sled database path (if storage=sled).
        #[arg(long, env = "STORAGE_PATH")]
        sled_path: Option<String>,

        /// Log level.
        #[arg(short, long, env = "RUST_LOG", default_value = "info")]
        log_level: String,

        /// Disable tarpc server (REST/JSON-RPC only).
        #[arg(long)]
        no_tarpc: bool,
    },

    /// Check status of a running `SweetGrass` instance.
    Status {
        /// Address of the running instance (discovered from env or explicit).
        #[arg(long, env = "SWEETGRASS_HTTP_ADDRESS")]
        address: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let code = match cli.command {
        Commands::Server {
            http_address,
            tarpc_address,
            storage,
            database_url,
            sled_path,
            log_level,
            no_tarpc,
        } => {
            let config = ServerConfig {
                http_address,
                tarpc_address,
                storage,
                database_url,
                sled_path,
                log_level,
                no_tarpc,
            };
            run_server(config).await
        },
        Commands::Status { address } => run_status(&address).await,
    };

    std::process::exit(code);
}

struct ServerConfig {
    http_address: String,
    tarpc_address: String,
    storage: String,
    database_url: Option<String>,
    sled_path: Option<String>,
    log_level: String,
    no_tarpc: bool,
}

async fn run_server(config: ServerConfig) -> i32 {
    let level = config
        .log_level
        .parse::<tracing::Level>()
        .unwrap_or(tracing::Level::INFO);
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false)
        .init();

    info!("SweetGrass starting — semantic provenance and attribution layer");

    let storage_config = StorageConfig {
        backend: config.storage.clone(),
        database_url: config.database_url.clone(),
        sled_path: config.sled_path.clone(),
        ..Default::default()
    };

    let bootstrap_config = BootstrapConfig {
        storage: storage_config,
        ..Default::default()
    };

    let bootstrap = match infant_bootstrap_with_config(bootstrap_config).await {
        Ok(b) => b,
        Err(e) => {
            tracing::error!("Bootstrap failed: {e}");
            return exit_code::CONFIG_ERROR;
        },
    };

    let state = bootstrap.app_state;
    info!(
        primal = %bootstrap.self_knowledge.name,
        storage = %config.storage,
        "Service initialized"
    );

    let app = create_router(state.clone());

    let http_addr = match parse_addr(&config.http_address, "HTTP") {
        Ok(a) => a,
        Err(code) => return code,
    };

    let http_listener = match tokio::net::TcpListener::bind(http_addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("Failed to bind HTTP on {http_addr}: {e}");
            return exit_code::NETWORK_ERROR;
        },
    };

    let actual_http_addr = http_listener.local_addr().unwrap_or(http_addr);
    info!("REST + JSON-RPC 2.0 listening on http://{actual_http_addr}");
    info!("  Health: http://{actual_http_addr}/health");
    info!("  JSON-RPC: POST http://{actual_http_addr}/jsonrpc");
    info!("  REST API: http://{actual_http_addr}/api/v1/");

    if !config.no_tarpc {
        let tarpc_addr = match parse_addr(&config.tarpc_address, "tarpc") {
            Ok(a) => a,
            Err(code) => return code,
        };
        spawn_tarpc_server(tarpc_addr, &state);
    }

    // Start Unix domain socket listener for biomeOS IPC
    #[cfg(unix)]
    {
        let uds_state = state.clone();
        tokio::spawn(async move {
            if let Err(e) = sweet_grass_service::uds::start_uds_listener(uds_state).await {
                tracing::warn!("UDS listener error: {e}");
            }
        });
    }

    axum::serve(http_listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_or_else(
            |e| {
                tracing::error!("Server error: {e}");
                #[cfg(unix)]
                sweet_grass_service::uds::cleanup_socket();
                exit_code::GENERAL_ERROR
            },
            |()| {
                #[cfg(unix)]
                sweet_grass_service::uds::cleanup_socket();
                info!("SweetGrass shut down cleanly");
                exit_code::SUCCESS
            },
        )
}

fn parse_addr(addr: &str, label: &str) -> Result<SocketAddr, i32> {
    addr.parse().map_err(|e| {
        tracing::error!("Invalid {label} address '{addr}': {e}");
        exit_code::CONFIG_ERROR
    })
}

fn spawn_tarpc_server(tarpc_addr: SocketAddr, state: &sweet_grass_service::AppState) {
    let server = SweetGrassServer::from_app_state(state);
    tokio::spawn(async move {
        info!("tarpc server starting on {tarpc_addr}");
        if let Err(e) = start_tarpc_server(tarpc_addr, server).await {
            tracing::error!("tarpc server error: {e}");
        }
    });
}

async fn shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();

    #[cfg(unix)]
    {
        let mut sigterm =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).ok();
        tokio::select! {
            () = async { let _ = ctrl_c.await; } => info!("Received SIGINT, shutting down gracefully"),
            () = async {
                if let Some(s) = sigterm.as_mut() {
                    s.recv().await;
                } else {
                    std::future::pending::<()>().await;
                }
            } => {
                info!("Received SIGTERM, shutting down gracefully");
            },
        }
    }

    #[cfg(not(unix))]
    {
        let _ = ctrl_c.await;
        info!("Received shutdown signal");
    }
}

async fn run_status(address: &str) -> i32 {
    let url = format!("http://{address}/health");
    println!("Checking SweetGrass at {url}...");

    match http_health_check(address).await {
        Ok(body) => {
            println!("SweetGrass is healthy at {address}");
            println!("  {body}");
            exit_code::SUCCESS
        },
        Err(e) => {
            eprintln!("Cannot reach SweetGrass at {address}: {e}");
            exit_code::NETWORK_ERROR
        },
    }
}

/// Errors from the raw TCP health check.
#[derive(Debug, thiserror::Error)]
enum HealthCheckError {
    /// TCP connection or IO failure.
    #[error("{0}")]
    Io(#[from] std::io::Error),

    /// Server responded with a non-200 status.
    #[error("unhealthy response: {0}")]
    Unhealthy(String),
}

/// Perform a minimal HTTP GET /health check using raw TCP.
///
/// Pure Rust implementation — no reqwest or hyper dependency needed.
/// Sends a bare HTTP/1.1 request and parses the response body.
async fn http_health_check(address: &str) -> Result<String, HealthCheckError> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut stream = tokio::net::TcpStream::connect(address).await?;

    let request = format!("GET /health HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes()).await?;
    stream.shutdown().await?;

    let mut response = String::new();
    stream.read_to_string(&mut response).await?;

    let body = response
        .split_once("\r\n\r\n")
        .map(|(_, body)| body.to_string())
        .unwrap_or_default();

    if response.contains("200 OK") {
        Ok(body)
    } else {
        let status_line = response.lines().next().unwrap_or("").to_string();
        Err(HealthCheckError::Unhealthy(status_line))
    }
}
