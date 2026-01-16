//! gRPC server setup and lifecycle.
//!
//! Configures tonic server with:
//! - Publish and Subscribe service handlers
//! - Graceful shutdown support
//! - Health check endpoint

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::watch;
use tonic::transport::Server;

use crate::config::Config;
use crate::flow::notify::NotificationBus;
use crate::observability::metrics::prometheus_registry;
use crate::observability::prometheus::run_prometheus_server;
use crate::proto::sluice::v1::sluice_server::SluiceServer;
use crate::service::{ConnectionRegistry, SluiceService};
use crate::storage::batch::BatchConfig;
use crate::storage::reader::ReaderPool;
use crate::storage::writer::{Writer, WriterHandle};

/// Server state shared across handlers.
pub struct ServerState {
    pub writer: WriterHandle,
    pub reader_pool: ReaderPool,
    pub notify_bus: NotificationBus,
    pub connection_registry: ConnectionRegistry,
}

/// Run the Sluice gRPC server.
///
/// # Arguments
///
/// * `config` - Server configuration
/// * `shutdown_rx` - Receiver for shutdown signal
///
/// # Returns
///
/// Returns when the server has shut down.
pub async fn run_server(
    config: Config,
    mut shutdown_rx: watch::Receiver<bool>,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;

    // Create notification bus
    let notify_bus = NotificationBus::new(config.notify_channel_size);

    // Create batch config from application config
    let batch_config = BatchConfig::from_config(config.batch_size, config.batch_delay_ms);

    // Spawn writer thread
    let writer = Writer::spawn(
        config.data_dir.join("sluice.db"),
        notify_bus.clone(),
        config.write_channel_size,
        batch_config,
        config.wal_checkpoint_pages,
    )?;
    let writer_handle = writer.handle();

    // Create reader pool
    let reader_pool = ReaderPool::new(config.data_dir.join("sluice.db"), config.reader_pool_size)?;

    // Create shared state
    let state = Arc::new(ServerState {
        writer: writer_handle.clone(),
        reader_pool,
        notify_bus,
        connection_registry: ConnectionRegistry::new(),
    });

    // Create service
    let service = SluiceService::new(state);

    // Spawn Prometheus metrics server if enabled
    if config.metrics_enabled {
        let metrics_addr: SocketAddr = format!("{}:{}", config.host, config.metrics_port).parse()?;
        let registry = prometheus_registry();
        let metrics_shutdown_rx = shutdown_rx.clone();

        tokio::spawn(async move {
            if let Err(e) = run_prometheus_server(metrics_addr, registry, metrics_shutdown_rx).await {
                tracing::error!(error = %e, "Prometheus server error");
            }
        });
    }

    tracing::info!(address = %addr, "Starting Sluice gRPC server");

    // Run server with graceful shutdown
    Server::builder()
        .add_service(SluiceServer::new(service))
        .serve_with_shutdown(addr, async move {
            // Wait for shutdown signal
            let _ = shutdown_rx.changed().await;
            tracing::info!("Shutdown signal received, stopping server");
        })
        .await?;

    // Shutdown writer
    tracing::info!("Shutting down writer thread");
    writer_handle.shutdown().await?;
    writer.join()?;

    tracing::info!("Server stopped");
    Ok(())
}
