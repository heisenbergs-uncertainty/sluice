//! Sluice: A gRPC-native message broker with credit-based flow control.
//!
//! # Usage
//!
//! ```bash
//! sluice --port 50051 --data-dir ./data --log-level info
//! ```
//!
//! Environment variables can also be used:
//! - `SLUICE_PORT`: Port to listen on
//! - `SLUICE_DATA_DIR`: Data directory for SQLite
//! - `RUST_LOG`: Log level (trace, debug, info, warn, error)

use sluice::config::Config;
use sluice::observability::metrics::init_metrics_with_endpoint;
use sluice::observability::tracing::init_tracing;
use sluice::server::run_server;
use std::fs;
use tokio::sync::watch;

/// Print startup banner with version and configuration.
fn print_banner(config: &Config) {
    let version = env!("CARGO_PKG_VERSION");
    eprintln!(
        r#"
    ____  __        _          
   / ___|| |_   _(_) ___ ___  
   \___ \| | | | | |/ __/ _ \ 
    ___) | | |_| | | (_|  __/ 
   |____/|_|\__,_|_|\___\___| 
                              
  Sluice v{} - gRPC Message Broker
  
  Configuration:
    Address:    {}:{}
    Data Dir:   {}
    Log Level:  {}
    
  Press Ctrl+C to shutdown gracefully.
"#,
        version,
        config.host,
        config.port,
        config.data_dir.display(),
        config.log_level
    );
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse configuration from CLI arguments and environment
    let config = Config::parse_args();

    // Initialize tracing/logging
    init_tracing(&config.log_level);

    // Initialize metrics (with optional OTLP export)
    init_metrics_with_endpoint(config.otel_endpoint.as_deref());

    // Ensure data directory exists
    fs::create_dir_all(&config.data_dir)?;

    // Print startup banner
    print_banner(&config);

    // Create shutdown signal channel
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    // Spawn signal handler task
    let shutdown_tx_clone = shutdown_tx.clone();
    tokio::spawn(async move {
        // Wait for SIGTERM or SIGINT (Ctrl+C)
        let ctrl_c = tokio::signal::ctrl_c();

        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};
            let mut sigterm =
                signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");

            tokio::select! {
                _ = ctrl_c => {
                    tracing::info!("Received SIGINT (Ctrl+C), initiating shutdown...");
                }
                _ = sigterm.recv() => {
                    tracing::info!("Received SIGTERM, initiating shutdown...");
                }
            }
        }

        #[cfg(not(unix))]
        {
            ctrl_c.await.expect("failed to listen for ctrl+c");
            tracing::info!("Received Ctrl+C, initiating shutdown...");
        }

        // Signal shutdown
        let _ = shutdown_tx_clone.send(true);
    });

    // Run the server
    run_server(config, shutdown_rx).await?;

    tracing::info!("Sluice shutdown complete");
    Ok(())
}
