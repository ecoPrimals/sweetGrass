// SPDX-License-Identifier: AGPL-3.0-or-later
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
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

use std::io::Write;

use clap::{Parser, Subcommand};
use sweet_grass_service::cli;
use sweet_grass_service::exit::exit_code;
use sweet_grass_service::{
    BootstrapConfig, StorageConfig, SweetGrassServer, create_router, infant_bootstrap_with_config,
    start_tarpc_server, start_tcp_jsonrpc_listener,
};
use tracing::info;

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
        /// TCP JSON-RPC port (newline-delimited framing, opt-in).
        ///
        /// Per Tower Atomic portability standard: TCP is opt-in only.
        /// UDS is the default transport. Pass `--port 0` for OS-assigned,
        /// or `--port <N>` for a specific port. Omit to run UDS-only.
        #[arg(long, env = "SWEETGRASS_PORT")]
        port: Option<u16>,

        /// REST/JSON-RPC bind address (HTTP-wrapped).
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

        /// Unix domain socket path override.
        ///
        /// When omitted, the socket path is resolved via the standard
        /// 4-tier fallback: `SWEETGRASS_SOCKET` env → `BIOMEOS_SOCKET_DIR` →
        /// `XDG_RUNTIME_DIR/biomeos/` → `$TMPDIR/sweetgrass.sock`.
        #[arg(long, env = "SWEETGRASS_SOCKET")]
        socket: Option<String>,
    },

    /// Check status of a running `SweetGrass` instance.
    Status {
        /// Address of the running instance (discovered from env or explicit).
        #[arg(long, env = "SWEETGRASS_HTTP_ADDRESS")]
        address: String,
    },

    /// Print all capabilities this primal offers (offline, no network).
    Capabilities,

    /// Print the resolved Unix domain socket path.
    Socket,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let code = match cli.command {
        Commands::Server {
            port,
            http_address,
            tarpc_address,
            storage,
            database_url,
            sled_path,
            log_level,
            no_tarpc,
            socket,
        } => {
            run_server(ServerConfig {
                port,
                http_address,
                tarpc_address,
                storage,
                database_url,
                sled_path,
                log_level,
                no_tarpc,
                socket,
            })
            .await
        },
        Commands::Status { address } => run_status(&address).await,
        Commands::Capabilities => run_capabilities(),
        Commands::Socket => run_socket(),
    };

    std::process::exit(code);
}

struct ServerConfig {
    port: Option<u16>,
    http_address: String,
    tarpc_address: String,
    storage: String,
    database_url: Option<String>,
    sled_path: Option<String>,
    log_level: String,
    no_tarpc: bool,
    socket: Option<String>,
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

    #[cfg(unix)]
    if let Err(e) = sweet_grass_service::uds::validate_insecure_guard() {
        tracing::error!("{e}");
        return exit_code::CONFIG_ERROR;
    }

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

