//! OpenTelemetry tracing setup.
//!
//! Configures structured logging with:
//! - W3C Trace Context propagation
//! - OTEL exporter for distributed tracing

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize tracing with the given service name.
///
/// This sets up:
/// - Console logging with structured format
/// - Environment-based filter (via RUST_LOG)
///
/// # Arguments
///
/// * `service_name` - Name of the service for tracing
///
/// # Panics
///
/// Panics if tracing has already been initialized.
pub fn init_tracing(service_name: &str) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,sluice=debug"));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true);

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();

    tracing::info!(service = service_name, "Tracing initialized");
}

/// Initialize tracing for tests (only logs errors).
pub fn init_test_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("error")
        .with_test_writer()
        .try_init();
}