    let http_addr = match cli::parse_socket_addr(&config.http_address).map_err(|msg| {
        tracing::error!("HTTP {msg}");
        exit_code::CONFIG_ERROR
    }) {
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

    serve_all(config, state, app, http_listener).await
}

async fn serve_all(
    config: ServerConfig,
    state: sweet_grass_service::AppState,
    app: axum::Router,
    http_listener: tokio::net::TcpListener,
) -> i32 {
    let (shutdown_tx, _) = tokio::sync::watch::channel(false);

    let tarpc_handle = if config.no_tarpc {
        None
    } else {
        let tarpc_addr = match cli::parse_socket_addr(&config.tarpc_address).map_err(|msg| {
            tracing::error!("tarpc {msg}");
            exit_code::CONFIG_ERROR
        }) {
            Ok(a) => a,
            Err(code) => return code,
        };
        let shutdown_rx = shutdown_tx.subscribe();
        Some(spawn_tarpc_server(tarpc_addr, &state, shutdown_rx))
    };

    let tcp_jsonrpc_handle = config.port.map(|port| {
        let tcp_state = state.clone();
        let shutdown_rx = shutdown_tx.subscribe();
        tracing::info!(port, "TCP JSON-RPC opt-in enabled");
        tokio::spawn(async move {
            if let Err(e) = start_tcp_jsonrpc_listener(tcp_state, port, shutdown_rx).await {
                tracing::warn!("TCP JSON-RPC listener error: {e}");
            }
        })
    });
    if config.port.is_none() {
        tracing::info!("TCP JSON-RPC disabled (UDS-only mode — pass --port to enable)");
    }

    #[cfg(unix)]
    let uds_socket_path = config.socket.as_ref().map(std::path::PathBuf::from);

    #[cfg(unix)]
    let uds_handle = {
        let uds_state = state.clone();
        let shutdown_rx = shutdown_tx.subscribe();
        let explicit_path = uds_socket_path.clone();
        Some(tokio::spawn(async move {
            let result = if let Some(ref path) = explicit_path {
                sweet_grass_service::uds::start_uds_listener_at(uds_state, path, shutdown_rx).await
            } else {
                sweet_grass_service::uds::start_uds_listener(uds_state, shutdown_rx).await
            };
            if let Err(e) = result {
                tracing::warn!("UDS listener error: {e}");
            }
        }))
    };

    let result = axum::serve(http_listener, app)
        .with_graceful_shutdown(async move {
            shutdown_signal().await;
            let _ = shutdown_tx.send(true);
        })
        .await;

    if let Some(h) = tarpc_handle {
        let _ = h.await;
    }
    if let Some(h) = tcp_jsonrpc_handle {
        let _ = h.await;
    }
    #[cfg(unix)]
    if let Some(h) = uds_handle {
        let _ = h.await;
    }

    #[cfg(unix)]
    if let Some(ref path) = uds_socket_path {
        sweet_grass_service::uds::cleanup_socket_at(path);
    } else {
        sweet_grass_service::uds::cleanup_socket();
    }

    result.map_or_else(
        |e| {
            tracing::error!("Server error: {e}");
            exit_code::GENERAL_ERROR
        },
        |()| {
            info!("SweetGrass shut down cleanly");
            exit_code::SUCCESS
        },
    )
}

fn spawn_tarpc_server(
    tarpc_addr: std::net::SocketAddr,
    state: &sweet_grass_service::AppState,
    shutdown: tokio::sync::watch::Receiver<bool>,
) -> tokio::task::JoinHandle<()> {
    let server = SweetGrassServer::from_app_state(state);
    tokio::spawn(async move {
        info!("tarpc server starting on {tarpc_addr}");
        if let Err(e) = start_tarpc_server(tarpc_addr, server, shutdown).await {
            tracing::error!("tarpc server error: {e}");
        }
    })
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

fn run_capabilities() -> i32 {
    let report = cli::capabilities_report(env!("CARGO_PKG_VERSION"));
    let output = cli::format_capabilities_report(&report);
    let _ = std::io::stdout().lock().write_all(output.as_bytes());
    exit_code::SUCCESS
}

fn run_socket() -> i32 {
    #[cfg(unix)]
    {
        let path = sweet_grass_service::uds::resolve_socket_path(None);
        let _ = writeln!(std::io::stdout().lock(), "{}", path.display());
        exit_code::SUCCESS
    }
    #[cfg(not(unix))]
    {
        tracing::error!("Unix domain sockets are not available on this platform");
        exit_code::GENERAL_ERROR
    }
}

async fn run_status(address: &str) -> i32 {
    let _ = writeln!(
        std::io::stdout().lock(),
        "Checking SweetGrass at http://{address}/health..."
    );

    match cli::http_health_check(address).await {
        Ok(body) => {
            let stdout = std::io::stdout();
            let mut out = stdout.lock();
            let _ = writeln!(out, "SweetGrass is healthy at {address}");
            let _ = writeln!(out, "  {body}");
            exit_code::SUCCESS
        },
        Err(e) => {
            tracing::error!("Cannot reach SweetGrass at {address}: {e}");
            exit_code::NETWORK_ERROR
        },
    }
}
